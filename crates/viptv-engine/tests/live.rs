use viptv_engine::live::*;
#[test]
fn m3u_channels_are_typed_and_private_uris_do_not_cross_public_contract(){
 let base=url::Url::parse("https://provider.example/subscription/list.m3u?token=private").unwrap();let body="#EXTM3U\n#EXTINF:-1 tvg-id=\"news\" tvg-logo=\"https://images.example/news.png\" group-title=\"News\",News TV\n#EXTVLCOPT:http-referrer=https://provider.example/\nhttps://media.example/live.m3u8?token=private\n#EXTINF:-1 tvg-id=\"news\" group-title=\"News\",News TV\nhttps://media.example/backup.m3u8\n";let channels=parse_m3u(body,&base,"m3u:opaque").unwrap();assert_eq!(channels.len(),1);assert_eq!(channels[0].sources.len(),2);assert_eq!(channels[0].channel.card.identity.kind,"live");let public=serde_json::to_string(&channels[0].channel).unwrap();assert!(!public.contains("private"));assert!(!public.contains("media.example"));assert_eq!(channels[0].sources[0].headers["Referer"],"https://provider.example/");assert!(parse_m3u("#EXTM3U\n#EXT-X-TARGETDURATION:6\nsegment.ts",&base,"m3u:opaque").is_err());
}
#[test]
fn xmltv_preserves_timezone_entities_and_rejects_external_entity_documents(){
 let xml=r#"<tv><programme channel="news" start="20261013000000 +0200" stop="20261013013000 +0200"><title>News &amp; weather</title><desc><![CDATA[Actual <guide>]]></desc></programme><programme channel="other" start="20261013000000 +0000" stop="20261013010000 +0000"><title>Other</title></programme></tv>"#;let programs=parse_xmltv(xml,"news","opaque-channel").unwrap();assert_eq!(programs.len(),1);assert_eq!(programs[0].title,"News & weather");assert_eq!(programs[0].end_millis-programs[0].start_millis,90*60*1000);assert_eq!(programs[0].description.as_deref(),Some("Actual <guide>"));assert!(parse_xmltv(r#"<!DOCTYPE tv SYSTEM "file:///private"><tv/>"#,"news","opaque").is_err());assert!(parse_xmltv("<tv><programme>","news","opaque").is_err());
}

#[test]
fn xmltv_naive_times_default_to_utc_and_offsets_stay_absolute(){
 let xml=r#"<tv><programme channel="news" start="20250101120000" stop="20250101130000"><title>Naive noon</title></programme><programme channel="news" start="20250101120000 +0200" stop="20250101130000 +0200"><title>Plus two</title></programme><programme channel="news" start="20250101230000" stop="20250102010000"><title>Midnight crossing</title></programme></tv>"#;
 let programs=parse_xmltv(xml,"news","opaque-channel").unwrap();
 assert_eq!(programs.len(),3);
 let find=|title:&str|programs.iter().find(|p|p.title==title).unwrap();
 let naive=find("Naive noon");
 assert_eq!(naive.start_millis,1735732800000);
 assert_eq!(naive.end_millis,1735736400000);
 let offset=find("Plus two");
 assert_eq!(offset.start_millis,1735725600000);
 assert_eq!(offset.end_millis,1735729200000);
 let midnight=find("Midnight crossing");
 assert_eq!(midnight.start_millis,1735772400000);
 assert_eq!(midnight.end_millis,1735779600000);
}

#[test]
fn xmltv_keeps_title_and_description_text_after_inline_tags(){
 let xml=r#"<tv><programme channel="news" start="20261013000000 +0000" stop="20261013010000 +0000"><title>Foo <b>bar</b> baz</title><desc>Nested <b>bold</b> and <i>italic</i> text</desc></programme></tv>"#;let programs=parse_xmltv(xml,"news","opaque-channel").unwrap();assert_eq!(programs.len(),1);assert_eq!(programs[0].title,"Foo bar baz");assert_eq!(programs[0].description.as_deref(),Some("Nested bold and italic text"));
}

#[test]
fn directory_offsets_do_not_truncate_catalogs_but_pages_remain_bounded() {
    for offset in [10_000,10_040,1_000_000,u32::MAX] {
        assert!(LiveQuery{offset,limit:Some(200),..Default::default()}.validate().is_ok());
    }
    for limit in [0,201,u32::MAX] {
        assert!(LiveQuery{offset:10_040,limit:Some(limit),..Default::default()}.validate().is_err());
    }
    assert!(serde_json::from_str::<LiveQuery>(r#"{"offset":4294967296}"#).is_err());
    assert!(LiveQuery{search:Some("x".repeat(201)),..Default::default()}.validate().is_err());
    for page_size in [0,201] {
        assert!(LiveQuery{page_size:Some(page_size),..Default::default()}.validate().is_err());
    }
    assert!(LiveQuery{offset:0,limit:Some(2),page_size:Some(40),..Default::default()}.validate().is_ok());
    assert!(serde_json::to_value(LiveQuery::default()).unwrap().get("page_size").is_none());
}

#[test]
fn m3u_parses_every_channel_of_a_large_playlist() {
    let base = url::Url::parse("https://provider.example/list.m3u").unwrap();
    let mut body = String::from("#EXTM3U\n");
    for i in 0..20_000 {
        body.push_str(&format!(
            "#EXTINF:-1 tvg-id=\"c{i}\" group-title=\"Group {i}\",Channel {i}\nhttps://media.example/{i}.m3u8?token=private\n"
        ));
    }
    let channels = parse_m3u(&body, &base, "m3u:opaque").unwrap();
    assert_eq!(channels.len(), 20_000);
    assert_eq!(channels[19_999].channel.card.title, "Channel 19999");
}

#[test]
fn xtream_parses_every_stream_of_a_large_feed() {
    let configuration = XtreamConfiguration {
        server_url: "https://provider.example".into(),
        username: "fixture-user".into(),
        password: "fixture-password".into(),
    };
    let values: Vec<serde_json::Value> = (0..20_000)
        .map(|i| {
            serde_json::json!({"stream_id": i, "name": format!("Channel {i}"), "category_id": "news"})
        })
        .collect();
    let raw = serde_json::to_string(&values).unwrap();
    let channels = parse_xtream(&raw, "xtream:opaque", &configuration).unwrap();
    assert_eq!(channels.len(), 20_000);
    assert_eq!(channels[19_999].stream_id.as_deref(), Some("19999"));
}
