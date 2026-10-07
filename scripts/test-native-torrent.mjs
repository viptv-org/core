// Execute original-text native contracts through actual browser WASM and matching native Rust.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const root=resolve(import.meta.dirname,'..');
const source=readFileSync(resolve(root,'generated/wasm/viptv_core.js'));
const core=await import(`data:text/javascript;base64,${source.toString('base64')}#native-torrent`);
await core.default(readFileSync(resolve(root,'generated/wasm/viptv_core_bg.wasm')));
const corpus=JSON.parse(readFileSync(resolve(root,'tests/native-torrent-vectors.json'),'utf8'));
const secrets=[corpus.infoHash,'magnet:?','metainfo_base64','private.invalid','private_value','grant_fixture'];
const outcome=fn=>{try{return {ok:true,result:fn()};}catch(error){return {ok:false,error:String(error)};}};
function step(bridge,v){
  const clock=JSON.stringify(v.clock??corpus.clock);
  switch(v.action){
    case 'acceptBytes':return JSON.parse(bridge.acceptBytes(200,new Uint8Array(v.bytes),JSON.stringify(corpus.observation)));
    case 'accept':{
      let body=v.body;
      if(v.metainfoBytes!==undefined){
        const suffix=Buffer.concat([Buffer.from('12:piece lengthi16384e6:pieces20:'),Buffer.alloc(20),Buffer.from('e')]);
        let name=v.metainfoBytes;
        for(;;){const next=v.metainfoBytes-(7+Buffer.byteLength('d6:lengthi4096e4:name')+String(name).length+1+suffix.length+1);if(next===name)break;name=next;}
        const info=Buffer.concat([Buffer.from(`d6:lengthi4096e4:name${name}:`),Buffer.alloc(name,0x61),suffix]);
        const bytes=Buffer.concat([Buffer.from('d4:info'),info,Buffer.from('e')]);
        assert.equal(bytes.length,v.metainfoBytes);
        body=body.replace('__META_BYTES__',bytes.toString('base64')).replace('__META_HASH__',createHash('sha1').update(info).digest('hex'));
      }
      if(v.padBodyBytes!==undefined)body+=' '.repeat(Math.max(0,v.padBodyBytes-Buffer.byteLength(body)));
      return JSON.parse(bridge.accept(v.status??200,body,JSON.stringify(v.observation??corpus.observation)));
    }
    case 'authorize':return JSON.parse(bridge.authorize(clock));
    case 'metadata':return bridge.metadataMatches(JSON.stringify(v.facts),clock);
    case 'metadataTyped':return bridge.metadataMatchesNative(v.facts.infoHash,v.facts.fileIndex,v.facts.fileCount,BigInt(v.facts.selectedFileSize),v.facts.validatedV1Metadata,clock);
    case 'private':return bridge.privateInfoHash(clock)===(v.infoHash??corpus.infoHash)
      && bridge.privateInputKind(clock)===(v.inputKind??corpus.inputKind)
      && bridge.privateInputValue(clock)===(v.inputValue??corpus.inputValue)
      && bridge.privateFileIndex(clock)===corpus.fileIndex
      && Number(bridge.privateExpectedFileSize(clock))===corpus.expectedFileSize;
    case 'debug':{
      for(const secret of secrets)assert.ok(!JSON.stringify(bridge).includes(secret));
      return bridge.toString();
    }
    case 'invalidate':bridge.invalidate();return null;
    case 'normalize':return JSON.parse(core.normalize(v.kind,v.input,'https://fixture.invalid'));
    default:throw new Error('Unknown fixture action');
  }
}
const wasm=corpus.cases.map(v=>{
  let bridge;
  const created=outcome(()=>{bridge=new core.NativeTorrentBridge(JSON.stringify(v.context??corpus.context));return null;});
  const results=created.ok?v.steps.map(action=>outcome(()=>step(bridge,action))):[created];
  assert.equal(results.length,v.expected.length,v.name);
  results.forEach((got,i)=>{
    const want=v.expected[i];
    assert.equal(got.ok,want.ok,v.name);
    if('error'in want)assert.equal(got.error,want.error,v.name);
    if('status'in want)assert.equal(got.result.status,want.status,v.name);
    if('deadline'in want)assert.equal(got.result.deadlineMillis,want.deadline,v.name);
    if('value'in want)assert.deepEqual(got.result,want.value,v.name);
    for(const secret of secrets)assert.ok(!JSON.stringify(got).includes(secret),v.name);
  });
  bridge?.free();
  return {name:v.name,results};
});
const native=spawnSync('cargo',['run','--offline','--locked','-p','viptv-core','--example','native_torrent_vectors'],{cwd:root,encoding:'utf8',timeout:120000,maxBuffer:16*1024*1024});
assert.equal(native.status,0,native.stderr);
assert.deepEqual(JSON.parse(native.stdout),wasm);
const kotlin=readFileSync(resolve(root,'generated/native-kotlin/uniffi/viptv_core/viptv_core.kt'),'utf8');
assert.match(kotlin,/override fun toString\(\): String/);
assert.match(kotlin,/uniffi_viptv_core_fn_method_nativetorrentbridge_uniffi_trait_debug/);
for(const path of ['generated/kotlin-wire/Wire.kt','generated/typescript/wire.ts']){
  assert.doesNotMatch(readFileSync(resolve(root,path),'utf8'),/(?:class|type) NativeTorrentGrant/);
}
console.log(`PASS ${wasm.length} native torrent capability/grant/clock/transition/privacy vectors identical across matching native Rust and actual WASM; generated private holder Debug/toString is redacted`);
