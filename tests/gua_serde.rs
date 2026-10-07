//! 生产爻卦类型的固定编码、Serde 数据模型与输入模式验收。
#![cfg(feature = "serde")]

use matharts_core::{Hexagram, HexagramPosition, Trigram, TrigramPosition};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::{Value, json};
use serde_test::{
    Compact, Configure, Readable, Token, assert_de_tokens, assert_de_tokens_error,
    assert_ser_tokens,
};
use std::fmt;

mod support;
use support::VariantModel;

fn fixture() -> Value {
    let value = serde_json::from_str(include_str!("fixtures/gua-v1.json")).unwrap();
    assert_frozen_fixture(&value);
    value
}

// Independently freeze every sample category, including metadata and duplicate-key text.
// Counts alone cannot reject replacement by an already present invalid sample.
fn assert_frozen_fixture(f: &Value) {
    assert_eq!(f.as_object().unwrap().len(), 10);
    assert_eq!(f["status"], "frozen-gua-encoding-v1");
    assert_eq!(
        f["bit_convention"],
        json!({"bottom_line_bit":0,"yang":1,"yin":0,"lines_order":"bottom-to-top"})
    );
    assert_eq!(f["invalid_trigram_bits"], json!([8, 255]));
    assert_eq!(f["invalid_hexagram_bits"], json!([64, 255]));
    assert_eq!(
        f["invalid_hexagram_json"],
        json!([
            null, 0, [], ["Qian", "Kun"], ["Kun", "Qian"], ["Qian"],
            ["Qian", "Kun", "Kan"], "Qian", true,
            {"lower":{"Qian":null},"upper":"Kun"}, {}, {"lower":"Qian"},
            {"lower":"Qian","upper":"unknown"}, {"lower":7,"upper":0},
            {"lower":"乾","upper":"坤"},
            {"lower":"Qian","upper":"Kun","moving_line":1}
        ]),
        "frozen JSON negatives must not shrink or be replaced"
    );
    assert_eq!(
        f["invalid_hexagram_json_text"],
        json!([
            r#"{"lower":"Qian","lower":"Kun","upper":"Kun"}"#,
            r#"{"lower":"Qian","upper":"Kun","upper":"Qian"}"#
        ]),
        "both duplicate-field texts must remain distinct"
    );
    assert_eq!(
        f["serde_model"],
        json!({
            "Trigram":{"kind":"unit_variant","name":"Trigram",
                "variants":["Kun","Zhen","Kan","Dui","Gen","Li","Xun","Qian"]},
            "TrigramPosition":{"kind":"unit_variant","name":"TrigramPosition",
                "variants":["First","Second","Third"]},
            "HexagramPosition":{"kind":"unit_variant","name":"HexagramPosition",
                "variants":["First","Second","Third","Fourth","Fifth","Sixth"]},
            "Hexagram":{"kind":"struct","name":"Hexagram","length":2,
                "fields":["lower","upper"]},
            "variant_index":"zero-based index in variants array",
            "human_readable_input":"map-only",
            "non_human_readable_input":"strict-map-or-two-element-struct-sequence"
        })
    );

    assert_frozen_shapes(f);
}

fn assert_frozen_shapes(f: &Value) {
    type Example = (
        &'static str,
        &'static str,
        &'static str,
        u8,
        [&'static str; 6],
    );
    let trigrams: [(&str, &str, [&str; 3]); 8] = [
        ("Kun", "坤", ["Yin", "Yin", "Yin"]),
        ("Zhen", "震", ["Yang", "Yin", "Yin"]),
        ("Kan", "坎", ["Yin", "Yang", "Yin"]),
        ("Dui", "兑", ["Yang", "Yang", "Yin"]),
        ("Gen", "艮", ["Yin", "Yin", "Yang"]),
        ("Li", "离", ["Yang", "Yin", "Yang"]),
        ("Xun", "巽", ["Yin", "Yang", "Yang"]),
        ("Qian", "乾", ["Yang", "Yang", "Yang"]),
    ];
    let rows = f["trigrams"].as_array().unwrap();
    assert_eq!(rows.len(), 8);
    for (bits, (row, (code, name, lines))) in rows.iter().zip(trigrams).enumerate() {
        assert_eq!(
            row,
            &json!({"code":code,"name":name,"bits":bits,"lines":lines})
        );
    }

    let examples: [Example; 6] = [
        ("乾", "Qian", "Qian", 63, ["Yang"; 6]),
        ("坤", "Kun", "Kun", 0, ["Yin"; 6]),
        (
            "泰",
            "Qian",
            "Kun",
            7,
            ["Yang", "Yang", "Yang", "Yin", "Yin", "Yin"],
        ),
        (
            "否",
            "Kun",
            "Qian",
            56,
            ["Yin", "Yin", "Yin", "Yang", "Yang", "Yang"],
        ),
        (
            "姤",
            "Xun",
            "Qian",
            62,
            ["Yin", "Yang", "Yang", "Yang", "Yang", "Yang"],
        ),
        (
            "屯",
            "Zhen",
            "Kan",
            17,
            ["Yang", "Yin", "Yin", "Yin", "Yang", "Yin"],
        ),
    ];
    let rows = f["hexagram_cases"].as_array().unwrap();
    assert_eq!(rows.len(), 6);
    for (row, (label, lower, upper, bits, lines)) in rows.iter().zip(examples) {
        assert_eq!(
            row,
            &json!({"label":label,"lower":lower,"upper":upper,
            "bits":bits,"lines":lines,"wire":{"lower":lower,"upper":upper}})
        );
    }
    let rows = f["hexagrams"].as_array().unwrap();
    assert_eq!(rows.len(), 64);
    let mut seen = [false; 64];
    for row in rows {
        let bits = usize::try_from(row["bits"].as_u64().unwrap()).unwrap();
        assert!(
            bits < 64 && !seen[bits],
            "missing or duplicate hexagram bits"
        );
        seen[bits] = true;
    }
    assert!(seen.into_iter().all(|present| present));
}

#[test]
fn json_examples_and_rejections() {
    // 对照：常规派生仍接受长度正确数组，新增负例必须区分这条路径。
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Derived {
        #[serde(rename = "lower")]
        _lower: Trigram,
        #[serde(rename = "upper")]
        _upper: Trigram,
    }
    let f = fixture();
    let positive = f["hexagram_cases"].as_array().unwrap();
    let negative = f["invalid_hexagram_json"].as_array().unwrap();
    let duplicate = f["invalid_hexagram_json_text"].as_array().unwrap();
    assert_eq!(
        (positive.len(), negative.len(), duplicate.len()),
        (6, 16, 2)
    );
    for row in positive {
        let value = row["wire"].clone();
        let h: Hexagram = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(json!(h.bits()), row["bits"]);
        assert_eq!(serde_json::to_value(h.lines()).unwrap(), row["lines"]);
        assert_eq!(serde_json::to_value(h).unwrap(), value);
        assert_eq!(
            serde_json::from_str::<Hexagram>(&value.to_string()).unwrap(),
            h
        );
    }
    for value in negative {
        assert!(
            serde_json::from_value::<Hexagram>(value.clone()).is_err(),
            "{value}"
        );
        assert!(
            serde_json::from_str::<Hexagram>(&value.to_string()).is_err(),
            "{value}"
        );
    }
    for text in duplicate {
        assert!(serde_json::from_str::<Hexagram>(text.as_str().unwrap()).is_err());
    }
    assert_eq!(
        serde_json::from_str::<Hexagram>(r#"{"upper":"Kun","lower":"Qian"}"#).unwrap(),
        tai()
    );
    assert!(serde_json::from_str::<Derived>(r#"["Qian","Kun"]"#).is_ok());
}

fn tai() -> Hexagram {
    Hexagram::from_trigrams(Trigram::Qian, Trigram::Kun)
}

#[test]
fn frozen_trigram_shapes_and_invalid_bits_match_production() {
    let f = fixture();
    for row in f["trigrams"].as_array().unwrap() {
        let value: Trigram = serde_json::from_value(row["code"].clone()).unwrap();
        assert_eq!(value.name(), row["name"].as_str().unwrap());
        assert_eq!(json!(value.bits()), row["bits"]);
        assert_eq!(serde_json::to_value(value.lines()).unwrap(), row["lines"]);
    }
    for bits in f["invalid_trigram_bits"].as_array().unwrap() {
        assert!(Trigram::try_from_bits(u8::try_from(bits.as_u64().unwrap()).unwrap()).is_err());
    }
    for bits in f["invalid_hexagram_bits"].as_array().unwrap() {
        assert!(Hexagram::try_from_bits(u8::try_from(bits.as_u64().unwrap()).unwrap()).is_err());
    }
}

#[test]
fn frozen_fixture_rejects_missing_reduced_and_repeated_samples() {
    let base = fixture();
    for key in [
        "trigrams",
        "hexagram_cases",
        "hexagrams",
        "invalid_trigram_bits",
        "invalid_hexagram_bits",
        "invalid_hexagram_json",
        "invalid_hexagram_json_text",
    ] {
        let mut missing = base.clone();
        missing.as_object_mut().unwrap().remove(key);
        let mut reduced = base.clone();
        reduced[key].as_array_mut().unwrap().pop();
        let mut repeated = base.clone();
        repeated[key][1] = repeated[key][0].clone();
        for mutation in [missing, reduced, repeated] {
            assert!(
                std::panic::catch_unwind(|| assert_frozen_fixture(&mutation)).is_err(),
                "fixture mutation escaped validation: {key}"
            );
        }
    }
}

#[test]
fn sixty_four_frozen_encodings_check_output_and_input_separately() {
    let f = fixture();
    assert_eq!(f["status"], "frozen-gua-encoding-v1");
    let rows = f["hexagrams"].as_array().unwrap();
    assert_eq!(rows.len(), 64);
    let mut seen = [false; 64];
    let mut values = std::collections::HashSet::new();
    for row in rows {
        let bits = u8::try_from(row["bits"].as_u64().unwrap()).unwrap();
        assert!(!seen[usize::from(bits)]);
        seen[usize::from(bits)] = true;
        let expected = Hexagram::try_from_bits(bits).unwrap();
        assert_eq!(serde_json::to_value(expected).unwrap(), row["wire"]);
        let decoded: Hexagram = serde_json::from_value(row["wire"].clone()).unwrap();
        assert_eq!(decoded, expected);
        assert!(values.insert(decoded));
        assert_eq!(
            serde_json::from_str::<Hexagram>(&row["wire"].to_string()).unwrap(),
            expected
        );
    }
    assert!(seen.into_iter().all(|v| v));
    assert_eq!(values.len(), 64);
}

fn struct_tokens() -> [Token; 6] {
    [
        Token::Struct {
            name: "Hexagram",
            len: 2,
        },
        Token::Str("lower"),
        Token::UnitVariant {
            name: "Trigram",
            variant: "Qian",
        },
        Token::Str("upper"),
        Token::UnitVariant {
            name: "Trigram",
            variant: "Kun",
        },
        Token::StructEnd,
    ]
}

#[test]
fn struct_model_and_input_profiles() {
    let tokens = struct_tokens();
    assert_ser_tokens(&tai().readable(), &tokens);
    assert_ser_tokens(&tai().compact(), &tokens);
    let mut readable_tokens = tokens;
    readable_tokens[2] = Token::Str("Qian");
    readable_tokens[4] = Token::Str("Kun");
    assert_de_tokens(&tai().readable(), &readable_tokens);
    assert_de_tokens(&tai().compact(), &tokens);
    assert_de_tokens(
        &tai().compact(),
        &[
            Token::Seq { len: Some(2) },
            Token::UnitVariant {
                name: "Trigram",
                variant: "Qian",
            },
            Token::UnitVariant {
                name: "Trigram",
                variant: "Kun",
            },
            Token::SeqEnd,
        ],
    );
    assert_de_tokens_error::<Readable<Hexagram>>(
        &[Token::Seq { len: Some(2) }],
        "human-readable Hexagram requires an object",
    );
    assert_de_tokens_error::<Compact<Hexagram>>(
        &[Token::Seq { len: Some(0) }, Token::SeqEnd],
        "invalid length 0, expected Hexagram fields lower and upper",
    );
    assert_de_tokens_error::<Compact<Hexagram>>(
        &[
            Token::Seq { len: Some(1) },
            Token::UnitVariant {
                name: "Trigram",
                variant: "Qian",
            },
            Token::SeqEnd,
        ],
        "invalid length 1, expected Hexagram fields lower and upper",
    );
    assert_de_tokens_error::<Compact<Hexagram>>(
        &[
            Token::Seq { len: Some(3) },
            Token::UnitVariant {
                name: "Trigram",
                variant: "Qian",
            },
            Token::UnitVariant {
                name: "Trigram",
                variant: "Kun",
            },
            Token::UnitVariant {
                name: "Trigram",
                variant: "Kan",
            },
            Token::SeqEnd,
        ],
        "invalid length 3, expected Hexagram fields lower and upper",
    );
    for readable in [true, false] {
        let missing = [Token::Map { len: Some(0) }, Token::MapEnd];
        let mut duplicate = [
            Token::Map { len: None },
            Token::Str("lower"),
            Token::Str("Qian"),
            Token::Str("lower"),
        ];
        let unknown = [Token::Map { len: None }, Token::Str("moving_line")];
        if readable {
            assert_de_tokens_error::<Readable<Hexagram>>(&missing, "missing field `lower`");
            assert_de_tokens_error::<Readable<Hexagram>>(&duplicate, "duplicate field `lower`");
            assert_de_tokens_error::<Readable<Hexagram>>(
                &unknown,
                "unknown field `moving_line`, expected `lower` or `upper`",
            );
        } else {
            duplicate[2] = Token::UnitVariant {
                name: "Trigram",
                variant: "Qian",
            };
            assert_de_tokens_error::<Compact<Hexagram>>(&missing, "missing field `lower`");
            assert_de_tokens_error::<Compact<Hexagram>>(&duplicate, "duplicate field `lower`");
            assert_de_tokens_error::<Compact<Hexagram>>(
                &unknown,
                "unknown field `moving_line`, expected `lower` or `upper`",
            );
        }
    }
}

fn check_variants<
    T: Serialize + serde::de::DeserializeOwned + PartialEq + fmt::Debug + Copy,
    const N: usize,
>(
    cases: &[(T, &'static str); N],
    name: &'static str,
) {
    let f = fixture();
    let model = &f["serde_model"][name];
    assert_eq!(model["kind"], "unit_variant");
    assert_eq!(model["name"], name);
    let codes = model["variants"].as_array().unwrap();
    assert_eq!(cases.len(), codes.len());
    for ((_, expected), code) in cases.iter().zip(codes) {
        assert_eq!(code, expected);
    }
    support::assert_enum_contract(name, cases, support::IdentifierProfile { bytes: false });
    for (_, code) in cases {
        for identifier in [
            Token::Bytes(code.as_bytes()),
            Token::BorrowedBytes(code.as_bytes()),
        ] {
            assert_de_tokens_error::<Compact<T>>(
                &[Token::Enum { name }, identifier],
                &format!("invalid type: byte array, expected {name} variant code or index"),
            );
        }
    }
    support::assert_json_rejected::<T>(&json!({"First":0}));
}

#[test]
fn all_seventeen_variant_models() {
    check_variants(
        &[
            (Trigram::Kun, "Kun"),
            (Trigram::Zhen, "Zhen"),
            (Trigram::Kan, "Kan"),
            (Trigram::Dui, "Dui"),
            (Trigram::Gen, "Gen"),
            (Trigram::Li, "Li"),
            (Trigram::Xun, "Xun"),
            (Trigram::Qian, "Qian"),
        ],
        "Trigram",
    );
    check_variants(
        &[
            (TrigramPosition::First, "First"),
            (TrigramPosition::Second, "Second"),
            (TrigramPosition::Third, "Third"),
        ],
        "TrigramPosition",
    );
    check_variants(
        &[
            (HexagramPosition::First, "First"),
            (HexagramPosition::Second, "Second"),
            (HexagramPosition::Third, "Third"),
            (HexagramPosition::Fourth, "Fourth"),
            (HexagramPosition::Fifth, "Fifth"),
            (HexagramPosition::Sixth, "Sixth"),
        ],
        "HexagramPosition",
    );
    assert_de_tokens_error::<Compact<Trigram>>(
        &[Token::Enum { name: "Trigram" }, Token::U32(8)],
        "invalid value: integer `8`, expected Trigram variant code or index",
    );
    assert_de_tokens_error::<Compact<Trigram>>(
        &[
            Token::Enum { name: "Trigram" },
            Token::Str("Qian"),
            Token::U8(1),
        ],
        "invalid type: integer `1`, expected unit",
    );
    let m = &fixture()["serde_model"]["Hexagram"];
    assert_eq!(
        m,
        &serde_json::json!({"kind":"struct","name":"Hexagram","length":2,"fields":["lower","upper"]})
    );
}

#[test]
fn model_checks_detect_json_invisible_mutations() {
    struct Mutation(u8);
    impl Serialize for Mutation {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            match self.0 {
                0 => s.serialize_str("Qian"),
                1 => s.serialize_unit_variant("Renamed", 7, "Qian"),
                _ => s.serialize_unit_variant("Trigram", 0, "Qian"),
            }
        }
    }
    for change in 0..3 {
        let m = Mutation(change);
        assert_eq!(
            serde_json::to_string(&m).unwrap(),
            serde_json::to_string(&Trigram::Qian).unwrap()
        );
        assert_ne!(m.serialize(VariantModel).ok(), Some(("Trigram", 7, "Qian")));
    }
    // Same model assertion as above, with deliberately altered expectations.
    for change in 0..3 {
        let mut tokens = struct_tokens();
        match change {
            0 => {
                tokens[0] = Token::Map { len: Some(2) };
                tokens[5] = Token::MapEnd;
            }
            1 => {
                tokens[0] = Token::Struct {
                    name: "Renamed",
                    len: 2,
                }
            }
            _ => {
                tokens.swap(1, 3);
                tokens.swap(2, 4);
            }
        }
        assert!(
            std::panic::catch_unwind(|| assert_ser_tokens(&tai().readable(), &tokens)).is_err()
        );
    }
}
