use serde_json::{Value, json};
use viptv_core::normalize;

fn run(kind: &str, input: Value) -> Result<Value, viptv_core::CoreError> {
    normalize(
        kind.into(),
        input.to_string(),
        "https://backend.example".into(),
    )
    .map(|text| serde_json::from_str(&text).unwrap())
}

#[test]
fn live_pages_preserve_provider_order_and_http_logos_without_fabricating_totals() {
    let page = json!({"catalog_id":2,"generation":0,"items":[{"id":"iptv:2:9","name":"Zulu","logo":"http://provider.example/logo.png","category_id":"Sports","category":"Sports"},{"id":"iptv:2:1","name":"Alpha","logo":null}],"next_cursor":"opaque_next"});
    let out = run("liveCatalogV2", page).unwrap();
    assert_eq!(out["catalogId"], "2");
    assert_eq!(out["generation"], "0");
    assert_eq!(out["items"][0]["id"], "iptv:2:9");
    assert_eq!(out["items"][1]["id"], "iptv:2:1");
    assert_eq!(
        out["items"][0]["poster"],
        "http://provider.example/logo.png"
    );
    assert_eq!(out["items"][0]["type"], "live");
    assert!(out.get("total").is_none());
    let typed: viptv_core::dto::LiveCatalogPage = serde_json::from_value(out).unwrap();
    assert_eq!(typed.next_cursor.as_deref(), Some("opaque_next"));
    let categories = run("liveCategoriesV2", json!({"catalog_id":2,"generation":1,"items":[{"id":"8","name":"Sports","count":99999,"url":"private-value"}],"next_cursor":null})).unwrap();
    assert_eq!(categories["items"][0], json!({"id":"8","name":"Sports"}));
    let _: viptv_core::dto::LiveCatalogCategories = serde_json::from_value(categories).unwrap();
}

#[test]
fn empty_account_and_malformed_or_legacy_pages_are_distinct() {
    assert!(
        run(
            "liveCatalogV2",
            json!({"catalog_id":null,"generation":null,"items":[],"next_cursor":null})
        )
        .is_ok()
    );
    for page in [
        json!({"items":[]}),
        json!({"catalog_id":null,"generation":1,"items":[],"next_cursor":null}),
        json!({"catalog_id":null,"generation":null,"items":[{"id":"foreign","name":"Hidden"}],"next_cursor":null}),
        json!({"catalog_id":1,"generation":0,"items":[],"next_cursor":"../../other"}),
        json!({"catalog_id":1,"generation":0,"items":vec![json!({"id":"x","name":"Channel"});201],"next_cursor":null}),
    ] {
        assert!(run("liveCatalogV2", page).is_err());
    }
}

#[test]
fn canonical_live_requests_are_cursor_bound_and_have_no_retired_filter_or_offset() {
    assert_eq!(
        run(
            "request",
            json!({"operation":"livePageV2","collection":"favorites"})
        )
        .unwrap()["path"],
        "/api/v2/iptv/live/channels?limit=50&collection=favorites"
    );
    assert!(
        run(
            "request",
            json!({"operation":"livePageV2","collection":"family"})
        )
        .is_err()
    );
    let out = run("request", json!({"operation":"livePageV2","catalogId":"2","categoryId":"news & sport","search":" News ","cursor":"opaque_next","limit":40})).unwrap();
    assert_eq!(out["method"], "GET");
    assert_eq!(
        out["path"],
        "/api/v2/iptv/live/channels?limit=40&catalog_id=2&category_id=news%20%26%20sport&search=News&cursor=opaque_next"
    );
    assert!(out["body"].is_null());
    assert_eq!(
        run("request", json!({"operation":"livePageV2"})).unwrap()["path"],
        "/api/v2/iptv/live/channels?limit=50"
    );
    assert_eq!(
        run(
            "request",
            json!({"operation":"liveCategoriesV2","cursor":"next"})
        )
        .unwrap()["path"],
        "/api/v2/iptv/live/categories?limit=50&cursor=next"
    );
    for (field, value) in [
        ("offset", json!(40)),
        ("view", json!("us")),
        ("limit", json!(201)),
        ("limit", json!(0)),
        ("catalogId", json!("-1")),
        ("cursor", json!("bad?token")),
        ("search", json!("x".repeat(129))),
    ] {
        let mut input = json!({"operation":"livePageV2"});
        input[field] = value;
        assert!(run("request", input).is_err());
    }
    assert!(
        run(
            "request",
            json!({"operation":"liveCategoriesV2","categoryId":"8"})
        )
        .is_err()
    );
    assert_eq!(
        run(
            "request",
            json!({"operation":"liveSourceV2","id":"iptv:2:7"})
        )
        .unwrap()["path"],
        "/api/v2/iptv/live/iptv%3A2%3A7/source"
    );
    assert_eq!(
        run(
            "request",
            json!({"operation":"liveGuideV2","id":"iptv:2:7"})
        )
        .unwrap()["path"],
        "/api/v2/iptv/guide/iptv%3A2%3A7"
    );
}

#[test]
fn exact_live_source_keeps_only_an_opaque_owned_provider_card() {
    let mut input = json!({"source":{"id":"opaque_live","source":"iptv:2","source_addon_id":"iptv:2","name":"Provider","title":"News","source_name":"Provider","source_fingerprint":"stable","integration_key":"private-value"}});
    let out = run("liveSourceV2", input.clone()).unwrap();
    assert_eq!(out["id"], "opaque_live");
    assert_eq!(out["sourceAddonId"], "iptv:2");
    assert!(!out.to_string().contains("private-value"));
    let _: viptv_core::dto::MediaSource = serde_json::from_value(out).unwrap();
    input["source"]["url"] = json!("http://provider.example/private-url");
    assert!(run("liveSourceV2", input.clone()).is_err());
    input["source"].as_object_mut().unwrap().remove("url");
    input["source"]["source_addon_id"] = json!("addon:2");
    assert!(run("liveSourceV2", input).is_err());
}
