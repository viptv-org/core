import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
const root = new URL('../', import.meta.url);
const source = await readFile(new URL('generated/wasm/viptv_core.js',root),'utf8');
const core = await import(`data:text/javascript;base64,${Buffer.from(source).toString('base64')}`);
await core.default(await readFile(new URL('generated/wasm/viptv_core_bg.wasm',root)));
const fixtures = JSON.parse(await readFile(new URL('tests/torrent-runtime-bridge-vectors.json',root),'utf8'));
const outcomes=[];
for (const fixture of fixtures.cases) {
  let bridge;
  try { bridge=new core.TorrentRuntimeBridge(JSON.stringify(fixture.context??fixtures.context)); }
  catch {outcomes.push({constructs:false,steps:[]});continue;}
  const steps=[];
  for (const step of fixture.steps) {
    const clock=JSON.stringify(step.clock??fixtures.clock);
    try {
      let result;
      switch(step.action) {
        case 'accept': case 'measured': {
          const obs=structuredClone(step.observation??fixtures.observation);
          if(step.action==='measured')obs.trustedWallUpperUnixMillis=null;
          const fn=step.action==='measured'?'acceptMeasuredBytes':'acceptBytes';
          result=JSON.parse(bridge[fn](200,new TextEncoder().encode(step.body??JSON.stringify(fixtures.body)),JSON.stringify(obs)));
          break;
        }
        case 'resolve':result=bridge.bindResolution(step.fileIndex,step.archiveIndex??undefined,BigInt(step.length),clock);break;
        case 'authorize': result=JSON.parse(bridge.authorize(clock));break;
        case 'state':result=JSON.parse(bridge.state());break;
        case 'invalidate':bridge.invalidate();result=null;break;
        case 'wall':result=Number(bridge.trustedWallUpperUnixMillis());break;
        case 'debug':result=bridge.toString();break;
        case 'private':result={fileIndex:bridge.privateFileIndex(clock)??null,archiveIndex:bridge.privateArchiveIndex(clock)??null,
          trackers:bridge.privateTrackers(clock),kind:bridge.privateInputKind(clock),value:bridge.privateInputValue(clock)};break;
        default:throw new Error('unknown action');
      }
      steps.push({ok:true,result});
    } catch {steps.push({ok:false});}
  }
  outcomes.push({constructs:true,steps});bridge.free();
}
const native=spawnSync('cargo',['run','--offline','--locked','-p','viptv-core','--example','torrent_runtime_vectors'],{cwd:root,encoding:'utf8',timeout:120000,maxBuffer:4*1024*1024});
assert.equal(native.status,0,native.stderr);
assert.deepEqual(outcomes,JSON.parse(native.stdout));
const protocol='{"version":2,"native_torrent_versions":[2]}';
for(const platform of ['android','android_tv','desktop','web','roku']) {
  const response=JSON.parse(core.normalize('torrentRuntime',JSON.stringify({operation:'negotiation',platform,qualified:true,scopeMatches:true,status:200,authorizationRefused:false,body:protocol}),''));
  assert.equal(response,['android','android_tv','desktop'].includes(platform)?'advertise':'legacy');
}
assert.equal(JSON.parse(core.normalize('torrentRuntime',JSON.stringify({operation:'recovery',facts:{admitted:true,authorityRetired:true,authorizationRefused:false,selectionRefused:false,action:'retry'}}),'')),'nativeRetry');
assert.throws(()=>core.normalize('torrentRuntime','{"operation":"recovery","operation":"recovery","facts":{}}',''));
console.log(`PASS: ${fixtures.cases.length} private runtime authority cases in native Rust and actual WASM; v2 retry stays on-device`);

for(const [stage,expected] of Object.entries({finding_peers:'Finding peers…',fetching_metadata:'Fetching metadata…',opening_archive:'Opening archive…',buffering:'Buffering…'})) {
  assert.equal(JSON.parse(core.normalize('torrentRuntime',JSON.stringify({operation:'stage',stage}),'')),expected);
}
for(const [stage,code] of Object.entries({finding_peers:'native_no_peers',fetching_metadata:'native_metadata_timeout',opening_archive:'native_archive_timeout',buffering:'native_buffering_timeout'})) {
  assert.equal(JSON.parse(core.normalize('torrentRuntime',JSON.stringify({operation:'failure',stage,reason:'startup_stalled'}),'' )).code,code);
}
assert.throws(()=>core.normalize('torrentRuntime','{"operation":"failure","stage":"buffering","reason":"https://private.invalid"}',''));

assert.equal(JSON.parse(core.normalize('torrentRuntime',JSON.stringify({operation:'negotiation',platform:'android',qualified:true,scopeMatches:true,status:200,authorizationRefused:false,body:'{"version":2,"native_torrent_versions":[]}'}),'')),'legacy');
