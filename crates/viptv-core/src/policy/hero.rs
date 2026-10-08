//! TV hero backdrop edge-fade pool. Renderers own shaders, timing and shuffle bags;
//! Rust owns which category a title belongs to and which edge styles it may use.
use crate::CoreError;
use facet::Facet;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The design's linear scrim. It stays available for comparison, never rotation.
pub const HERO_BASELINE_EDGE: &str = "linear";

/// Category edge pools, copied in key and member order from the hero shader
/// index (`genreEdges`). Members a renderer has not shipped are filtered out.
pub const HERO_GENRE_EDGES: &[(&str, &[&str])] = &[
    ("Horror", &["fog", "smoke", "stipple", "oldfilm"]),
    ("Sci-Fi", &["streak", "heat", "fog", "cinematic"]),
    ("Anime", &["brush", "watercolor", "streak"]),
    ("Animation", &["watercolor", "brush", "cinematic"]),
    ("Action", &["streak", "embers", "heat", "smoke"]),
    ("Crime", &["smoke", "stipple", "cinematic"]),
    ("Thriller", &["fog", "smoke", "stipple", "cinematic"]),
    ("Romance", &["cinematic", "watercolor", "fog"]),
    ("Comedy", &["brush", "watercolor", "cinematic"]),
    ("Fantasy", &["embers", "fog", "smoke", "watercolor"]),
    ("Western", &["heat", "oldfilm", "embers", "brush"]),
    ("Documentary", &["cinematic", "oldfilm", "stipple"]),
    ("War", &["embers", "smoke", "oldfilm", "stipple"]),
    ("Music", &["streak", "cinematic", "embers"]),
    ("Mystery", &["fog", "smoke", "stipple"]),
    ("Drama", &["cinematic", "brush", "fog", "oldfilm"]),
    ("Adventure", &["heat", "streak", "brush", "embers"]),
    ("Family", &["watercolor", "brush", "cinematic"]),
    ("Biography", &["oldfilm", "cinematic", "stipple"]),
    ("History", &["oldfilm", "stipple", "smoke"]),
];

/// Facts for one hero title: its normalized media type, provider genres in
/// their original order, and the edge styles this renderer actually ships.
/// Generated codecs omit empty lists, so absent lists read as empty.
#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct HeroEdgePoolInput {
    pub media_type: String,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub available_edges: Vec<String>,
}

/// The title's canonical category (a key of the shared genre edge table) and
/// the edge styles it may rotate through. `category` is absent when no genre
/// matched an available pool; `edges` is then every available edge except the
/// baseline. Empty `edges` means the renderer keeps its baseline scrim.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct HeroEdgePool {
    pub category: Option<String>,
    pub edges: Vec<String>,
}

/// Genre keys match ASCII case-insensitively. Core preserves provider genre
/// text verbatim, so case is not normalized upstream; the animation rule was
/// already case-insensitive and the pool lookup now agrees with it. Exactly
/// cased genres behave as a plain key lookup.
fn genre_pool(genre: &str) -> Option<(&'static str, &'static [&'static str])> {
    HERO_GENRE_EDGES
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(genre))
        .copied()
}

fn available_pool(pool: &[&str], available: &[String]) -> Vec<String> {
    pool.iter()
        .filter(|edge| available.iter().any(|id| id == *edge))
        .map(|edge| (*edge).to_owned())
        .collect()
}

pub fn hero_edge_pool(v: &HeroEdgePoolInput) -> HeroEdgePool {
    let pool = |category: &str| {
        genre_pool(category).map_or_else(Vec::new, |(_, pool)| {
            available_pool(pool, &v.available_edges)
        })
    };
    // Animated series read as anime even when that pool has nothing available.
    let animated = v
        .genres
        .iter()
        .any(|g| g.eq_ignore_ascii_case("Animation") || g.eq_ignore_ascii_case("Anime"));
    let (category, edges) = if animated {
        let category = if v.media_type == "series" {
            "Anime"
        } else {
            "Animation"
        };
        (Some(category.to_owned()), pool(category))
    } else {
        v.genres
            .iter()
            .find_map(|genre| {
                let (key, members) = genre_pool(genre)?;
                let edges = available_pool(members, &v.available_edges);
                (!edges.is_empty()).then(|| (Some(key.to_owned()), edges))
            })
            .unwrap_or((None, Vec::new()))
    };
    let edges = if edges.is_empty() {
        v.available_edges
            .iter()
            .filter(|id| *id != HERO_BASELINE_EDGE)
            .cloned()
            .collect()
    } else {
        edges
    };
    HeroEdgePool { category, edges }
}

pub(super) fn normalize(v: &Value) -> Result<Value, CoreError> {
    let input: HeroEdgePoolInput =
        serde_json::from_value(v.clone()).map_err(|_| CoreError::InvalidInput)?;
    serde_json::to_value(hero_edge_pool(&input)).map_err(|_| CoreError::InvalidInput)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_edges() -> Vec<String> {
        let mut ids = vec![HERO_BASELINE_EDGE.to_owned()];
        for (_, pool) in HERO_GENRE_EDGES {
            for edge in *pool {
                if !ids.iter().any(|id| id == edge) {
                    ids.push((*edge).to_owned());
                }
            }
        }
        ids
    }

    fn decide(media_type: &str, genres: &[&str], available: &[String]) -> HeroEdgePool {
        hero_edge_pool(&HeroEdgePoolInput {
            media_type: media_type.into(),
            genres: genres.iter().map(|g| (*g).into()).collect(),
            available_edges: available.to_vec(),
        })
    }

    fn strings(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|id| (*id).into()).collect()
    }

    #[test]
    fn table_has_unique_keys_and_never_rotates_the_baseline() {
        assert_eq!(HERO_GENRE_EDGES.len(), 20);
        for (i, (key, pool)) in HERO_GENRE_EDGES.iter().enumerate() {
            assert!(
                HERO_GENRE_EDGES[..i]
                    .iter()
                    .all(|(k, _)| !k.eq_ignore_ascii_case(key))
            );
            assert!(!pool.is_empty() && !pool.contains(&HERO_BASELINE_EDGE));
        }
    }

    #[test]
    fn animated_titles_choose_anime_for_series_and_animation_otherwise() {
        let all = all_edges();
        let series = decide("series", &["Drama", "animation"], &all);
        assert_eq!(series.category.as_deref(), Some("Anime"));
        assert_eq!(series.edges, strings(&["brush", "watercolor", "streak"]));
        for kind in ["movie", "episode", "Series"] {
            assert_eq!(
                decide(kind, &["ANIME"], &all).category.as_deref(),
                Some("Animation")
            );
        }
    }

    #[test]
    fn first_genre_with_an_available_pool_wins_in_table_order() {
        let available = strings(&["linear", "heat", "fog", "smoke", "cinematic"]);
        let out = decide("movie", &["Unknown", "Horror", "Mystery"], &available);
        assert_eq!(out.category.as_deref(), Some("Horror"));
        assert_eq!(out.edges, strings(&["fog", "smoke"]));
        // Action's pool is unavailable here, so Mystery is the first usable genre.
        let out = decide(
            "movie",
            &["Action", "mystery"],
            &strings(&["fog", "cinematic"]),
        );
        assert_eq!(out.category.as_deref(), Some("Mystery"));
        assert_eq!(out.edges, strings(&["fog"]));
    }

    #[test]
    fn unmatched_titles_rotate_every_available_edge_except_the_baseline() {
        let available = strings(&["brush", "linear", "unknown"]);
        assert_eq!(
            decide("movie", &["Reality", "Horror"], &available),
            HeroEdgePool {
                category: None,
                edges: strings(&["brush", "unknown"])
            }
        );
        assert_eq!(
            decide("movie", &[], &strings(&["linear"])).edges,
            Vec::<String>::new()
        );
        // The animation category stays even when its pool has nothing available.
        assert_eq!(
            decide("series", &["Anime"], &strings(&["linear", "fog"])),
            HeroEdgePool {
                category: Some("Anime".into()),
                edges: strings(&["fog"])
            }
        );
    }

    #[test]
    fn bridge_requires_media_type_and_defaults_absent_lists() {
        assert!(normalize(&serde_json::json!({"genres":[],"availableEdges":[]})).is_err());
        assert_eq!(
            normalize(&serde_json::json!({"mediaType":"movie"})).unwrap(),
            serde_json::json!({"category":null,"edges":[]})
        );
        assert_eq!(
            normalize(&serde_json::json!({"mediaType":"movie","genres":["Crime"],"availableEdges":["cinematic","smoke"]})).unwrap(),
            serde_json::json!({"category":"Crime","edges":["smoke","cinematic"]})
        );
    }
}
