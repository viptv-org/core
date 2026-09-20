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
// Addon-defined catalog namespaces are preserved verbatim; only unsupported
// media rows inside a catalog response are filtered.
assert.deepEqual(catalogs.map(catalog => catalog.type), ['movie', 'unsupported']);
// A playback URL never carries embedded credentials, and a root-relative URL
// must stay a same-origin /media/ capability. Absolute http(s) URLs are the
// ORIGINAL source URLs direct-URL clients play natively.
assert.throws(() => core.normalize('playback', JSON.stringify({id: 's', url: '/api/elsewhere'}), 'https://example.test'));
assert.throws(() => core.normalize('playback', JSON.stringify({id: 's', url: 'https://user:pass@evil.test/media/s'}), 'https://example.test'));
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
// directUrls rides the same generic snake_case wire path as directFiles.
assert.equal(domain('request',{operation:'playback',playback:{streamId:'opaque',capabilities:{directPlay:true,directUrls:true}}}).body.capabilities.direct_urls,true);
const direct = domain('playback',{id:'s',url:'https://provider.example/stream.mkv',mode:'direct',authorization:{cookie:'session=1',user_agent:'VIPTV Desktop'}});
assert.equal(direct.url,'https://provider.example/stream.mkv');
assert.deepEqual(direct.authorization,{cookie:'session=1',userAgent:'VIPTV Desktop'});
console.log('WASM: presentation, enrichment, Android compatibility and playback request policy passed');
// Every remote http(s) image routes through the shared wsrv cache so one
// resize pipeline serves all artwork; data:, relative sources, and wsrv
// re-wrapping are the passthrough/unwrap behaviors the UI relies on.
const artwork = (original, extra = {}) => domain('artworkUrl', { original, width: 256, height: 144, ...extra });
assert.ok(artwork('https://image.tmdb.org/t/p/w500/abc.jpg').startsWith('https://wsrv.nl/?url=https%3A%2F%2Fimage.tmdb.org%2Ft%2Fp%2Fw500%2Fabc.jpg&w=256&h=144&fit=cover&output=jpg&q=85&we'));
assert.ok(artwork('https://image.tmdb.org/t/p/original/bg.jpg', { width: 1280, height: 720, large: true }).startsWith('https://wsrv.nl/?url=https%3A%2F%2Fimage.tmdb.org%2Ft%2Fp%2Fw1280%2Fbg.jpg&w=1280&h=720&fit=cover&output=jpg&q=95&we'));
assert.ok(artwork('https://cdn.some-addon.example/poster.jpg').startsWith('https://wsrv.nl/?url=https%3A%2F%2Fcdn.some-addon.example%2Fposter.jpg&w=256'));
assert.ok(artwork('http://mixed-content.example/img.jpg').startsWith('https://wsrv.nl/?url=http%3A%2F%2Fmixed-content.example%2Fimg.jpg'));
assert.ok(artwork('https://example.test/logo.png', { logo: true }).includes('&fit=inside&output=png'));
assert.ok(artwork('https://wsrv.nl/?url=https%3A%2F%2Fcdn.some-addon.example%2Fposter.jpg').startsWith('https://wsrv.nl/?url=https%3A%2F%2Fcdn.some-addon.example%2Fposter.jpg&w='));
assert.equal(artwork('data:image/svg+xml;base64,AAA'), 'data:image/svg+xml;base64,AAA');
assert.equal(artwork('relative/poster.jpg'), 'relative/poster.jpg');
assert.equal(domain('artworkUrl', { original: '', width: 256, height: 144 }), null);
console.log('WASM: all remote artwork routes through the wsrv image proxy');
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
