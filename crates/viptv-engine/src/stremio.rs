//! Bounded Stremio manifest and catalog projections shared by local and server hosts.
use crate::{
    Error,
    capabilities::Capability,
    identity::{ExternalId, MediaIdentity},
    presentation::Card,
};
use serde_json::Value;
use viptv_provider::{
    discover::addon_endpoint,
    extras::{bounded_exact_text, catalog_extras},
    validate_url,
};

#[derive(Clone, Debug)]
pub struct Catalog {
    pub kind: String,
    pub id: String,
    pub name: String,
    pub search: bool,
    pub required_extras: bool,
}

#[derive(Clone, Debug)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<Capability>,
    pub catalogs: Vec<Catalog>,
}

pub fn manifest_url(raw: &str) -> Result<String, Error> {
    if raw.len() > 4096 {
        return Err(Error::InvalidQuery);
    }
    let url = validate_url(raw).map_err(|_| Error::InvalidQuery)?;
    if !url.path().ends_with("/manifest.json") {
        return Err(Error::InvalidQuery);
    }
    Ok(url.into())
}

pub fn parse_manifest(raw: &str) -> Result<Manifest, Error> {
    if raw.len() > 2 * 1024 * 1024 {
        return Err(Error::InvalidQuery);
    }
    let data: Value = serde_json::from_str(raw).map_err(|_| Error::InvalidQuery)?;
    let name = bounded_exact_text(&data["name"], 128).ok_or(Error::InvalidQuery)?;
    let version = bounded_exact_text(&data["version"], 64).ok_or(Error::InvalidQuery)?;
    bounded_exact_text(&data["id"], 128).ok_or(Error::InvalidQuery)?;
    let resources = data["resources"].as_array().ok_or(Error::InvalidQuery)?;
    let has = |role: &str| {
        resources
            .iter()
            .any(|r| r.as_str() == Some(role) || r["name"].as_str() == Some(role))
    };
    let mut capabilities = vec![];
    if has("meta") {
        capabilities.push(Capability::Metadata);
    }
    if has("stream") {
        capabilities.push(Capability::Sources);
    }
    if has("subtitles") {
        capabilities.push(Capability::Subtitles);
    }
    let catalogs = data["catalogs"]
        .as_array()
        .into_iter()
        .flatten()
        .take(64)
        .filter_map(|c| {
            let kind = bounded_exact_text(&c["type"], 64)?;
            let id = bounded_exact_text(&c["id"], 128)?;
            if !kind
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || "._-".contains(ch))
            {
                return None;
            }
            let extras = catalog_extras(c);
            Some(Catalog {
                kind,
                id,
                name: bounded_exact_text(&c["name"], 128).unwrap_or_else(|| name.clone()),
                search: extras.iter().any(|e| e.name == "search"),
                required_extras: extras
                    .iter()
                    .any(|e| e.required && e.default.is_none() && e.name != "skip"),
            })
        })
        .collect::<Vec<_>>();
    if has("catalog") && !catalogs.is_empty() {
        capabilities.push(Capability::Discovery);
    }
    if catalogs.iter().any(|c| c.search) {
        capabilities.push(Capability::Search);
    }
    if capabilities.is_empty() {
        return Err(Error::InvalidQuery);
    }
    Ok(Manifest {
        name,
        version,
        capabilities,
        catalogs,
    })
}

pub fn catalog_url(manifest_url: &str, catalog: &Catalog) -> Result<String, Error> {
    let tail = format!("{}.json", catalog.id);
    addon_endpoint(manifest_url, &["catalog", &catalog.kind, &tail])
        .map_err(|_| Error::InvalidQuery)
}

fn field(v: &Value, key: &str, max: usize) -> Option<String> {
    bounded_exact_text(&v[key], max)
}
pub fn cards(raw: &str, provider: &str) -> Result<Vec<Card>, Error> {
    if raw.len() > 2 * 1024 * 1024 {
        return Err(Error::InvalidQuery);
    }
    let root: Value = serde_json::from_str(raw).map_err(|_| Error::InvalidQuery)?;
    let values = root["metas"].as_array().ok_or(Error::InvalidQuery)?;
    let mut seen = std::collections::HashSet::new();
    Ok(values
        .iter()
        .take(100)
        .filter_map(|v| {
            let id = field(v, "id", 512)?;
            let kind = field(v, "type", 64)?;
            if !seen.insert(format!("{kind}:{id}")) {
                return None;
            }
            let title = field(v, "name", 512).unwrap_or_else(|| id.clone());
            let ids = id
                .strip_prefix("tt")
                .filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
                .map(|_| {
                    vec![ExternalId {
                        namespace: "imdb".into(),
                        value: id.clone(),
                    }]
                })
                .unwrap_or_default();
            Some(Card {
                identity: MediaIdentity {
                    id,
                    kind,
                    ids,
                    episode: None,
                    edition: None,
                },
                provider_id: provider.into(),
                title,
                subtitle: field(v, "releaseInfo", 256),
                description: field(v, "description", 1600),
                poster: field(v, "poster", 4096),
                landscape: field(v, "background", 4096),
                logo: field(v, "logo", 4096),
                progress: None,
                badges: vec![],
            })
        })
        .take(24)
        .collect())
}
