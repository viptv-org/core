import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
// A data import allows the generated browser module to run under Node without
// changing its package metadata or maintaining a separate WASM implementation.
const root = new URL('../', import.meta.url);
const moduleSource = await readFile(new URL('generated/wasm/viptv_core.js', root), 'utf8');
const core = await import(`data:text/javascript;base64,${Buffer.from(moduleSource).toString('base64')}`);
await core.default({ module_or_path: await readFile(new URL('generated/wasm/viptv_core_bg.wasm', root)) });
const vectors = JSON.parse(await readFile(new URL('tests/bridge-vectors.json', root), 'utf8'));
for (const scenario of vectors) {
  const app = new core.CoreBridge();
  let requests = [];
  for (const step of scenario.steps) {
    if ('event' in step) requests = JSON.parse(app.update(JSON.stringify(step.event)));
    else {
      const request = requests.find(request => step.effect in request.effect);
      assert.ok(request, scenario.name);
      requests = JSON.parse(app.resolve(request.id, JSON.stringify(step.output)));
    }
    const view = JSON.parse(app.view());
    assert.equal(view.phase, step.phase, scenario.name);
    if ('profileId' in step) assert.equal(view.selectedProfileId, step.profileId);
  }
  app.free();
}
const catalogs = JSON.parse(core.normalize('catalogs', JSON.stringify([{ id: 'movie', type: 'movie' }, { id: 'other', type: 'unsupported' }]), 'https://example.test'));
assert.equal(catalogs.length, 1);
assert.equal(catalogs[0].type, 'movie');
assert.throws(() => core.normalize('playback', JSON.stringify({id: 's', url:'https://evil.test/media/s'}), 'https://example.test'));
console.log(`WASM: ${vectors.length} shared native/WASM startup vectors and domain boundary checks passed`);
