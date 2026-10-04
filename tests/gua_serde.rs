//! 生产爻卦类型的固定编码、Serde 数据模型与输入模式验收。
#![cfg(feature = "serde")]

use matharts_core::{Hexagram, HexagramPosition, Trigram, TrigramPosition};
use serde::{Deserialize, Serialize, Serializer};
use serde_test::{
    Compact, Configure, Readable, Token, assert_de_tokens, assert_de_tokens_error,
    assert_ser_tokens,
};
use std::fmt;

mod support;
use support::VariantModel;

fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("fixtures/gua-v1.json")).unwrap()
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

// serde_test 1.0.177 deliberately ignores variant_index in serialize_unit_variant.
// This tiny serializer observes the exact name/index/code triple independently.

fn check_variants<T: Serialize + serde::de::DeserializeOwned + PartialEq + fmt::Debug + Copy>(
    values: &[T],
    name: &'static str,
) {
    let f = fixture();
    let model = &f["serde_model"][name];
    assert_eq!(model["kind"], "unit_variant");
    assert_eq!(model["name"], name);
    let codes = model["variants"].as_array().unwrap();
    assert_eq!(values.len(), codes.len());
    for (index, (&value, code)) in values.iter().zip(codes).enumerate() {
        let triple = value.serialize(VariantModel).unwrap();
        let index = u32::try_from(index).unwrap();
        assert_eq!(triple, (name, index, code.as_str().unwrap()));
        assert_eq!(serde_json::to_value(value).unwrap(), *code);
        assert_eq!(serde_json::from_value::<T>(code.clone()).unwrap(), value);
        let object = serde_json::json!({code.as_str().unwrap(): null});
        assert!(serde_json::from_value::<T>(object.clone()).is_err());
        assert!(serde_json::from_str::<T>(&object.to_string()).is_err());
        assert_de_tokens(
            &value.compact(),
            &[Token::Enum { name }, Token::U32(index), Token::Unit],
        );
    }
    for value in [
        serde_json::json!(0),
        serde_json::json!("unknown"),
        serde_json::json!({"First":0}),
    ] {
        assert!(serde_json::from_value::<T>(value).is_err());
    }
}

#[test]
fn all_seventeen_variant_models() {
    check_variants(
        &[
            Trigram::Kun,
            Trigram::Zhen,
            Trigram::Kan,
            Trigram::Dui,
            Trigram::Gen,
            Trigram::Li,
            Trigram::Xun,
            Trigram::Qian,
        ],
        "Trigram",
    );
    check_variants(
        &[
            TrigramPosition::First,
            TrigramPosition::Second,
            TrigramPosition::Third,
        ],
        "TrigramPosition",
    );
    check_variants(
        &[
            HexagramPosition::First,
            HexagramPosition::Second,
            HexagramPosition::Third,
            HexagramPosition::Fourth,
            HexagramPosition::Fifth,
            HexagramPosition::Sixth,
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
