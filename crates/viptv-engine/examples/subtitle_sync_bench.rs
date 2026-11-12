use std::time::Instant;
use viptv_engine::subtitle_sync::*;

fn main() {
    let mut random = 42_u64;
    let mut cues = Vec::new();
    let mut t = 0_i64;
    while t < 1_200_000_000 {
        random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
        t += (3 + ((random >> 32) % 30) as i64) * 100_000;
        let end = t + (4 + ((random >> 40) % 35) as i64) * 100_000;
        cues.push(CueInterval {
            start_us: t,
            end_us: end,
        });
        t = end;
    }
    let mut latencies = Vec::new();
    let mut accepted = 0;
    for run in 0..10 {
        let mut estimator = Estimator::new(run, cues.clone()).unwrap();
        for window in 0..15 {
            let start = 40_000_000 + window * 60_000_000;
            let observations: Vec<_> = (0..3000)
                .map(|i| {
                    let start_us = start + i * 20_000;
                    let subtitle = start_us + 10_000 - 2_000_000;
                    let speech = cues
                        .iter()
                        .any(|c| c.start_us <= subtitle && subtitle < c.end_us);
                    Observation {
                        start_us,
                        end_us: start_us + 20_000,
                        activity: if speech {
                            Activity::Speech
                        } else {
                            Activity::Silence
                        },
                    }
                })
                .collect();
            let before = Instant::now();
            estimator.push(run, &observations).unwrap();
            let estimate = estimator.estimate(run).unwrap();
            latencies.push(before.elapsed().as_secs_f64() * 1_000.0);
            accepted += usize::from(estimate.state == State::Accepted);
        }
    }
    latencies.sort_by(f64::total_cmp);
    let process_peak_rss_kib =
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|status| {
                status.lines().find_map(|line| {
                    line.strip_prefix("VmHWM:")?
                        .split_whitespace()
                        .next()?
                        .parse::<u64>()
                        .ok()
                })
            });
    println!(
        "{}",
        serde_json::json!({ "fixture": "synthetic-activity-only", "windows": latencies.len(),
        "acceptedWindows": accepted, "windowSeconds": 60, "meanMs": latencies.iter().sum::<f64>() / latencies.len() as f64,
        "p95Ms": latencies[(latencies.len() * 95 / 100).min(latencies.len() - 1)],
        "maxMs": latencies.last().unwrap(), "processPeakRssKiB": process_peak_rss_kib, "algorithmVersion": ALGORITHM_VERSION })
    );
}
