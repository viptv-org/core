// Preserve raw JSON tokens/duplicate fields when comparing native and real WASM.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
const root = resolve(import.meta.dirname, '..');
const source = readFileSync(resolve(root, 'generated/wasm/viptv_core.js'));
const core = await import(`data:text/javascript;base64,${source.toString('base64')}#playback-protocol`);
await core.default(readFileSync(resolve(root, 'generated/wasm/viptv_core_bg.wasm')));
const vectors = JSON.parse(readFileSync(resolve(root, 'tests/playback-protocol-vectors.json'), 'utf8'));
const expected = vectors.map(v => 'output' in v ? {ok:true,result:v.output} : {ok:false,error:v.error});
const wasm = vectors.map(v => {
  try { return {ok:true,result:JSON.parse(core.normalize(v.kind, v.input, 'https://fixture.invalid'))}; }
  catch (error) { return {ok:false,error:String(error)}; }
});
assert.deepEqual(wasm, expected);
// Invalid JSON must reach normalize as text, rather than being rejected by the shell.
// A string-input test runner accepts every vector, including truncated JSON.
const native = spawnSync('cargo', ['run','--offline','--locked','-p','viptv-core','--example','playback_protocol_vectors'], {cwd:root,encoding:'utf8',timeout:120000,maxBuffer:8*1024*1024});
assert.equal(native.status, 0, native.stderr);
assert.deepEqual(JSON.parse(native.stdout), expected);
assert.deepEqual(JSON.parse(native.stdout), wasm);
console.log(`PASS ${vectors.length} playback protocol/request raw-text vectors identical across native Rust and actual WASM`);
