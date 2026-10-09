import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
const root = new URL('../', import.meta.url);
const source = await readFile(new URL('generated/wasm/viptv_core.js', root), 'utf8');
const core = await import(`data:text/javascript;base64,${Buffer.from(source).toString('base64')}`);
await core.default(await readFile(new URL('generated/wasm/viptv_core_bg.wasm', root)));
const v1 = '{"version":1,"native_torrent_versions":[1]}';
const v2 = '{"version":2,"native_torrent_versions":[2]}';
assert.deepEqual(JSON.parse(core.normalize('torrentRuntimeProtocol', v2, '')), {version: 2, nativeTorrentVersions: [2]});
assert.throws(() => core.normalize('playbackProtocolV2', v2, ''));
assert.throws(() => core.normalize('torrentRuntimeProtocol', v1, ''));
for (const body of [
  '{"version":2.0,"native_torrent_versions":[2]}',
  '{"version":2,"native_torrent_versions":[2.0]}',
  '{"version":2,"version":2,"native_torrent_versions":[2]}',
  '{"version":2,"native_torrent_versions":[1,2]}',
]) assert.throws(() => core.normalize('torrentRuntimeProtocol', body, ''));
const request = JSON.parse(core.normalize('request', '{"operation":"torrentRuntimeProtocol"}', ''));
assert.equal(request.method, 'GET');
assert.equal(request.path, '/api/v2/torrent-runtime-protocol');
assert.equal(request.body, null);
assert.throws(() => core.normalize('request', '{"operation":"torrentRuntimeProtocol","body":{}}', ''));
console.log('PASS: actual-WASM runtime v2 negotiation, closed input and v1 isolation');
