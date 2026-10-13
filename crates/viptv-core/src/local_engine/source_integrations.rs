use super::*;
use serde_json::json;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
use viptv_engine::{
    capabilities::Capability,
    profile::IntegrationConnection,
    source_integration::{VsourcesConfiguration, vsources_schema},
};
pub(super) struct Job {
    scope: Scope,
    created: Instant,
    cancel: AtomicBool,
    state: Mutex<JobState>,
}
struct JobState {
    events: Vec<serde_json::Value>,
    done: bool,
    error: Option<String>,
}
impl LocalEngine {
    pub(super) fn source_key(&self, profile: &str, id: &str) -> String {
        format!("{}:source:{profile}:{id}", self.namespace)
    }
}
#[uniffi::export]
impl LocalEngine {
    pub fn source_integration_schemas(&self, profile: String) -> Result<String, CoreError> {
        let store = self.store.lock().map_err(|_| CoreError::Bridge)?;
        let selected = store
            .profile(&local_scope(&profile))
            .map_err(|_| CoreError::Bridge)?
            .ok_or(CoreError::InvalidInput)?;
        let connection = store
            .connections("local")
            .map_err(|_| CoreError::Bridge)?
            .into_iter()
            .find(|c| {
                c.plugin_id == "vsources" && c.private_profile_id.as_deref() == Some(&profile)
            });
        let configured = connection.as_ref().is_some_and(|c| {
            self.secrets
                .get(self.source_key(&profile, &c.id))
                .ok()
                .flatten()
                .is_some()
        });
        let connections = store
            .connections("local")
            .map_err(|_| CoreError::Bridge)?
            .into_iter()
            .filter(|c| c.private_profile_id.as_deref() == Some(&profile))
            .collect::<Vec<_>>();
        let mut schemas = vec![vsources_schema(
            configured,
            connection.as_ref().map(|c| c.id.clone()),
        )];
        for plugin in ["xtream", "m3u"] {
            let connection = connections.iter().find(|c| c.plugin_id == plugin);
            schemas.push(viptv_engine::source_integration::live_schema(
                plugin,
                connection.is_some(),
                connection.map(|c| c.id.clone()),
                cfg!(feature = "live-runtime"),
            ));
        }
        serde_json::to_string(&json!({"schemas":schemas,"revision":selected.scope.revision,"connections":connections})).map_err(|_|CoreError::Bridge)
    }
    pub fn configure_source_integration(
        &self,
        profile: String,
        revision: u64,
        plugin_id: String,
        configuration_json: String,
    ) -> Result<String, CoreError> {
        if configuration_json.len() > 32768 {
            return Err(CoreError::InvalidInput);
        }
        let value: serde_json::Value =
            serde_json::from_str(&configuration_json).map_err(|_| CoreError::InvalidInput)?;
        match plugin_id.as_str() {
            "vsources" => {
                if !cfg!(feature = "vsources-runtime") {
                    return Err(CoreError::InvalidInput);
                }
                let config: VsourcesConfiguration =
                    serde_json::from_value(value.clone()).map_err(|_| CoreError::InvalidInput)?;
                config.validate().map_err(|_| CoreError::InvalidInput)?;
                #[cfg(feature = "vsources-runtime")]
                viptv_engine::vsources_runtime::engine(
                    &config,
                    Arc::new(viptv_engine::vsources_runtime::ScopedFetcher::new(
                        Arc::new(|| false),
                    )),
                )
                .map_err(|_| CoreError::InvalidInput)?;
            }
            "m3u" => {
                if !cfg!(feature = "live-runtime") {
                    return Err(CoreError::InvalidInput);
                }
                serde_json::from_value::<viptv_engine::live::M3uConfiguration>(value.clone())
                    .map_err(|_| CoreError::InvalidInput)?
                    .validate()
                    .map_err(|_| CoreError::InvalidInput)?;
            }
            "xtream" => {
                if !cfg!(feature = "live-runtime") {
                    return Err(CoreError::InvalidInput);
                }
                serde_json::from_value::<viptv_engine::live::XtreamConfiguration>(value.clone())
                    .map_err(|_| CoreError::InvalidInput)?
                    .validate()
                    .map_err(|_| CoreError::InvalidInput)?;
            }
            _ => return Err(CoreError::InvalidInput),
        }
        let mut store = self.store.lock().map_err(|_| CoreError::Bridge)?;
        let selected = store
            .profile(&local_scope(&profile))
            .map_err(|_| CoreError::Bridge)?
            .ok_or(CoreError::InvalidInput)?;
        if selected.scope.revision != revision {
            return Err(CoreError::InvalidInput);
        }
        let existing = store
            .connections("local")
            .map_err(|_| CoreError::Bridge)?
            .into_iter()
            .find(|c| {
                c.plugin_id == plugin_id && c.private_profile_id.as_deref() == Some(&profile)
            });
        let connection = IntegrationConnection {
            id: existing
                .as_ref()
                .map(|c| c.id.clone())
                .unwrap_or_else(|| format!("{}:{}", plugin_id, uuid::Uuid::new_v4())),
            plugin_id: plugin_id.clone(),
            account_id: "local".into(),
            private_profile_id: Some(profile.clone()),
            name: match plugin_id.as_str() {
                "m3u" => "M3U live TV",
                "xtream" => "Xtream live TV",
                _ => "VIPTV Sources",
            }
            .into(),
            enabled: true,
            revision: existing
                .as_ref()
                .map(|c| c.revision.checked_add(1).ok_or(CoreError::InvalidInput))
                .transpose()?
                .unwrap_or(0),
            capabilities: if plugin_id == "vsources" {
                vec![Capability::Sources]
            } else {
                vec![Capability::Sources, Capability::LiveTv]
            },
        };
        let key = self.source_key(&profile, &connection.id);
        let old = self.secrets.get(key.clone())?;
        self.secrets.put(
            key.clone(),
            serde_json::to_string(&value).map_err(|_| CoreError::Bridge)?,
        )?;
        let saved = if existing.is_some() {
            store.replace_source_connection(&profile, revision, &connection)
        } else {
            store.install(&profile, &connection)
        };
        if saved.is_err() {
            if let Some(old) = old {
                let _ = self.secrets.put(key, old);
            } else {
                let _ = self.secrets.remove(key);
            }
            return Err(CoreError::Bridge);
        }
        serde_json::to_string(&connection).map_err(|_| CoreError::Bridge)
    }
    pub fn remove_source_integration(
        &self,
        profile: String,
        revision: u64,
        connection_id: String,
    ) -> Result<(), CoreError> {
        let mut store = self.store.lock().map_err(|_| CoreError::Bridge)?;
        let selected = store
            .profile(&local_scope(&profile))
            .map_err(|_| CoreError::Bridge)?
            .ok_or(CoreError::InvalidInput)?;
        if selected.scope.revision != revision
            || !store
                .connections("local")
                .map_err(|_| CoreError::Bridge)?
                .iter()
                .any(|c| {
                    c.id == connection_id
                        && ["vsources", "xtream", "m3u"].contains(&c.plugin_id.as_str())
                        && c.private_profile_id.as_deref() == Some(&profile)
                })
        {
            return Err(CoreError::InvalidInput);
        }
        store
            .remove_connection(&connection_id)
            .map_err(|_| CoreError::Bridge)?;
        self.secrets
            .remove(self.source_key(&profile, &connection_id))
    }
    pub fn start_integration_sources(
        &self,
        profile: String,
        card_json: String,
    ) -> Result<String, CoreError> {
        #[cfg(not(feature = "vsources-runtime"))]
        {
            let _ = (profile, card_json);
            Err(CoreError::InvalidInput)
        }
        #[cfg(feature = "vsources-runtime")]
        {
            self.start_sdk(profile, card_json)
        }
    }
    pub fn poll_integration_sources(
        &self,
        profile: String,
        job_id: String,
        after: u64,
    ) -> Result<String, CoreError> {
        let jobs = self
            .integration_jobs
            .lock()
            .map_err(|_| CoreError::Bridge)?;
        let job = jobs.get(&job_id).ok_or(CoreError::InvalidInput)?;
        if job.scope.profile_id != profile
            || job.created.elapsed().as_secs() > 120
            || job.cancel.load(Ordering::Relaxed)
        {
            return Err(CoreError::InvalidInput);
        }
        let current = self
            .store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .profile(&job.scope)
            .map_err(|_| CoreError::Bridge)?
            .ok_or(CoreError::InvalidInput)?;
        if current.scope.revision != job.scope.revision {
            return Err(CoreError::InvalidInput);
        }
        let state = job.state.lock().map_err(|_| CoreError::Bridge)?;
        Ok(json!({"id":job_id,"events":state.events.iter().filter(|e|e["seq"].as_u64().unwrap_or(0)>after).collect::<Vec<_>>(),"done":state.done,"error_code":state.error}).to_string())
    }
    pub fn cancel_integration_sources(
        &self,
        profile: String,
        job_id: String,
    ) -> Result<(), CoreError> {
        let mut jobs = self
            .integration_jobs
            .lock()
            .map_err(|_| CoreError::Bridge)?;
        if jobs
            .get(&job_id)
            .is_none_or(|job| job.scope.profile_id != profile)
        {
            return Err(CoreError::InvalidInput);
        }
        if let Some(job) = jobs.remove(&job_id) {
            job.cancel.store(true, Ordering::Relaxed);
        }
        Ok(())
    }
}
#[cfg(feature = "vsources-runtime")]
impl LocalEngine {
    fn start_sdk(&self, profile: String, card_json: String) -> Result<String, CoreError> {
        if card_json.len() > 64 * 1024 {
            return Err(CoreError::InvalidInput);
        }
        let card: viptv_engine::presentation::Card =
            serde_json::from_str(&card_json).map_err(|_| CoreError::InvalidInput)?;
        card.validate_library()
            .map_err(|_| CoreError::InvalidInput)?;
        let (verified_scope, card) = self
            .verified_cards
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .get(&(
                profile.clone(),
                card.identity.kind.clone(),
                card.identity.id.clone(),
            ))
            .cloned()
            .ok_or(CoreError::InvalidInput)?;
        let media = viptv_engine::vsources_runtime::media_ref(
            &card.identity,
            card.parent_identity.as_ref(),
            &[],
        )
        .map_err(|_| CoreError::InvalidInput)?;
        let store = self.store.lock().map_err(|_| CoreError::Bridge)?;
        let selected = store
            .profile(&local_scope(&profile))
            .map_err(|_| CoreError::Bridge)?
            .ok_or(CoreError::InvalidInput)?;
        if verified_scope.revision != selected.scope.revision {
            return Err(CoreError::InvalidInput);
        }
        let connection = store
            .connections("local")
            .map_err(|_| CoreError::Bridge)?
            .into_iter()
            .find(|c| {
                c.plugin_id == "vsources"
                    && c.enabled
                    && c.private_profile_id.as_deref() == Some(&profile)
                    && selected.assignments.iter().any(|a| {
                        a.connection_id == c.id && a.enabled.contains(&Capability::Sources)
                    })
            })
            .ok_or(CoreError::InvalidInput)?;
        let raw = self
            .secrets
            .get(self.source_key(&profile, &connection.id))?
            .ok_or(CoreError::Bridge)?;
        let config: VsourcesConfiguration =
            serde_json::from_str(&raw).map_err(|_| CoreError::Bridge)?;
        let id = uuid::Uuid::new_v4().to_string();
        let job = Arc::new(Job {
            scope: selected.scope.clone(),
            created: Instant::now(),
            cancel: AtomicBool::new(false),
            state: Mutex::new(JobState {
                events: vec![],
                done: false,
                error: None,
            }),
        });
        let mut jobs = self
            .integration_jobs
            .lock()
            .map_err(|_| CoreError::Bridge)?;
        jobs.retain(|_, j| {
            let keep = j.scope.profile_id != profile && j.created.elapsed().as_secs() < 120;
            if !keep {
                j.cancel.store(true, Ordering::Relaxed);
            }
            keep
        });
        if jobs.len() >= 12 {
            return Err(CoreError::InvalidInput);
        }
        jobs.insert(id.clone(), job.clone());
        drop(jobs);
        drop(store);
        let store = self.store.clone();
        let scope = job.scope.clone();
        let guard_job = job.clone();
        let authorize: Arc<dyn Fn() -> bool + Send + Sync> = Arc::new(move || {
            !guard_job.cancel.load(Ordering::Relaxed)
                && store
                    .lock()
                    .ok()
                    .and_then(|s| s.profile(&scope).ok().flatten())
                    .is_some_and(|p| p.scope.revision == scope.revision)
        });
        let playable = self.playable.clone();
        playable
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .retain(|_, grant| grant.scope.profile_id != profile || grant.job_id.is_none());
        let worker_id = id.clone();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build();
            let result = runtime.map_err(|_| ()).and_then(|rt| {
                rt.block_on(viptv_engine::vsources_runtime::resolve_progressive(
                    config,
                    media,
                    authorize.clone(),
                    |streams| {
                        if !authorize() {
                            return;
                        }
                        let mut grants = playable.lock().unwrap();
                        let sources = streams
                            .into_iter()
                            .map(|stream| {
                                let id = uuid::Uuid::new_v4().to_string();
                                let label = stream
                                    .meta
                                    .source_label
                                    .clone()
                                    .unwrap_or_else(|| "VIPTV Sources".into());
                                let detail = stream.meta.resolution.map(|r| format!("{r}p"));
                                let candidate = viptv_engine::stremio::SourceCandidate {
                                    id: id.clone(),
                                    provider_id: connection.id.clone(),
                                    label,
                                    detail,
                                    quality_height: stream.meta.resolution.map(u32::from),
                                    audio_languages: stream
                                        .meta
                                        .audio_selection
                                        .as_ref()
                                        .map(|a| vec![a.language.as_str().into()])
                                        .unwrap_or_default(),
                                };
                                grants.insert(
                                    id.clone(),
                                    sources::Playable {
                                        scope: job.scope.clone(),
                                        candidate: candidate.clone(),
                                        kind: card.identity.kind.clone(),
                                        media_id: card.identity.id.clone(),
                                        url: stream.url.to_string(),
                                        headers: stream.meta.request_headers.into_iter().collect(),
                                        job_id: Some(worker_id.clone()),
                                        expires: Some(
                                            Instant::now()
                                                + stream
                                                    .ttl
                                                    .min(std::time::Duration::from_secs(1800)),
                                        ),
                                    },
                                );
                                candidate
                            })
                            .collect::<Vec<_>>();
                        let mut state = job.state.lock().unwrap();
                        let seq = state.events.len() + 1;
                        state.events.push(json!({"seq":seq,"sources":sources}));
                    },
                ))
                .map_err(|_| ())
            });
            let mut state = job.state.lock().unwrap();
            state.done = true;
            if result.is_err() {
                state.error = Some("source_unavailable".into());
            }
        });
        Ok(json!({"id":id}).to_string())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Secrets(Mutex<std::collections::HashMap<String, String>>);
    impl SecretStore for Secrets {
        fn get(&self, k: String) -> Result<Option<String>, CoreError> {
            Ok(self.0.lock().unwrap().get(&k).cloned())
        }
        fn put(&self, k: String, v: String) -> Result<(), CoreError> {
            self.0.lock().unwrap().insert(k, v);
            Ok(())
        }
        fn remove(&self, k: String) -> Result<(), CoreError> {
            self.0.lock().unwrap().remove(&k);
            Ok(())
        }
    }
    #[test]
    fn credentials_are_write_only_and_feature_availability_is_honest() {
        let dir =
            std::env::temp_dir().join(format!("viptv-source-config-{}", uuid::Uuid::new_v4()));
        let secrets = Arc::new(Secrets::default());
        let engine = LocalEngine::new(
            dir.join("local.sqlite").to_string_lossy().into(),
            secrets.clone(),
        )
        .unwrap();
        engine
            .create_profile("other".into(), "Other".into())
            .unwrap();
        let schemas = engine.source_integration_schemas("default".into()).unwrap();
        let value: serde_json::Value = serde_json::from_str(&schemas).unwrap();
        assert_eq!(
            value["schemas"][0]["runtime_enabled"],
            cfg!(feature = "vsources-runtime")
        );
        let configured = engine.configure_source_integration(
            "default".into(),
            0,
            "vsources".into(),
            json!({"tmdb_api_key":"fixture-write-only-key"}).to_string(),
        );
        if cfg!(feature = "vsources-runtime") {
            let public = configured.unwrap();
            assert!(!public.contains("fixture-write-only-key"));
            let id: serde_json::Value = serde_json::from_str(&public).unwrap();
            assert!(
                engine
                    .remove_source_integration("other".into(), 0, id["id"].as_str().unwrap().into())
                    .is_err()
            );
            assert!(
                !engine
                    .profiles()
                    .unwrap()
                    .contains("fixture-write-only-key")
            );
            assert_eq!(secrets.0.lock().unwrap().len(), 1);
            let update = engine
                .configure_source_integration(
                    "default".into(),
                    1,
                    "vsources".into(),
                    json!({"tmdb_api_key":"fixture-updated-key"}).to_string(),
                )
                .unwrap();
            assert!(!update.contains("fixture-updated-key"));
            let snapshot: serde_json::Value =
                serde_json::from_str(&engine.source_integration_schemas("default".into()).unwrap())
                    .unwrap();
            assert_eq!(snapshot["revision"], 2);
            assert_eq!(snapshot["schemas"][0]["state"], "Ready");
            assert!(
                snapshot["schemas"][0]["provider_choices"]
                    .as_array()
                    .unwrap()
                    .len()
                    > 20
            );

            engine
                .remove_source_integration("default".into(), 2, id["id"].as_str().unwrap().into())
                .unwrap();
            assert!(secrets.0.lock().unwrap().is_empty());
        } else {
            assert!(configured.is_err());
            assert!(secrets.0.lock().unwrap().is_empty());
        }
        drop(engine);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
#[cfg(all(test, feature = "vsources-runtime"))]
mod real_native_qualification {
    use super::*;
    struct Memory(Mutex<std::collections::HashMap<String, String>>);
    impl SecretStore for Memory {
        fn get(&self, k: String) -> Result<Option<String>, CoreError> {
            Ok(self.0.lock().unwrap().get(&k).cloned())
        }
        fn put(&self, k: String, v: String) -> Result<(), CoreError> {
            self.0.lock().unwrap().insert(k, v);
            Ok(())
        }
        fn remove(&self, k: String) -> Result<(), CoreError> {
            self.0.lock().unwrap().remove(&k);
            Ok(())
        }
    }
    #[test]
    #[ignore = "requires explicitly injected owned TMDB credential and real public provider access"]
    fn native_sdk_job_publishes_opaque_grants_and_early_cancel_preserves_selection() {
        let dir = std::env::temp_dir().join(format!("viptv-native-sdk-{}", uuid::Uuid::new_v4()));
        let engine = LocalEngine::new(
            dir.join("native.sqlite").to_string_lossy().into(),
            Arc::new(Memory(Mutex::new(Default::default()))),
        )
        .unwrap();
        let manifest = r#"{"id":"metadata","name":"Metadata","version":"1","resources":["meta"],"types":["movie"]}"#;
        let addon: serde_json::Value = serde_json::from_str(
            &engine
                .install_addon(
                    "default".into(),
                    "https://v3-cinemeta.strem.io/manifest.json".into(),
                    manifest.into(),
                )
                .unwrap(),
        )
        .unwrap();
        engine
            .configure_source_integration(
                "default".into(),
                1,
                "vsources".into(),
                json!({"tmdb_api_key":std::env::var("VIPTV_QA_TMDB_KEY").unwrap()}).to_string(),
            )
            .unwrap();
        let plan: serde_json::Value = serde_json::from_str(
            &engine
                .plan_detail(
                    "default".into(),
                    "movie".into(),
                    "tt1375666".into(),
                    addon["id"].as_str().unwrap().into(),
                )
                .unwrap(),
        )
        .unwrap();
        let detail: serde_json::Value = serde_json::from_str(
            &engine
                .project_detail(
                    plan["operation"].as_str().unwrap().into(),
                    json!({"meta":{"id":"tt1375666","name":"Inception","type":"movie"}})
                        .to_string(),
                    0,
                )
                .unwrap(),
        )
        .unwrap();
        let mut hint = detail["item"].clone();
        hint["identity"]["ids"] = json!([{"namespace":"tmdb","value":"1"}]);
        let job: serde_json::Value = serde_json::from_str(
            &engine
                .start_integration_sources("default".into(), hint.to_string())
                .unwrap(),
        )
        .unwrap();
        let job_id = job["id"].as_str().unwrap();
        let start = Instant::now();
        let selected = loop {
            let poll = engine
                .poll_integration_sources("default".into(), job_id.into(), 0)
                .unwrap();
            assert!(!poll.contains("request_headers"));
            assert!(!poll.contains("https://"));
            let poll: serde_json::Value = serde_json::from_str(&poll).unwrap();
            if let Some(id) = poll["events"].as_array().and_then(|events| {
                events
                    .iter()
                    .find_map(|event| event["sources"][0]["id"].as_str())
            }) {
                break id.to_string();
            }
            assert!(
                start.elapsed().as_secs() < 55 && poll["done"] != true,
                "No native SDK candidate qualified"
            );
            std::thread::sleep(std::time::Duration::from_millis(250));
        };
        engine
            .cancel_integration_sources("default".into(), job_id.into())
            .unwrap();
        assert!(engine.open_source(selected).is_ok());
        println!(
            "Native SDK opaque grant qualified; early cancel preserves selected source; elapsed_seconds={}",
            start.elapsed().as_secs()
        );
        drop(engine);
    }
}
