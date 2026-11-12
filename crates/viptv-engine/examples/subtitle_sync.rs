use serde::Deserialize;
use std::io::{self, Read};
use viptv_engine::subtitle_sync::{CueInterval, Estimator, Observation};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Input {
    epoch: u64,
    cues: Vec<CueInterval>,
    batches: Vec<Vec<Observation>>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = Vec::new();
    io::stdin()
        .take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut input)?;
    if input.len() > 4 * 1024 * 1024 {
        return Err("input exceeds 4 MiB".into());
    }
    let input: Input = serde_json::from_slice(&input)?;
    if input.batches.len() > 32 {
        return Err("input exceeds 32 batches".into());
    }
    let mut estimator = Estimator::new(input.epoch, input.cues)?;
    for batch in input.batches {
        estimator.push(input.epoch, &batch)?;
        estimator.estimate(input.epoch)?;
    }
    println!(
        "{}",
        serde_json::to_string(&estimator.estimate(input.epoch)?)?
    );
    Ok(())
}
