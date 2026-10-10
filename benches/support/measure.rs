//! Timing shared only by the benchmark executables.

#![allow(clippy::cast_precision_loss)]

use serde_json::{Value, json};
use std::time::{Duration, Instant};

pub const SAMPLE_COUNT: usize = 11;
pub const BATCH_MILLIS: u64 = 5;

pub fn measure(
    name: &str,
    operation: &str,
    inputs_per_round: usize,
    mut round: impl FnMut(),
) -> Value {
    assert!(
        inputs_per_round > 0,
        "measurement must execute nonempty inputs"
    );
    let target = Duration::from_millis(BATCH_MILLIS);
    let time = |rounds: usize, round: &mut dyn FnMut()| {
        let start = Instant::now();
        for _ in 0..rounds {
            round();
        }
        start.elapsed()
    };
    // Calibration is also warm-up. Keep timing calls outside the operation loop.
    let mut rounds = 16_usize;
    while time(rounds, &mut round) < target {
        rounds = rounds.checked_mul(2).expect("calibration overflow");
    }
    let _ = time(rounds, &mut round);
    let samples: Vec<_> = (0..SAMPLE_COUNT)
        .map(|_| {
            time(rounds, &mut round).as_secs_f64() * 1_000_000_000.0
                / (rounds * inputs_per_round) as f64
        })
        .collect();
    json!({
        "type": name,
        "operation": operation,
        "inputs_per_round": inputs_per_round,
        "rounds_per_sample": rounds,
        "ns_per_op": samples,
    })
}
