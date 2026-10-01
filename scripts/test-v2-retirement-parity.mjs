// Compare the frozen JSON wire through baseline/candidate native and actual WASM.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
const candidate = resolve(import.meta.dirname, '..');
const baseline = resolve(process.argv[2] ?? '../../core');
async function wasm(root) {
  const source = readFileSync(resolve(root, 'generated/wasm/viptv_core.js'));
  // Unique fragment prevents Node sharing the two different generated modules.
  const module = await import(`data:text/javascript;base64,${source.toString('base64')}#${encodeURIComponent(root)}`);
  await module.default(readFileSync(resolve(root, 'generated/wasm/viptv_core_bg.wasm')));
  return module;
}
const old = await wasm(baseline), current = await wasm(candidate);
const retired = ['addonEndpoint','addonCatalogExtras','addonSupports','discoverPlan','discoverAggregate','providerCandidate','providerSelectCandidates','providerMediaUrl'];
for (const name of retired) { assert.equal(typeof old[name], 'function'); assert.equal(current[name], undefined, name); }
const nativeBindings = readFileSync(resolve(candidate,'generated/native-kotlin/uniffi/viptv_core/viptv_core.kt'),'utf8');
for (const name of retired) assert(!nativeBindings.includes(name), `retired native binding ${name}`);
for (const path of ['generated/typescript/wire.ts','generated/kotlin-wire/Wire.kt','generated/typescript/viptv_core_types.ts','generated/kotlin/org/viptv/core/types/Types.kt']) {
  assert.equal(readFileSync(resolve(candidate,path),'utf8'),readFileSync(resolve(baseline,path),'utf8'),path);
}
const requests = [];
const expected = [];
const add = (request, output) => { requests.push(request); expected.push({ok:true,result:output}); };
const normalize = (kind,input) => {
  const origin = 'https://fixture.invalid';
  let a;
  try { a = JSON.parse(old.normalize(kind,JSON.stringify(input),origin)); }
  catch { throw new Error(`Invalid parity fixture: ${kind}/${input.operation ?? ''}`); }
  assert.deepEqual(JSON.parse(current.normalize(kind,JSON.stringify(input),origin)),a,kind);
  add({op:'normalize',kind,input,origin},a);
  return a;
};
normalize('liveCatalogV2',{catalog_id:2,generation:0,items:[{id:'iptv:2:7',name:'News',logo:'http://fixture.invalid/news.png'}],next_cursor:'next',previous_cursor:'previous'});
normalize('liveCategoriesV2',{catalog_id:2,generation:0,items:[{id:'7',name:'News'}],next_cursor:'next',previous_cursor:'previous'});
normalize('liveSourceV2',{source:{id:'opaque_live',source:'iptv:2',source_addon_id:'iptv:2',name:'Provider',title:'News'}});
for (const operation of ['livePageV2','liveCategoriesV2']) normalize('request',{operation,catalogId:'2',cursor:'previous',limit:40});
normalize('request',{operation:'livePageV2',catalogId:'2',categoryId:'7',collection:'favorites',cursor:'previous',limit:40});
normalize('request',{operation:'liveSourceV2',id:'iptv:2:7'});
normalize('request',{operation:'sourcesV2',item:{id:'tt1234567:1:2',type:'series',season:1,episode:2}});
const intent = normalize('playbackV2Intent',{requestId:'stable_request',platform:'tauri',playback:{streamId:'opaque_source',position:12,capabilities:{maxWidth:3840,maxHeight:2160,h264:true,aac:true,directUrls:true},forceTranscode:true,conversionReason:'audio-codec'}});
normalize('request',{operation:'playbackV2',playback:intent});
for (const operation of ['playbackV2Heartbeat','playbackV2Stop']) normalize('request',{operation,id:'pb2_fixture'});
normalize('playbackV2',{id:'pb2_fixture',status:'ready',expires_at:1800000060,renew_after_seconds:20,delivery:{kind:'direct',url:'http://fixture.invalid/movie.mp4',headers:{'User-Agent':'Fixture'},position:12,live:false,format:'original'}});
normalize('sourcesPollStep',{poll:{done:true,events:[{seq:1,source:'iptv:2',streams:[],error_code:'provider_connection_limit',error:'https://fixture.invalid/private-token'}]}});
const vectors = JSON.parse(readFileSync(resolve(candidate,'tests/bridge-vectors.json'),'utf8'));
for (const scenario of vectors) {
  const a = new old.CoreBridge(), b = new current.CoreBridge();
  let effects = [];
  add({op:'reset'},JSON.parse(a.view()));
  assert.deepEqual(JSON.parse(b.view()),JSON.parse(a.view()));
  for (const step of scenario.steps) {
    const request = 'event' in step ? {op:'update',event:step.event} : {op:'resolve',id:effects.find(effect => step.effect in effect.effect).id,result:step.output};
    const invoke = app => request.op === 'update' ? app.update(JSON.stringify(request.event)) : app.resolve(request.id,JSON.stringify(request.result));
    effects = JSON.parse(invoke(a));
    assert.deepEqual(JSON.parse(invoke(b)),effects,scenario.name);
    add(request,effects); add({op:'view'},JSON.parse(a.view()));
    assert.deepEqual(JSON.parse(b.view()),JSON.parse(a.view()),scenario.name);
  }
  a.free(); b.free();
}
for (const root of [baseline,candidate]) {
  const result = spawnSync('cargo',['run','--offline','--locked','-p','viptv-core','--example','wasi'],{cwd:root,input:requests.map(request=>JSON.stringify(request)).join('\n')+'\n',encoding:'utf8',maxBuffer:16*1024*1024});
  assert.equal(result.status,0,result.stderr);
  assert.deepEqual(result.stdout.trim().split('\n').map(line=>JSON.parse(line)),expected,root);
}
console.log(`PASS ${requests.length} frozen v2/startup operations identical across baseline/candidate native and actual WASM; protocol declarations unchanged; eight retired exports absent`);
