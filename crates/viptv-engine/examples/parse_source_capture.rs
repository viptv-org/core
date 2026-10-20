//! Replay sanitized, real provider labels through the production parser.
//! Usage: cargo run -p viptv-engine --example parse_source_capture -- CAPTURE.json
//! Input is an array, or {"cases":[{"source":{...}}]} / {"sources":[...]}.
use serde_json::{Value, json};
use std::{collections::BTreeMap, env, fs, time::Instant};
use viptv_engine::stremio::{
    SourcePresentation, source_audio_languages, source_detail, source_quality_height,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args()
        .nth(1)
        .ok_or("Pass a sanitized real capture JSON path")?;
    let capture: Value = serde_json::from_slice(&fs::read(path)?)?;
    let sources = capture
        .as_array()
        .or_else(|| capture.get("cases").and_then(Value::as_array))
        .or_else(|| capture.get("sources").and_then(Value::as_array))
        .ok_or("Expected an array of captured sources")?;
    if sources.is_empty() {
        return Err("Capture has no real sources".into());
    }
    let start = Instant::now();
    let mut heights = BTreeMap::<String, usize>::new();
    let mut rows = Vec::with_capacity(sources.len());
    for (index, entry) in sources.iter().enumerate() {
        let source = entry.get("source").unwrap_or(entry);
        let height = source_quality_height(source);
        *heights
            .entry(height.map_or("unknown".into(), |v| v.to_string()))
            .or_default() += 1;
        // Output only parsed display metadata; never echo raw delivery fields.
        rows.push(json!({"index":index,"quality_height":height,
            "audio_languages":source_audio_languages(source),
            "detail":source_detail(source),
            "presentation":SourcePresentation::from_stream(source)}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "captured_sources":sources.len(),"elapsed_ms":start.elapsed().as_millis(),
            "network_requests":0,"quality_counts":heights,"parsed":rows
        }))?
    );
    Ok(())
}
