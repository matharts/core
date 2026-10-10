//! Frozen-input measurements of the core operations, without Serde timing cases.
//!
//! Run with `cargo bench --bench core_operations --features serde --locked`.
//! `-- --verify-only` executes independent corpus checks without timing.
//! Timings are batch averages, not individual-operation latency percentiles.

#![allow(clippy::too_many_lines)]

use matharts_core::{CyclicSequence, Ganzhi, Hexagram, HexagramPosition, Stem, Xun, YinYang};
use serde_json::{Value, json};
use std::hint::black_box;

#[path = "support/measure.rs"]
mod measure;
use measure::{BATCH_MILLIS, SAMPLE_COUNT, measure};

fn frozen_ganzhi(checks: &mut Vec<Value>) -> Vec<Ganzhi> {
    let encoding: Value =
        serde_json::from_str(include_str!("../tests/fixtures/encoding-v3.json")).unwrap();
    assert_eq!(encoding["schema_version"], 3);
    let rows = encoding["Ganzhi"].as_array().expect("frozen Ganzhi rows");
    assert_eq!(rows.len(), 60);
    let mut values = Vec::with_capacity(60);
    for row in rows {
        let pair = row.as_array().expect("frozen stem/branch pair");
        assert_eq!(pair.len(), 2);
        let wire = json!({"stem": pair[0], "branch": pair[1]});
        let value: Ganzhi = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(value).unwrap(), wire);
        assert!(!values.contains(&value), "duplicate frozen Ganzhi");
        values.push(value);
    }
    checks.push(json!({
        "type": "Ganzhi",
        "corpus_count": values.len(),
        "verified_cases": values.len(),
    }));
    values
}

fn frozen_xuns(ganzhi: &[Ganzhi], checks: &mut Vec<Value>) -> Vec<Xun> {
    assert_eq!(ganzhi.len(), 60);
    let identities: Value =
        serde_json::from_str(include_str!("../tests/fixtures/identities-v1.json")).unwrap();
    let rows = identities["Xun"].as_array().expect("frozen Xun rows");
    assert_eq!(rows.len(), 6);
    let mut values = Vec::with_capacity(6);
    let mut verified_members = 0;
    for (index, row) in rows.iter().enumerate() {
        let value: Xun = serde_json::from_value(row.clone()).unwrap();
        assert!(!values.contains(&value), "duplicate frozen Xun");
        let members = value.members();
        for (member_index, member) in members.iter().enumerate() {
            assert_eq!(*member, ganzhi[index * 10 + member_index]);
            verified_members += 1;
        }
        values.push(value);
    }
    assert_eq!(verified_members, 60);
    checks.push(json!({
        "type": "Xun",
        "corpus_count": values.len(),
        "verified_cases": verified_members,
    }));
    values
}

struct FrozenHexagram {
    value: Hexagram,
    lines: [YinYang; 6],
}

struct HexagramInputs {
    lines: Vec<(Hexagram, HexagramPosition)>,
    updates: Vec<(Hexagram, HexagramPosition, YinYang)>,
}

fn expected_hexagram(rows: &[FrozenHexagram], lines: [YinYang; 6]) -> Hexagram {
    rows.iter()
        .find(|row| row.lines == lines)
        .expect("all six-line structures must exist in frozen corpus")
        .value
}

fn frozen_hexagrams(checks: &mut Vec<Value>) -> HexagramInputs {
    let gua: Value = serde_json::from_str(include_str!("../tests/fixtures/gua-v1.json")).unwrap();
    assert_eq!(gua["status"], "frozen-gua-encoding-v1");
    assert_eq!(gua["bit_convention"]["lines_order"], "bottom-to-top");
    assert_eq!(gua["bit_convention"]["yang"], 1);
    assert_eq!(gua["bit_convention"]["yin"], 0);
    let trigram_rows = gua["trigrams"].as_array().expect("frozen trigram rows");
    assert_eq!(trigram_rows.len(), 8);
    let mut trigrams: Vec<(&str, [YinYang; 3])> = Vec::with_capacity(8);
    for (index, row) in trigram_rows.iter().enumerate() {
        let code = row["code"].as_str().expect("frozen trigram code");
        let lines: [YinYang; 3] = serde_json::from_value(row["lines"].clone()).unwrap();
        let expected_bits = lines.iter().enumerate().fold(0_u64, |bits, (i, line)| {
            bits | if *line == YinYang::Yang { 1 << i } else { 0 }
        });
        assert_eq!(row["bits"], index);
        assert_eq!(row["bits"], expected_bits);
        assert!(!trigrams.iter().any(|(known, _)| *known == code));
        trigrams.push((code, lines));
    }
    let hexagram_rows = gua["hexagrams"].as_array().expect("frozen hexagram rows");
    assert_eq!(hexagram_rows.len(), 64);
    let mut hexagrams: Vec<FrozenHexagram> = Vec::with_capacity(64);
    for (index, row) in hexagram_rows.iter().enumerate() {
        assert_eq!(row["bits"], index);
        let wire = &row["wire"];
        let lower = trigrams
            .iter()
            .find(|(code, _)| Some(*code) == wire["lower"].as_str())
            .expect("frozen lower trigram");
        let upper = trigrams
            .iter()
            .find(|(code, _)| Some(*code) == wire["upper"].as_str())
            .expect("frozen upper trigram");
        // Expected lines come only from the independent three-line rows.
        let lines = [
            lower.1[0], lower.1[1], lower.1[2], upper.1[0], upper.1[1], upper.1[2],
        ];
        let value: Hexagram = serde_json::from_value(wire.clone()).unwrap();
        assert!(!hexagrams.iter().any(|known| known.value == value));
        assert!(!hexagrams.iter().any(|known| known.lines == lines));
        assert_eq!(value.lines(), lines);
        hexagrams.push(FrozenHexagram { value, lines });
    }
    let position_rows = gua["serde_model"]["HexagramPosition"]["variants"]
        .as_array()
        .expect("frozen hexagram positions");
    assert_eq!(position_rows.len(), 6);
    let positions: Vec<HexagramPosition> = position_rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let position: HexagramPosition = serde_json::from_value(row.clone()).unwrap();
            assert_eq!(usize::from(position.index()), index);
            position
        })
        .collect();
    let mut inputs = HexagramInputs {
        lines: Vec::with_capacity(384),
        updates: Vec::with_capacity(768),
    };
    let mut query_checks = 0;
    let mut update_checks = 0;
    let mut toggle_checks = 0;
    for row in &hexagrams {
        for (index, position) in positions.iter().enumerate() {
            assert_eq!(row.value.line(*position), row.lines[index]);
            query_checks += 1;
            let mut toggled = row.lines;
            toggled[index] = match toggled[index] {
                YinYang::Yang => YinYang::Yin,
                YinYang::Yin => YinYang::Yang,
            };
            assert_eq!(
                row.value.toggle_line(*position),
                expected_hexagram(&hexagrams, toggled)
            );
            toggle_checks += 1;
            inputs.lines.push((row.value, *position));
            for value in [YinYang::Yin, YinYang::Yang] {
                let mut replaced = row.lines;
                replaced[index] = value;
                assert_eq!(
                    row.value.with_line(*position, value),
                    expected_hexagram(&hexagrams, replaced)
                );
                update_checks += 1;
                inputs.updates.push((row.value, *position, value));
            }
        }
    }
    assert_eq!(inputs.lines.len(), 384);
    assert_eq!(inputs.updates.len(), 768);
    assert_eq!(
        (query_checks, update_checks, toggle_checks),
        (384, 768, 384)
    );
    checks.push(json!({
        "type": "Hexagram",
        "corpus_count": hexagrams.len(),
        "trigram_count": trigrams.len(),
        "position_count": positions.len(),
        "identity_cases": hexagrams.len(),
        "line_cases": query_checks,
        "with_line_cases": update_checks,
        "toggle_line_cases": toggle_checks,
        "verified_cases": hexagrams.len() + query_checks + update_checks + toggle_checks,
    }));
    inputs
}

fn main() {
    let verify_only = std::env::args().any(|arg| arg == "--verify-only");
    let mut checks = Vec::new();
    let ganzhi = frozen_ganzhi(&mut checks);
    let xuns = frozen_xuns(&ganzhi, &mut checks);
    let hexagrams = frozen_hexagrams(&mut checks);
    assert_eq!(checks.len(), 3);
    let mut measurements = Vec::new();
    if !verify_only {
        let shifts = [i32::MIN, -1, 0, 1, i32::MAX];
        measurements.push(measure("core", "stem_offset", 50, || {
            for stem in Stem::ALL {
                for shift in shifts {
                    let _ = black_box(black_box(stem).offset(black_box(shift)));
                }
            }
        }));
        measurements.push(measure("core", "ganzhi_offset", 300, || {
            for value in &ganzhi {
                for shift in shifts {
                    let _ = black_box(black_box(*value).offset(black_box(shift)));
                }
            }
        }));
        measurements.push(measure("core", "ganzhi_nayin", 60, || {
            for value in &ganzhi {
                let _ = black_box(black_box(*value).nayin());
            }
        }));
        measurements.push(measure("core", "xun_members", xuns.len(), || {
            for value in &xuns {
                let _ = black_box(black_box(*value).members());
            }
        }));
        measurements.push(measure(
            "core",
            "hexagram_line",
            hexagrams.lines.len(),
            || {
                for (value, position) in &hexagrams.lines {
                    let _ = black_box(black_box(*value).line(black_box(*position)));
                }
            },
        ));
        measurements.push(measure(
            "core",
            "hexagram_with_line",
            hexagrams.updates.len(),
            || {
                for (value, position, line) in &hexagrams.updates {
                    let _ = black_box(
                        black_box(*value).with_line(black_box(*position), black_box(*line)),
                    );
                }
            },
        ));
        measurements.push(measure(
            "core",
            "hexagram_toggle_line",
            hexagrams.lines.len(),
            || {
                for (value, position) in &hexagrams.lines {
                    let _ = black_box(black_box(*value).toggle_line(black_box(*position)));
                }
            },
        ));
        assert_eq!(measurements.len(), 7);
    }
    println!(
        "{}",
        json!({
            "schema_version": 2,
            "benchmark": "core_operations",
            "sample_count": SAMPLE_COUNT,
            "target_batch_ms": BATCH_MILLIS,
            "verify_only": verify_only,
            "checks": checks,
            "measurements": measurements,
        })
    );
}
