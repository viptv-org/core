//! Shared normalized presentation data. Unknown metadata stays JSON and is sanitized.
use facet::Facet;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
#[serde(untagged)]
#[facet(untagged)]
#[repr(C)]
pub enum JsonValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}
pub type JsonObject = BTreeMap<String, JsonValue>;
#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
#[serde(rename_all = "lowercase")]
#[facet(rename_all = "lowercase")]
#[repr(C)]
pub enum MediaKind {
    Movie,
    Series,
    Live,
    Episode,
}
#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct CatalogExtra {
    pub name: String,
    pub required: bool,
    pub options: Vec<String>,
    pub default_value: Option<String>,
    pub options_limit: Option<f64>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct Catalog {
    pub id: String,
    pub name: String,
    pub r#type: MediaKind,
    pub addon_id: Option<f64>,
    pub addon_key: Option<String>,
    pub supports_search: bool,
    pub supports_skip: bool,
    pub extras: Vec<CatalogExtra>,
    pub genres: Vec<String>,
    pub raw: JsonObject,
}
#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct MediaItem {
    pub id: String,
    pub r#type: MediaKind,
    pub name: String,
    pub title: String,
    pub poster: Option<String>,
    pub background: Option<String>,
    pub thumbnail: Option<String>,
    pub imdb_rating: Option<String>,
    pub credits: Option<String>,
    pub poster_shape: Option<String>,
    pub updated_at_millis: Option<f64>,
    pub released_at_millis: Option<f64>,
    #[serde(default)]
    pub episodes: Vec<MediaItem>,
    pub description: Option<String>,
    pub year: Option<f64>,
    pub runtime: Option<String>,
    pub genres: Vec<String>,
    pub position: Option<f64>,
    pub duration: Option<f64>,
    pub watched: Option<bool>,
    pub season: Option<f64>,
    pub episode: Option<f64>,
    pub episode_title: Option<String>,
    pub series_id: Option<String>,
    pub queue_status: Option<String>,
    pub previous_episode: Option<Box<MediaItem>>,
    pub source_addon_id: Option<String>,
    pub source_name: Option<String>,
    pub source_fingerprint: Option<String>,
    pub source_binge_group: Option<String>,
    pub source_release_group: Option<String>,
    pub source_quality: Option<String>,
    pub source_audio: Option<String>,
    pub raw: JsonObject,
}
#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct MediaSource {
    pub provider: Option<String>,
    pub description: Option<String>,
    pub source_fingerprint: Option<String>,
    pub id: String,
    pub name: String,
    pub title: Option<String>,
    pub filename: Option<String>,
    pub source_addon_id: Option<String>,
    pub source_name: Option<String>,
    pub quality: Option<String>,
    pub audio: Option<String>,
    pub raw: JsonObject,
}
#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct MediaTrack {
    pub input_index: f64,
    pub codec: Option<String>,
    pub language: Option<String>,
    pub language_status: String,
    pub title: String,
    pub selected: bool,
    pub supported: bool,
    pub selectable: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct PlaybackSession {
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    pub id: String,
    pub url: String,
    pub format: String,
    pub mode: String,
    pub video_mode: String,
    pub audio_mode: String,
    pub position: f64,
    pub live: bool,
    pub duration: f64,
    pub audio_tracks: Vec<MediaTrack>,
    pub subtitle_tracks: Vec<MediaTrack>,
    pub subtitles_supported: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct DiscoverPage {
    pub items: Vec<MediaItem>,
    pub has_more: bool,
    pub next_skip: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct MediaPresentation {
    pub hero_image: Option<String>,
    pub poster_image: Option<String>,
    pub episode_image: Option<String>,
    pub title: String,
    pub episode_label: String,
    pub progress: f64,
    pub primary_action: String,
    pub primary_action_label: String,
    pub resume_eligible: bool,
    pub can_auto_next: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
pub struct ApiRequest {
    pub method: String,
    pub path: String,
    pub body: Option<JsonObject>,
}
