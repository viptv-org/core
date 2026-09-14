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
const domain = (kind, value) => JSON.parse(core.normalize(kind, JSON.stringify(value), 'https://example.test'));
const portrait = {id:'m',type:'movie',name:'Movie',poster:'portrait.jpg',position:20,duration:100};
assert.equal(domain('presentation',portrait).heroImage,null);
assert.equal(domain('presentation',{...portrait,background:'landscape.jpg'}).heroImage,'landscape.jpg');
assert.equal(domain('presentation',portrait).progress,0.2);
assert.equal(domain('enrichHome',{original:portrait,metadata:{position:0,background:'landscape.jpg'}}).position,20);
assert.equal(domain('androidPreferences',{}).autoplay,true);
assert.equal(domain('guide',{items:[{start_time:10,end_time:20,display_time:'Now'}]}).programs[0].start,10);
assert.equal(domain('live',{items:[{id:'c',name:'Channel',logo:'logo.png'}]}).channels[0].poster,'logo.png');
assert.equal(domain('request',{operation:'playback',profileId:'must-not-send',playback:{streamId:'opaque',capabilities:{directPlay:true}}}).body.profile_id,undefined);
console.log('WASM: presentation, enrichment, Android compatibility and playback request policy passed');
for (const [item,label] of [[{type:'live'},'Watch live'],[{type:'series',episode:2,queueStatus:'next'},'Play next episode'],[{type:'movie',position:2},'Resume'],[{type:'series'},'Episodes'],[{type:'movie'},'Play']]) {
  assert.equal(domain('presentation',item).primaryActionLabel,label);
}
console.log('WASM: Home hero action labels match the shared contract');
{
  const app = new core.CoreBridge();
  const find = (requests, kind) => requests.find(r => kind in r.effect);
  const resolve = (r, output) => JSON.parse(app.resolve(r.id, JSON.stringify(output)));
  const http = (r, body) => resolve(r,{Ok:{status:200,headers:[],body:[...new TextEncoder().encode(JSON.stringify(body))]}});
  const session = {sessionId:'s',accountId:'a',profileId:'p',accessToken:'fake',refreshToken:'fake',expiresIn:3600};
  const identity = {account:{id:'a',username:'viewer',name:'Viewer',role:'member'},profiles:[{id:'p',name:'Main',setup_complete:true}],profile_id:null,restricted:false,profile_setup_required:false};
  const load=find(JSON.parse(app.update(JSON.stringify({Begin:{origin:'https://example.test',allowInsecurePreview:false}}))),'Storage');
  const me=find(resolve(load,{Ok:JSON.stringify(session)}),'Http');
  const select=find(http(me,identity),'Http');
  const save=find(http(select,{}),'Storage');
  const refresh=find(resolve(save,{Ok:null}),'Http');
  const effects=http(refresh,identity);
  assert.equal(JSON.parse(app.view()).phase,'Error');
  assert.equal(effects.some(r=>'Http' in r.effect || 'Storage' in r.effect),false);
  app.free();
}
console.log('WASM: inconsistent post-selection identity stops without another mutation');
{
 const source=domain('source',{id:'source',source_name:'Evening News',title:'HD broadcast',filename:'evening-news.mkv',source_fingerprint:'stable'});
 const display=domain('sourceDisplay',source);
 assert.equal(source.name,'HD broadcast');
 assert.equal(display.title,'Evening News');
 assert.equal(display.body,'HD broadcast\nevening-news.mkv');
}
console.log('WASM: source projection preserves normalized identity and native labels');
const logoSeries = domain('media',{id:'series',type:'series',name:'Series',logo:'series.png',videos:[{id:'series:1:2',name:'Episode two',season:1,episode:2,position:42,duration:100}]});
const logoEpisode = logoSeries.episodes[0];
assert.equal(logoEpisode.id,'series:1:2');
assert.equal(logoEpisode.seriesId,'series');
assert.equal(domain('presentation',logoEpisode).title,'Series');
assert.equal(domain('presentation',logoEpisode).titleLogo,'series.png');
assert.equal(domain('presentation',logoEpisode).progress,0.42);
assert.equal(domain('presentation',domain('media',{id:'channel',type:'live',name:'Channel',logo:'station.png'})).titleLogo,null);
assert.equal(domain('presentation',portrait).titleLogo,null);
assert.equal(domain('enrichHome',{original:logoEpisode,metadata:{titleLogo:'new.png',position:0}}).position,42);
console.log('WASM: title logos retain text, episode identity and channel-logo distinction');

const failedStillItem = {id:'series:1:3',type:'series',name:'Series',season:1,episode:3,thumbnail:'missing.jpg',background:'landscape.jpg',poster:'portrait.jpg',position:42,duration:100};
const failedStillCard = domain('cardPresentation',{item:failedStillItem,context:'queue',failedImages:['missing.jpg']});
assert.equal(failedStillCard.image,'landscape.jpg');
assert.equal(failedStillCard.imageRole,'landscape');
assert.equal(failedStillCard.progress,.42);
assert.equal(failedStillCard.primaryAction,'resume');
const failedLandscapeCard = domain('cardPresentation',{item:failedStillItem,context:'queue',failedImages:['missing.jpg','landscape.jpg']});
assert.equal(failedLandscapeCard.image,null);
assert.equal(failedLandscapeCard.imageRole,'none');
console.log('WASM: failed episode artwork falls back through shared landscape policy');
