import test from 'node:test';
import assert from 'node:assert/strict';
import { createCoreDriver, tauriCorePort } from './src/driver.ts';

test('HTTP batches start concurrently before a slow native view and render once', async () => {
  let releaseView!: (value: string) => void;
  const slowView = new Promise<string>(resolve => { releaseView = resolve; });
  let viewCalls = 0;
  let renders = 0;
  const started: number[] = [];
  const responses = new Map<number, (value: { Ok: { status: number; headers: []; body: [] } }) => void>();
  const resolved: number[] = [];
  const driver = createCoreDriver({
    core: {
      update: () => JSON.stringify([
        { id: 1, effect: { Render: null } },
        ...[2, 3].map(id => ({ id, effect: { Http: { method: 'GET', url: `https://backend.example/${id}`, headers: [], body: [] } } })),
        { id: 4, effect: { Render: null } },
      ]),
      resolve: id => { resolved.push(id); return '[]'; },
      view: () => { viewCalls++; return slowView; },
    },
    storage: { load: () => null, save: () => {}, clear: () => {} },
    http: request => new Promise(resolve => {
      const id = Number(new URL(request.url).pathname.slice(1));
      started.push(id);
      responses.set(id, resolve);
    }),
    render: () => { renders++; },
    onError: message => assert.fail(message),
  });
  const dispatch = driver.dispatch('Retry');
  await new Promise(resolve => setImmediate(resolve));
  const beforeView = [...started];
  releaseView(JSON.stringify({ phase: 'Profiles', identity: null, selectedProfileId: null, error: null }));
  await dispatch;
  responses.get(3)!({ Ok: { status: 200, headers: [], body: [] } });
  await new Promise(resolve => setImmediate(resolve));
  responses.get(2)!({ Ok: { status: 200, headers: [], body: [] } });
  await driver.idle();
  driver.dispose();
  assert.deepEqual(beforeView, [2, 3]);
  assert.equal(viewCalls, 1);
  assert.equal(renders, 1);
  assert.deepEqual(resolved, [3, 2]);
});

test('one dispatcher drains storage, HTTP and render through correlated native/WASM ports', async () => {
  const resolved: Array<[number, unknown]> = [];
  const views: unknown[] = [];
  const driver = createCoreDriver({
    core: {
      update: event => {
        assert.deepEqual(JSON.parse(event), { Begin: { origin: 'https://backend.example', allowInsecurePreview: false } });
        return JSON.stringify([{ id: 1, effect: { Storage: 'Load' } }]);
      },
      resolve: (id, value) => {
        resolved.push([id, JSON.parse(value)]);
        return JSON.stringify(id === 1
          ? [{ id: 2, effect: { Http: { method: 'GET', url: 'https://backend.example/api/auth/me', headers: [], body: [] } } }]
          : [{ id: 3, effect: { Render: null } }]);
      },
      view: () => JSON.stringify({ phase: 'Profiles', identity: null, selectedProfileId: null, error: null }),
    },
    storage: { load: () => 'fixture-tokens', save: () => {}, clear: () => {} },
    http: async () => ({ Ok: { status: 200, headers: [], body: [0, 255] } }),
    render: view => views.push(view),
    onError: message => assert.fail(message),
  });
  await driver.dispatch({ Begin: { origin: 'https://backend.example', allowInsecurePreview: false } });
  await driver.idle();
  assert.deepEqual(resolved, [[1, { Ok: 'fixture-tokens' }], [2, { Ok: { status: 200, headers: [], body: [0, 255] } }]]);
  assert.equal(views.length, 1);
  driver.dispose();
});

test('disposal cancels pending HTTP and suppresses core resolution/render', async () => {
  let aborted = false;
  const driver = createCoreDriver({
    core: {
      update: () => JSON.stringify([{ id: 4, effect: { Http: { method: 'GET', url: 'https://backend.example', headers: [], body: [] } } }]),
      resolve: () => assert.fail('Disposed core must not be resolved'),
      view: () => assert.fail('Disposed core must not be read'),
    },
    storage: { load: () => null, save: () => {}, clear: () => {} },
    http: (_, signal) => new Promise(resolve => signal.addEventListener('abort', () => {
      aborted = true; resolve({ Err: { Io: 'Request cancelled' } });
    })),
    render: () => assert.fail('Disposed view must not render'),
    onError: message => assert.fail(message),
  });
  await driver.dispatch('Retry');
  driver.dispose();
  await driver.idle();
  assert.equal(aborted, true);
  await assert.rejects(driver.dispatch('Retry'), /disposed/);
});

test('disposal during an asynchronous native view suppresses its render', async () => {
  let finish!: (view: string) => void;
  const view = new Promise<string>(resolve => { finish = resolve; });
  const driver = createCoreDriver({
    core: {
      update: () => JSON.stringify([{ id: 1, effect: { Render: null } }]),
      resolve: () => assert.fail('No response expected'),
      view: () => view,
    },
    storage: { load: () => null, save: () => {}, clear: () => {} },
    http: async () => assert.fail('No HTTP expected'),
    render: () => assert.fail('Disposed view must not render'),
    onError: message => assert.fail(message),
  });
  const dispatch = driver.dispatch('Retry');
  await new Promise(resolve => setImmediate(resolve));
  driver.dispose();
  finish('{}');
  await dispatch;
  await driver.idle();
});

test('Tauri port maps the exact native command arguments', async () => {
  const calls: unknown[] = [];
  const port = tauriCorePort(async <T>(command: string, args?: Record<string, unknown>): Promise<T> => {
    calls.push([command, args]); return '[]' as T;
  });
  await port.update('"Retry"'); await port.resolve(12, '{"Ok":null}'); await port.view();
  assert.deepEqual(calls, [['core_update', { event: '"Retry"' }],
    ['core_resolve', { id: 12, result: '{"Ok":null}' }], ['core_view', undefined]]);
});
