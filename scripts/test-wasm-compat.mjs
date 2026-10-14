import assert from 'assert';
import {readFileSync} from 'fs';
import {fileURLToPath} from 'url';
const root=process.argv[2] || fileURLToPath(new URL('../',import.meta.url));
const source=readFileSync(root+'/generated/wasm/viptv_core.js');
const core=await import('data:text/javascript;base64,'+source.toString('base64'));
await core.default(readFileSync(root+'/generated/wasm/viptv_core_bg.wasm'));
const vectors=JSON.parse(readFileSync(root+'/tests/playback-protocol-vectors.json'));
for(const vector of vectors){
 let actual;
 try{actual={ok:true,result:JSON.parse(core.normalize(vector.kind,vector.input,'https://fixture.invalid'))};}
 catch(error){actual={ok:false,error:String(error)};}
 assert.deepStrictEqual(actual,'output'in vector?{ok:true,result:vector.output}:{ok:false,error:vector.error},vector.name);
}
const startup=JSON.parse(readFileSync(root+'/tests/bridge-vectors.json'));
for(const scenario of startup){
 const app=new core.CoreBridge();let requests=[];
 for(const step of scenario.steps){
  if('event'in step)requests=JSON.parse(app.update(JSON.stringify(step.event)));
  else{const request=requests.find(r=>step.effect in r.effect);assert(request);requests=JSON.parse(app.resolve(request.id,JSON.stringify(step.output)));}
  const view=JSON.parse(app.view());assert.strictEqual(view.phase,step.phase);
  if('profileId'in step)assert.strictEqual(view.selectedProfileId,step.profileId);
 }
 app.free();
}
console.log(JSON.stringify({node:process.version,v8:process.versions.v8,playback_vectors:vectors.length,startup_vectors:startup.length,actual_module_instantiated:true,reference_types_required:false}));
