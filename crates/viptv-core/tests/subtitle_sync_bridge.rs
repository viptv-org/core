use viptv_core::SubtitleSyncBridge;

#[test]
fn json_bridge_is_epoch_fenced_and_size_bounded() {
    let bridge = SubtitleSyncBridge::new(7, "[]".into()).unwrap();
    let output = bridge.push(7, "[]".into()).unwrap();
    let output: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(output["epoch"], 7);
    assert_eq!(output["revision"], 0);
    assert_eq!(output["state"], "observing");
    assert_eq!(output["confidenceKind"], "uncalibratedHeuristic");
    assert!(bridge.push(8, "[]".into()).is_err());
    assert!(
        bridge
            .push(
                7,
                "[{\"startUs\":0,\"endUs\":1,\"activity\":\"unknown\"}]".into()
            )
            .is_err()
    );
    assert!(bridge.push(7, " ".repeat(1024 * 1024 + 1)).is_err());
    assert!(SubtitleSyncBridge::new(0, "[{\"startUs\":2,\"endUs\":1}]".into()).is_err());
}
