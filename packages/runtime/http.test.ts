import test from 'node:test';
import assert from 'node:assert/strict';
import { createHttpTransport, type HttpRequest } from './src/index.ts';
import { createNativeHttpTransport } from './src/native-fetch.ts';

const origin = 'https://backend.example';
const request: HttpRequest = { method: 'PATCH', url: `${origin}/api/profile`, headers: [
  { name: 'authorization', value: 'Bearer fixture' },
], body: [0, 127, 255] };

test('preserves binary request/response, headers, method and HTTP errors', async () => {
  const run = createHttpTransport({ allowedOrigins: [origin], fetch: async (url, init) => {
    assert.equal(url, request.url);
    assert.equal(init.method, 'PATCH');
    assert.equal(new Headers(init.headers).get('authorization'), 'Bearer fixture');
    assert.deepEqual(Array.from(init.body as Uint8Array), [0, 127, 255]);
    assert.equal(init.redirect, 'error');
    assert.equal(init.credentials, 'omit');
    return new Response(new Uint8Array([255, 0]), { status: 401, headers: { 'retry-after': '4' } });
  } });
  assert.deepEqual(await run(request), { Ok: { status: 401, headers: [{ name: 'retry-after', value: '4' }], body: [255, 0] } });
});

test('rejects wrong origins and credential URLs without invoking fetch', async () => {
  let calls = 0;
  const run = createHttpTransport({ allowedOrigins: [origin], fetch: async () => { calls++; return new Response(); } });
  for (const url of ['https://backend.example.attacker.test/api', 'https://user:pass@backend.example/api', 'file:///api']) {
    assert.ok('Err' in await run({ ...request, url }));
  }
  assert.equal(calls, 0);
});

test('rejects redirects and oversized streams', async () => {
  const redirect = createHttpTransport({ allowedOrigins: [origin], fetch: async () => new Response(null, {
    status: 302, headers: { location: 'https://other.example' },
  }) });
  assert.ok('Err' in await redirect(request));
  const large = createHttpTransport({ allowedOrigins: [origin], maxResponseBytes: 1,
    fetch: async () => new Response(new Uint8Array([1, 2])) });
  assert.deepEqual(await large(request), { Err: { Io: 'HTTP response exceeds the size limit' } });
});

test('aborts before dispatch and during an active request', async () => {
  let calls = 0;
  const controller = new AbortController();
  controller.abort();
  const run = createHttpTransport({ allowedOrigins: [origin], fetch: async () => { calls++; return new Response(); } });
  assert.deepEqual(await run(request, controller.signal), { Err: { Io: 'Request cancelled' } });
  assert.equal(calls, 0);
  const active = new AbortController();
  const pending = createHttpTransport({ allowedOrigins: [origin], fetch: (_, init) => new Promise((_, reject) => {
    init.signal!.addEventListener('abort', () => reject(new Error('secret URL must not leak')));
    active.abort();
  }) });
  assert.deepEqual(await pending(request, active.signal), { Err: { Io: 'Request cancelled' } });
});

test('deadline covers response body reading', async () => {
  const run = createHttpTransport({ allowedOrigins: [origin], timeoutMs: 5, fetch: async (_, init) => {
    const stream = new ReadableStream({ start(controller) {
      init.signal!.addEventListener('abort', () => controller.error(new Error('aborted')));
    } });
    return new Response(stream);
  } });
  assert.deepEqual(await run(request), { Err: 'Timeout' });
});

test('native fetch receives explicit redirect limit and cancellation signal', async () => {
  const run = createNativeHttpTransport({ allowedOrigins: [origin] }, async (_, init) => {
    assert.equal(init.maxRedirections, 0);
    assert.equal(init.redirect, 'error');
    assert.ok(init.signal instanceof AbortSignal);
    return new Response(new Uint8Array([255]), { status: 503, headers: { 'retry-after': '1' } });
  });
  assert.deepEqual(await run(request), { Ok: { status: 503,
    headers: [{ name: 'retry-after', value: '1' }], body: [255] } });
});
