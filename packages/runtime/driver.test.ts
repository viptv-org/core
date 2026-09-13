import test from 'node:test';
import assert from 'node:assert/strict';
import { createCoreDriver, tauriCorePort } from './src/driver.ts';

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

test('Tauri port maps the exact native command arguments', async () => {
  const calls: unknown[] = [];
  const port = tauriCorePort(async <T>(command: string, args?: Record<string, unknown>): Promise<T> => {
    calls.push([command, args]); return '[]' as T;
  });
  await port.update('"Retry"'); await port.resolve(12, '{"Ok":null}'); await port.view();
  assert.deepEqual(calls, [['core_update', { event: '"Retry"' }],
    ['core_resolve', { id: 12, result: '{"Ok":null}' }], ['core_view', undefined]]);
});
