use super::*;
use viptv_engine::{
    capabilities::{Availability, Capability},
    live::*,
    profile::IntegrationConnection,
};
#[derive(Clone)]
pub(super) struct Cache {
    revision: u64,
    expires: std::time::Instant,
    channels: Vec<PrivateChannel>,
    groups: Vec<LiveGroup>,
    guides: std::collections::HashMap<String, (std::time::Instant, LiveGuide)>,
}
#[uniffi::export]
impl LocalEngine {
    pub fn live_channels(&self, profile: String, query_json: String) -> Result<String, CoreError> {
        let query: LiveQuery =
            serde_json::from_str(&query_json).map_err(|_| CoreError::InvalidInput)?;
        query.validate().map_err(|_| CoreError::InvalidInput)?;
        #[cfg(not(feature = "live-runtime"))]
        {
            let _ = profile;
            return serde_json::to_string(&LivePage {
                channels: vec![],
                groups: vec![],
                connections: vec![],
                next_cursor: None,
                complete: true,
                availability: Availability::Unsupported,
                reason: Some("Live runtime unavailable".into()),
            })
            .map_err(|_| CoreError::Bridge);
        }
        #[cfg(feature = "live-runtime")]
        {
            let (scope, connections) = self.live_connections(&profile)?;
            let selected = connections
                .iter()
                .find(|c| query.connection_id.as_ref().map_or(true, |id| id == &c.id));
            let Some(connection) = selected else {
                return serde_json::to_string(&LivePage {
                    channels: vec![],
                    groups: vec![],
                    connections,
                    next_cursor: None,
                    complete: true,
                    availability: Availability::NeedsConfiguration,
                    reason: Some("Assign a live TV connection".into()),
                })
                .map_err(|_| CoreError::Bridge);
            };
            let cache = self.live_directory(&scope, connection)?;
            let filtered = cache
                .channels
                .iter()
                .filter(|c| {
                    query
                        .group_id
                        .as_ref()
                        .is_none_or(|g| c.channel.group_id.as_ref() == Some(g))
                        && query.search.as_ref().is_none_or(|search| {
                            c.channel
                                .card
                                .title
                                .to_lowercase()
                                .contains(&search.to_lowercase())
                        })
                })
                .collect::<Vec<_>>();
            let limit = query.limit.unwrap_or(100) as usize;
            let end = (query.offset as usize + limit).min(filtered.len());
            let page = LivePage {
                channels: filtered
                    .iter()
                    .skip(query.offset as usize)
                    .take(limit)
                    .map(|c| c.channel.clone())
                    .collect(),
                groups: cache.groups,
                connections,
                next_cursor: (end < filtered.len()).then(|| end.to_string()),
                complete: end >= filtered.len(),
                availability: Availability::Available,
                reason: None,
            };
            serde_json::to_string(&page).map_err(|_| CoreError::Bridge)
        }
    }
    pub fn live_guide(&self, profile: String, channel_id: String) -> Result<String, CoreError> {
        #[cfg(not(feature = "live-runtime"))]
        {
            let _ = profile;
            return serde_json::to_string(&LiveGuide {
                channel_id,
                programs: vec![],
                availability: Availability::Unsupported,
                reason: Some("Live runtime unavailable".into()),
            })
            .map_err(|_| CoreError::Bridge);
        }
        #[cfg(feature = "live-runtime")]
        {
            let (scope, connection, channel, cache) = self.live_channel(&profile, &channel_id)?;
            if let Some((expiry, guide)) = cache
                .guides
                .get(&channel_id)
                .filter(|(expiry, _)| std::time::Instant::now() < *expiry)
            {
                let _ = expiry;
                return serde_json::to_string(guide).map_err(|_| CoreError::Bridge);
            }
            let _lock = self.live_fetch_lock.lock().map_err(|_| CoreError::Bridge)?;
            let value = self
                .secrets
                .get(self.source_key(&profile, &connection.id))?
                .ok_or(CoreError::Bridge)?;
            if let Some((expiry, guide)) = self
                .live_cache
                .lock()
                .map_err(|_| CoreError::Bridge)?
                .get(&connection.id)
                .and_then(|cache| cache.guides.get(&channel_id))
                .filter(|(expiry, _)| std::time::Instant::now() < *expiry)
            {
                let _ = expiry;
                return serde_json::to_string(guide).map_err(|_| CoreError::Bridge);
            }
            let mut guide = if connection.plugin_id == "m3u" {
                let config: M3uConfiguration =
                    serde_json::from_str(&value).map_err(|_| CoreError::Bridge)?;
                if let Some((url, epg)) =
                    config.epg_url.filter(|s| !s.is_empty()).zip(channel.epg_id)
                {
                    let response = self.live_http(&scope, &url)?;
                    LiveGuide {
                        channel_id: channel_id.clone(),
                        programs: parse_xmltv(&response.1, &epg, &channel_id)
                            .map_err(|_| CoreError::Bridge)?,
                        availability: Availability::Available,
                        reason: None,
                    }
                } else {
                    LiveGuide {
                        channel_id: channel_id.clone(),
                        programs: vec![],
                        availability: Availability::Unsupported,
                        reason: Some("No XMLTV guide configured for this channel".into()),
                    }
                }
            } else {
                let config: XtreamConfiguration =
                    serde_json::from_str(&value).map_err(|_| CoreError::Bridge)?;
                let uri = xtream_request(&config, "get_short_epg", channel.stream_id.as_deref())
                    .map_err(|_| CoreError::Bridge)?;
                let response = self.live_http(&scope, uri.as_str())?;
                LiveGuide {
                    channel_id: channel_id.clone(),
                    programs: parse_xtream_guide(
                        &serde_json::from_str(&response.1).map_err(|_| CoreError::Bridge)?,
                        &channel_id,
                    ),
                    availability: Availability::Available,
                    reason: None,
                }
            };
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            guide.programs.retain(|p| {
                p.end_millis > now.saturating_sub(24 * 60 * 60 * 1000)
                    && p.start_millis < now + 7 * 24 * 60 * 60 * 1000
            });
            guide.programs.truncate(100);
            if let Some(cache) = self
                .live_cache
                .lock()
                .map_err(|_| CoreError::Bridge)?
                .get_mut(&connection.id)
            {
                cache.guides.insert(
                    channel_id,
                    (
                        std::time::Instant::now() + std::time::Duration::from_secs(60),
                        guide.clone(),
                    ),
                );
            }
            serde_json::to_string(&guide).map_err(|_| CoreError::Bridge)
        }
    }
    pub fn live_sources(&self, profile: String, channel_id: String) -> Result<String, CoreError> {
        #[cfg(not(feature = "live-runtime"))]
        {
            let _ = (profile, channel_id);
            Err(CoreError::InvalidInput)
        }
        #[cfg(feature = "live-runtime")]
        {
            let (scope, connection, channel, _) = self.live_channel(&profile, &channel_id)?;
            let mut grants = self.playable.lock().map_err(|_| CoreError::Bridge)?;
            let mut output = Vec::new();
            for source in channel.sources {
                let id = uuid::Uuid::new_v4().to_string();
                let candidate = viptv_engine::stremio::SourceCandidate {
                    id: id.clone(),
                    provider_id: connection.id.clone(),
                    label: channel.channel.card.title.clone(),
                    detail: None,
                    quality_height: None,
                    audio_languages: vec![],
                };
                grants.insert(
                    id,
                    sources::Playable {
                        scope: scope.clone(),
                        candidate: candidate.clone(),
                        kind: "live".into(),
                        media_id: channel_id.clone(),
                        url: source.url,
                        headers: source.headers.into_iter().collect(),
                        job_id: None,
                        expires: Some(
                            std::time::Instant::now() + std::time::Duration::from_secs(300),
                        ),
                    },
                );
                output.push(candidate);
            }
            serde_json::to_string(&output).map_err(|_| CoreError::Bridge)
        }
    }
}
#[cfg(feature = "live-runtime")]
impl LocalEngine {
    fn live_connections(
        &self,
        profile: &str,
    ) -> Result<(Scope, Vec<IntegrationConnection>), CoreError> {
        let store = self.store.lock().map_err(|_| CoreError::Bridge)?;
        let selected = store
            .profile(&local_scope(profile))
            .map_err(|_| CoreError::Bridge)?
            .ok_or(CoreError::InvalidInput)?;
        let connections = store
            .connections("local")
            .map_err(|_| CoreError::Bridge)?
            .into_iter()
            .filter(|c| {
                c.enabled
                    && c.private_profile_id.as_deref() == Some(profile)
                    && selected
                        .assignments
                        .iter()
                        .any(|a| a.connection_id == c.id && a.enabled.contains(&Capability::LiveTv))
            })
            .collect();
        Ok((selected.scope, connections))
    }
    fn live_http(&self, scope: &Scope, url: &str) -> Result<(String, String), CoreError> {
        let store = self.store.clone();
        let expected = scope.clone();
        let guard: Arc<dyn Fn() -> bool + Send + Sync> = Arc::new(move || {
            store
                .lock()
                .ok()
                .and_then(|s| s.profile(&expected).ok().flatten())
                .is_some_and(|p| p.scope.revision == expected.revision)
        });
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| CoreError::Bridge)?;
        let uri = url::Url::parse(url).map_err(|_| CoreError::InvalidInput)?;
        runtime.block_on(async {
            let response = viptv_engine::egress::ScopedHttp::new(guard)
                .request(viptv_engine::egress::HttpRequest::get(uri))
                .await
                .map_err(|_| CoreError::Bridge)?;
            if !(200..300).contains(&response.status) {
                return Err(CoreError::Bridge);
            }
            Ok((
                response.url.to_string(),
                String::from_utf8_lossy(&response.body).into_owned(),
            ))
        })
    }
    fn live_directory(
        &self,
        scope: &Scope,
        connection: &IntegrationConnection,
    ) -> Result<Cache, CoreError> {
        let _lock = self.live_fetch_lock.lock().map_err(|_| CoreError::Bridge)?;
        if let Some(cache) = self
            .live_cache
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .get(&connection.id)
            .filter(|c| c.revision == connection.revision && std::time::Instant::now() < c.expires)
            .cloned()
        {
            return Ok(cache);
        }
        let raw = self
            .secrets
            .get(self.source_key(&scope.profile_id, &connection.id))?
            .ok_or(CoreError::Bridge)?;
        let mut groups = Vec::new();
        let channels = if connection.plugin_id == "m3u" {
            let config: M3uConfiguration =
                serde_json::from_str(&raw).map_err(|_| CoreError::Bridge)?;
            let (url, body) = self.live_http(scope, &config.playlist_url)?;
            let channels = parse_m3u(
                &body,
                &url::Url::parse(&url).map_err(|_| CoreError::Bridge)?,
                &connection.id,
            )
            .map_err(|_| CoreError::Bridge)?;
            let mut seen = std::collections::HashSet::new();
            for channel in &channels {
                if let Some((id, label)) = channel
                    .channel
                    .group_id
                    .clone()
                    .zip(channel.channel.group_name.clone())
                {
                    if seen.insert(id.clone()) {
                        groups.push(LiveGroup { id, label });
                    }
                }
            }
            channels
        } else {
            let config: XtreamConfiguration =
                serde_json::from_str(&raw).map_err(|_| CoreError::Bridge)?;
            let (_, body) = self.live_http(
                scope,
                xtream_request(&config, "get_live_streams", None)
                    .map_err(|_| CoreError::Bridge)?
                    .as_str(),
            )?;
            let mut channels =
                parse_xtream(&body, &connection.id, &config).map_err(|_| CoreError::Bridge)?;
            let (_, body) = self.live_http(
                scope,
                xtream_request(&config, "get_live_categories", None)
                    .map_err(|_| CoreError::Bridge)?
                    .as_str(),
            )?;
            groups = xtream_groups(&body).map_err(|_| CoreError::Bridge)?;
            for channel in &mut channels {
                if let Some(group) = groups
                    .iter()
                    .find(|g| channel.channel.group_id.as_ref() == Some(&g.id))
                {
                    channel.channel.group_name = Some(group.label.clone());
                }
            }
            channels
        };
        let cache = Cache {
            revision: connection.revision,
            expires: std::time::Instant::now() + std::time::Duration::from_secs(300),
            channels,
            groups,
            guides: Default::default(),
        };
        self.live_cache
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .insert(connection.id.clone(), cache.clone());
        Ok(cache)
    }
    fn live_channel(
        &self,
        profile: &str,
        id: &str,
    ) -> Result<(Scope, IntegrationConnection, PrivateChannel, Cache), CoreError> {
        let (scope, connections) = self.live_connections(profile)?;
        let owner = {
            let cache = self.live_cache.lock().map_err(|_| CoreError::Bridge)?;
            connections
                .iter()
                .find(|connection| {
                    cache.get(&connection.id).is_some_and(|entry| {
                        entry
                            .channels
                            .iter()
                            .any(|channel| channel.channel.id == id)
                    })
                })
                .cloned()
        };
        if let Some(connection) = owner {
            let cache = self.live_directory(&scope, &connection)?;
            let channel = cache
                .channels
                .iter()
                .find(|c| c.channel.id == id)
                .cloned()
                .ok_or(CoreError::InvalidInput)?;
            return Ok((scope, connection, channel, cache));
        }
        Err(CoreError::InvalidInput)
    }
}

#[cfg(all(test, feature = "live-runtime"))]
mod tests {
    use super::*;
    struct Secrets(Mutex<std::collections::HashMap<String, String>>);
    impl SecretStore for Secrets {
        fn get(&self, key: String) -> Result<Option<String>, CoreError> {
            Ok(self.0.lock().unwrap().get(&key).cloned())
        }
        fn put(&self, key: String, value: String) -> Result<(), CoreError> {
            self.0.lock().unwrap().insert(key, value);
            Ok(())
        }
        fn remove(&self, key: String) -> Result<(), CoreError> {
            self.0.lock().unwrap().remove(&key);
            Ok(())
        }
    }
    #[test]
    fn live_directory_and_grants_preserve_scope_and_never_export_private_uris() {
        let dir = std::env::temp_dir().join(format!("viptv-native-live-{}", uuid::Uuid::new_v4()));
        let engine = LocalEngine::new(
            dir.join("live.sqlite").to_string_lossy().into(),
            Arc::new(Secrets(Mutex::new(Default::default()))),
        )
        .unwrap();
        engine
            .create_profile("other".into(), "Other".into())
            .unwrap();
        let public=engine.configure_source_integration("default".into(),0,"m3u".into(),serde_json::json!({"playlist_url":"https://subscription.example/list.m3u?token=private"}).to_string()).unwrap();
        assert!(!public.contains("token=private"));
        let connection: IntegrationConnection = serde_json::from_str(&public).unwrap();
        let channels=parse_m3u("#EXTM3U\n#EXTINF:-1 tvg-id=\"news\" group-title=\"News\",News TV\nhttps://media.example/live.m3u8?token=private\n",&url::Url::parse("https://subscription.example/list.m3u").unwrap(),&connection.id).unwrap();
        let channel_id = channels[0].channel.id.clone();
        engine.live_cache.lock().unwrap().insert(
            connection.id.clone(),
            Cache {
                revision: 0,
                expires: std::time::Instant::now() + std::time::Duration::from_secs(300),
                channels,
                groups: vec![],
                guides: Default::default(),
            },
        );
        let page = engine.live_channels("default".into(), "{}".into()).unwrap();
        assert!(!page.contains("token=private"));
        assert!(!page.contains("media.example"));
        let sources: serde_json::Value = serde_json::from_str(
            &engine
                .live_sources("default".into(), channel_id.clone())
                .unwrap(),
        )
        .unwrap();
        let opened: serde_json::Value = serde_json::from_str(
            &engine
                .open_source(sources[0]["id"].as_str().unwrap().into())
                .unwrap(),
        )
        .unwrap();
        assert_eq!(opened["live"], true);
        assert_eq!(opened["position_millis"], 0);
        let guide: serde_json::Value = serde_json::from_str(
            &engine
                .live_guide("default".into(), channel_id.clone())
                .unwrap(),
        )
        .unwrap();
        assert_eq!(guide["availability"], "Unsupported");
        assert!(guide["programs"].as_array().unwrap().is_empty());
        assert!(engine.live_sources("other".into(), channel_id).is_err());
        engine
            .remove_source_integration("default".into(), 1, connection.id)
            .unwrap();
        assert!(
            engine
                .open_source(sources[0]["id"].as_str().unwrap().into())
                .is_err()
        );
        drop(engine);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
#[cfg(all(test,feature="live-runtime"))]
mod public_qualification {
 use super::*;
 struct Memory(Mutex<std::collections::HashMap<String,String>>);
 impl SecretStore for Memory{fn get(&self,k:String)->Result<Option<String>,CoreError>{Ok(self.0.lock().unwrap().get(&k).cloned())}fn put(&self,k:String,v:String)->Result<(),CoreError>{self.0.lock().unwrap().insert(k,v);Ok(())}fn remove(&self,k:String)->Result<(),CoreError>{self.0.lock().unwrap().remove(&k);Ok(())}}
 #[test]
 #[ignore="reads the real published public M3U fixture through scoped HTTPS"]
 fn real_public_m3u_directory_uses_host_fetch_cache_and_live_grants(){
  let dir=std::env::temp_dir().join(format!("viptv-public-live-{}",uuid::Uuid::new_v4()));let engine=LocalEngine::new(dir.join("live.sqlite").to_string_lossy().into(),Arc::new(Memory(Mutex::new(Default::default())))).unwrap();engine.configure_source_integration("default".into(),0,"m3u".into(),serde_json::json!({"playlist_url":"https://iptv-org.github.io/iptv/countries/us.m3u"}).to_string()).unwrap();let start=std::time::Instant::now();let page:LivePage=serde_json::from_str(&engine.live_channels("default".into(),"{\"limit\":20}".into()).unwrap()).unwrap();assert!(!page.channels.is_empty());let first=page.channels[0].id.clone();let warm=std::time::Instant::now();let repeat:LivePage=serde_json::from_str(&engine.live_channels("default".into(),"{\"limit\":20}".into()).unwrap()).unwrap();assert_eq!(repeat.channels[0].id,first);assert!(warm.elapsed().as_millis()<500);let sources:Vec<viptv_engine::stremio::SourceCandidate>=serde_json::from_str(&engine.live_sources("default".into(),first.clone()).unwrap()).unwrap();assert!(!sources.is_empty());let open:serde_json::Value=serde_json::from_str(&engine.open_source(sources[0].id.clone()).unwrap()).unwrap();assert_eq!(open["live"],true);let guide:LiveGuide=serde_json::from_str(&engine.live_guide("default".into(),first).unwrap()).unwrap();assert_eq!(guide.availability,Availability::Unsupported);println!("Public M3U qualification: channels={}, source_grants={}, elapsed_seconds={}, warm_cache_ms={}",page.channels.len(),sources.len(),start.elapsed().as_secs(),warm.elapsed().as_millis());drop(engine);
 }
}
