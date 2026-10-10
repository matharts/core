//! Frozen-input release measurements for the production Serde decoders.
//!
//! Run with `cargo bench --bench serde_decode --features serde --locked`.
//! `-- --verify-only` checks the corpus without running timed samples.
//! Compile one decoder with `cargo rustc --bench serde_decode --profile bench
//! --features serde --locked -- --cfg serde_bench_isolated
//! --cfg 'serde_bench_type="GrowthPhase"'`, then run the emitted executable.
//! Timings are batch averages, not individual-operation latency percentiles.

#![allow(clippy::cast_precision_loss, clippy::too_many_lines)]

use serde::de::{self, DeserializeOwned, DeserializeSeed, EnumAccess, VariantAccess, Visitor};
use serde::{Deserializer, Serialize};
use serde_json::{Value, json};
use std::hint::black_box;

#[path = "support/measure.rs"]
mod measure;
use measure::{BATCH_MILLIS, SAMPLE_COUNT, measure};

/// Minimal compact enum input. This measures production identifier decoding,
/// rather than the byte parsing of any particular third-party binary format.
#[derive(Clone, Copy)]
struct CompactIndex(u64);

impl<'de> Deserializer<'de> for CompactIndex {
    type Error = de::value::Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u64(self.0)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_enum(self)
    }

    fn is_human_readable(&self) -> bool {
        false
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        map struct identifier ignored_any
    }
}

impl<'de> EnumAccess<'de> for CompactIndex {
    type Error = de::value::Error;
    type Variant = UnitVariant;

    fn variant_seed<S: DeserializeSeed<'de>>(
        self,
        seed: S,
    ) -> Result<(S::Value, Self::Variant), Self::Error> {
        seed.deserialize(de::value::U64Deserializer::<Self::Error>::new(self.0))
            .map(|value| (value, UnitVariant))
    }
}

struct UnitVariant;

impl<'de> VariantAccess<'de> for UnitVariant {
    type Error = de::value::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<S: DeserializeSeed<'de>>(
        self,
        _seed: S,
    ) -> Result<S::Value, Self::Error> {
        Err(de::Error::custom("expected unit variant"))
    }

    fn tuple_variant<V: Visitor<'de>>(
        self,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        Err(de::Error::custom("expected unit variant"))
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        Err(de::Error::custom("expected unit variant"))
    }
}

fn string_value<T: DeserializeOwned>(value: &str) -> Result<T, de::value::Error> {
    T::deserialize(de::value::BorrowedStrDeserializer::new(value))
}

fn frozen_codes(fixture: &Value, name: &str, count: usize) -> Vec<String> {
    let rows = fixture[name]
        .as_array()
        .unwrap_or_else(|| panic!("missing frozen array: {name}"));
    assert_eq!(rows.len(), count, "frozen corpus size: {name}");
    let codes: Vec<_> = rows
        .iter()
        .map(|row| {
            row.as_str()
                .or_else(|| row["code"].as_str())
                .expect("frozen code string")
                .to_owned()
        })
        .collect();
    for (index, code) in codes.iter().enumerate() {
        assert!(code.is_ascii());
        assert!(!codes[..index].contains(code), "duplicate frozen code");
    }
    codes
}

fn bench_type<T>(
    name: &str,
    codes: &[String],
    verify_only: bool,
    checks: &mut Vec<Value>,
    measurements: &mut Vec<Value>,
) where
    T: DeserializeOwned + Serialize + PartialEq + std::fmt::Debug,
{
    let json_codes: Vec<_> = codes
        .iter()
        .map(|code| serde_json::to_string(code).unwrap())
        .collect();
    let near_misses: Vec<_> = codes
        .iter()
        .map(|code| {
            let mut value = code.clone();
            value.pop();
            value.push('?');
            value
        })
        .collect();
    let json_misses: Vec<_> = near_misses
        .iter()
        .map(|code| serde_json::to_string(code).unwrap())
        .collect();
    for (index, code) in codes.iter().enumerate() {
        let from_string = string_value::<T>(code).unwrap();
        let from_json: T = serde_json::from_str(&json_codes[index]).unwrap();
        let from_index = T::deserialize(CompactIndex(index as u64)).unwrap();
        assert_eq!(from_string, from_json, "{name} string/JSON {index}");
        assert_eq!(from_string, from_index, "{name} string/index {index}");
        assert_eq!(serde_json::to_value(&from_string).unwrap(), *code);
        assert!(string_value::<T>(&near_misses[index]).is_err());
        assert!(serde_json::from_str::<T>(&json_misses[index]).is_err());
    }
    let len = codes.len() as u64;
    let invalid_codes = [
        String::new(),
        "__invalid__".to_owned(),
        near_misses.last().unwrap().clone(),
        format!(" {}", codes.last().unwrap()),
        codes.last().unwrap().to_lowercase(),
        "甲".to_owned(),
    ];
    let string_errors: Vec<_> = invalid_codes
        .iter()
        .map(|code| string_value::<T>(code).unwrap_err().to_string())
        .collect();
    let index_errors: Vec<_> = [len, 255, 256, u64::MAX]
        .into_iter()
        .map(|index| T::deserialize(CompactIndex(index)).unwrap_err().to_string())
        .collect();
    checks.push(json!({
        "type": name,
        "codes": codes,
        "string_errors": string_errors,
        "index_errors": index_errors,
        "verified_cases": codes.len() * 5 + 10,
    }));
    if verify_only {
        return;
    }

    measurements.push(measure(name, "str_uniform", codes.len(), || {
        for code in codes {
            let _ = black_box(string_value::<T>(black_box(code.as_str())));
        }
    }));
    for (operation, index) in [("str_first", 0), ("str_last", codes.len() - 1)] {
        measurements.push(measure(name, operation, 1, || {
            let _ = black_box(string_value::<T>(black_box(codes[index].as_str())));
        }));
    }
    measurements.push(measure(name, "str_near_miss", near_misses.len(), || {
        for code in &near_misses {
            let _ = black_box(string_value::<T>(black_box(code.as_str())));
        }
    }));
    measurements.push(measure(name, "json_uniform", json_codes.len(), || {
        for code in &json_codes {
            let _ = black_box(serde_json::from_str::<T>(black_box(code.as_str())));
        }
    }));
    measurements.push(measure(name, "json_near_miss", json_misses.len(), || {
        for code in &json_misses {
            let _ = black_box(serde_json::from_str::<T>(black_box(code.as_str())));
        }
    }));
    let indices: Vec<_> = (0..len).collect();
    measurements.push(measure(name, "index_uniform", indices.len(), || {
        for index in &indices {
            let _ = black_box(T::deserialize(CompactIndex(black_box(*index))));
        }
    }));
    for (operation, index) in [("index_boundary", len), ("index_u64_max", u64::MAX)] {
        measurements.push(measure(name, operation, 1, || {
            let _ = black_box(T::deserialize(CompactIndex(black_box(index))));
        }));
    }
}

#[cfg_attr(serde_bench_isolated, allow(unused_variables))]
fn main() {
    let verify_only = std::env::args().any(|arg| arg == "--verify-only");
    let encoding: Value =
        serde_json::from_str(include_str!("../tests/fixtures/encoding-v3.json")).unwrap();
    let pairs: Value =
        serde_json::from_str(include_str!("../tests/fixtures/pair-groups-v1.json")).unwrap();
    let branch_pairs: Value =
        serde_json::from_str(include_str!("../tests/fixtures/branch-pairs-v1.json")).unwrap();
    let groups: Value =
        serde_json::from_str(include_str!("../tests/fixtures/branch-groups-v1.json")).unwrap();
    let identities: Value =
        serde_json::from_str(include_str!("../tests/fixtures/identities-v1.json")).unwrap();
    let gua: Value = serde_json::from_str(include_str!("../tests/fixtures/gua-v1.json")).unwrap();
    let mut checks = Vec::new();
    let mut measurements = Vec::new();
    // The macro is benchmark-only. No production Serde implementation is shared.
    macro_rules! run {
        ($type:ident, $name:literal, $fixture:expr, $count:expr) => {
            #[cfg(any(not(serde_bench_isolated), serde_bench_type = $name))]
            {
                bench_type::<matharts_core::$type>(
                    $name,
                    &frozen_codes($fixture, $name, $count),
                    verify_only,
                    &mut checks,
                    &mut measurements,
                );
            }
        };
    }
    run!(YinYang, "YinYang", &encoding, 2);
    run!(Element, "Element", &encoding, 5);
    run!(ElementRelation, "ElementRelation", &encoding, 5);
    run!(Stem, "Stem", &encoding, 10);
    run!(Branch, "Branch", &encoding, 12);
    run!(TenGod, "TenGod", &encoding, 10);
    run!(GrowthPhase, "GrowthPhase", &encoding, 12);
    run!(FiveCombination, "FiveCombination", &pairs, 5);
    run!(SixCombination, "SixCombination", &pairs, 6);
    run!(SixClash, "SixClash", &branch_pairs, 6);
    run!(SixHarm, "SixHarm", &branch_pairs, 6);
    run!(SixBreak, "SixBreak", &branch_pairs, 6);
    run!(ThreeCombination, "ThreeCombination", &groups, 4);
    run!(ThreeMeeting, "ThreeMeeting", &groups, 4);
    run!(Xun, "Xun", &identities, 6);
    run!(Nayin, "Nayin", &identities, 30);
    let positions = json!({
        "TrigramPosition": gua["serde_model"]["TrigramPosition"]["variants"],
        "HexagramPosition": gua["serde_model"]["HexagramPosition"]["variants"],
        "Trigram": gua["serde_model"]["Trigram"]["variants"],
    });
    run!(TrigramPosition, "TrigramPosition", &positions, 3);
    run!(HexagramPosition, "HexagramPosition", &positions, 6);
    run!(Trigram, "Trigram", &positions, 8);
    let isolated = cfg!(serde_bench_isolated);
    let expected_types = if isolated { 1 } else { 19 };
    assert_eq!(
        checks.len(),
        expected_types,
        "select exactly one isolated type"
    );
    let selected_type = isolated.then(|| checks[0]["type"].as_str().unwrap());
    if verify_only {
        assert_eq!(measurements.len(), 0);
    } else {
        assert_eq!(measurements.len(), expected_types * 9);
    }
    println!(
        "{}",
        json!({
            "schema_version": 2,
            "isolated": isolated,
            "selected_type": selected_type,
            "sample_count": SAMPLE_COUNT,
            "target_batch_ms": BATCH_MILLIS,
            "verify_only": verify_only,
            "checks": checks,
            "measurements": measurements,
        })
    );
}
