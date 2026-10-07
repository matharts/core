//! 当前编码协议的独立样本验收，不通过当前编码器生成预期。
#![cfg(feature = "serde")]
use matharts_core::{
    Branch, Element, ElementRelation, God, Growth, HiddenStems, Primitive, SexagenaryCycle, Stem,
};
use matharts_core::{Nayin, Xun};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

fn fixture() -> Value {
    let value: Value = serde_json::from_str(include_str!("fixtures/encoding-v2.json")).unwrap();
    assert_eq!(value["schema_version"], 2);
    value
}

fn check<T: Serialize + DeserializeOwned + PartialEq + core::fmt::Debug>(key: &str, values: &[T]) {
    let f = fixture();
    let expected = f[key].as_array().unwrap();
    assert_eq!(values.len(), expected.len());
    for (value, wire) in values.iter().zip(expected) {
        assert_eq!(serde_json::to_value(value).unwrap(), *wire);
        assert_eq!(serde_json::from_value::<T>(wire.clone()).unwrap(), *value);
    }
}

#[test]
fn every_enum_matches_current_frozen_codes() {
    check("Primitive", &[Primitive::Yang, Primitive::Yin]);
    check(
        "Element",
        &[
            Element::Wood,
            Element::Fire,
            Element::Earth,
            Element::Metal,
            Element::Water,
        ],
    );
    check(
        "ElementRelation",
        &[
            ElementRelation::Same,
            ElementRelation::Generates,
            ElementRelation::Overcomes,
            ElementRelation::OvercomeBy,
            ElementRelation::GeneratedBy,
        ],
    );
    check("Stem", &stems());
    check("Branch", &branches());
    check(
        "God",
        &[
            God::BiJian,
            God::JieCai,
            God::ShiShen,
            God::ShangGuan,
            God::PianCai,
            God::ZhengCai,
            God::QiSha,
            God::ZhengGuan,
            God::PianYin,
            God::ZhengYin,
        ],
    );
    check(
        "Growth",
        &[
            Growth::ChangSheng,
            Growth::MuYu,
            Growth::GuanDai,
            Growth::LinGuan,
            Growth::DiWang,
            Growth::Shuai,
            Growth::Bing,
            Growth::Si,
            Growth::Mu,
            Growth::Jue,
            Growth::Tai,
            Growth::Yang,
        ],
    );
}

fn stems() -> [Stem; 10] {
    [
        Stem::Jia,
        Stem::Yi,
        Stem::Bing,
        Stem::Ding,
        Stem::Wu,
        Stem::Ji,
        Stem::Geng,
        Stem::Xin,
        Stem::Ren,
        Stem::Gui,
    ]
}
fn branches() -> [Branch; 12] {
    [
        Branch::Zi,
        Branch::Chou,
        Branch::Yin,
        Branch::Mao,
        Branch::Chen,
        Branch::Si,
        Branch::Wu,
        Branch::Wei,
        Branch::Shen,
        Branch::You,
        Branch::Xu,
        Branch::Hai,
    ]
}

#[test]
fn sixty_ganzhi_match_current_two_field_representation() {
    let f = fixture();
    let pairs = f["Ganzhi"].as_array().unwrap();
    assert_eq!(pairs.len(), 60);
    let s_codes = f["Stem"].as_array().unwrap();
    let b_codes = f["Branch"].as_array().unwrap();
    for pair in pairs {
        // 以固定代码位置查测试中的显式变体数组，避免被测解码器生成预期值。
        let s = stems()[s_codes.iter().position(|v| *v == pair[0]).unwrap()];
        let b = branches()[b_codes.iter().position(|v| *v == pair[1]).unwrap()];
        let expected = SexagenaryCycle::new(s, b).unwrap();
        let wire = json!({"stem":pair[0],"branch":pair[1]});
        assert_eq!(serde_json::to_value(expected).unwrap(), wire);
        assert_eq!(
            serde_json::from_value::<SexagenaryCycle>(wire).unwrap(),
            expected
        );
    }
}

#[test]
fn hidden_stems_match_current_optional_fields_and_reject_invalid_pairs() {
    let f = fixture();
    for (key, value) in [
        (
            "HiddenStems",
            HiddenStems {
                primary: Stem::Ji,
                secondary: Some(Stem::Gui),
                tertiary: Some(Stem::Xin),
            },
        ),
        (
            "HiddenStemsSingle",
            HiddenStems {
                primary: Stem::Gui,
                secondary: None,
                tertiary: None,
            },
        ),
    ] {
        assert_eq!(serde_json::to_value(value).unwrap(), f[key]);
        assert_eq!(
            serde_json::from_value::<HiddenStems>(f[key].clone()).unwrap(),
            value
        );
    }
    let invalid_ganzhi = f["invalid_ganzhi"]
        .as_array()
        .expect("invalid_ganzhi must exist and be an array of frozen negative samples");
    let required_cases = [
        ("mismatched parity", json!({"stem":"Jia","branch":"Chou"})),
        ("unknown stem", json!({"stem":"Unknown","branch":"Zi"})),
        ("numeric encoding", json!({"stem":0,"branch":0})),
        ("missing branch", json!({"stem":"Jia"})),
    ];
    assert_eq!(
        invalid_ganzhi.len(),
        required_cases.len(),
        "invalid_ganzhi must retain all 4 frozen negative samples; empty or reduced coverage is invalid"
    );
    for (category, sample) in required_cases {
        assert!(
            invalid_ganzhi.contains(&sample),
            "invalid_ganzhi is missing the required {category} sample: {sample}"
        );
    }
    for invalid in invalid_ganzhi {
        assert!(
            serde_json::from_value::<SexagenaryCycle>(invalid.clone()).is_err(),
            "invalid_ganzhi sample unexpectedly accepted: {invalid}"
        );
    }
    assert!(serde_json::from_str::<Stem>("10").is_err());
    assert!(serde_json::from_str::<Stem>("\"jia\"").is_err());
}

mod support;
use serde_test::{
    Compact, Configure, Token, assert_de_tokens, assert_de_tokens_error, assert_tokens,
};
use support::VariantModel;

fn check_model<T: Serialize + DeserializeOwned + Copy + PartialEq + core::fmt::Debug>(
    name: &'static str,
    cases: &[(T, &'static str)],
    expected: &Value,
) {
    let codes = expected.as_array().expect("frozen codes must be an array");
    assert_eq!(cases.len(), codes.len());
    for (index, ((value, code), frozen)) in cases.iter().zip(codes).enumerate() {
        assert_eq!(frozen, code);
        let index = u32::try_from(index).unwrap();
        assert_eq!(value.serialize(VariantModel).unwrap(), (name, index, *code));
        assert_tokens(
            &(*value).compact(),
            &[Token::UnitVariant {
                name,
                variant: code,
            }],
        );
        for identifier in [
            Token::U32(index),
            Token::U64(u64::from(index)),
            Token::Str(code),
            Token::BorrowedStr(code),
            Token::Bytes(code.as_bytes()),
            Token::BorrowedBytes(code.as_bytes()),
        ] {
            assert_de_tokens(
                &(*value).compact(),
                &[Token::Enum { name }, identifier, Token::Unit],
            );
        }
        assert_de_tokens(&(*value).readable(), &[Token::Str(code)]);
        // 可读格式统一拒绝旧 unit 对象输入。
        let object = json!({*code: null});
        assert!(serde_json::from_value::<T>(object.clone()).is_err());
        assert!(serde_json::from_str::<T>(&object.to_string()).is_err());
        assert!(serde_json::from_value::<T>(json!({*code: 1})).is_err());
        assert!(serde_json::from_str::<T>(&format!(r#"{{"{code}":null,"{code}":null}}"#)).is_err());
    }
    let len = u32::try_from(cases.len()).unwrap();
    for index in [u64::from(len), 255, 256, u64::MAX] {
        assert_de_tokens_error::<Compact<T>>(
            &[Token::Enum { name }, Token::U64(index)],
            &format!("invalid value: integer `{index}`, expected variant index 0 <= i < {len}"),
        );
    }
    assert_de_tokens_error::<Compact<T>>(
        &[Token::Enum { name }, Token::Str(cases[0].1), Token::U8(1)],
        "invalid type: integer `1`, expected unit",
    );
    for invalid in [
        json!(0),
        json!("unknown"),
        json!({}),
        json!(null),
        json!([]),
    ] {
        assert!(serde_json::from_value::<T>(invalid.clone()).is_err());
        assert!(serde_json::from_str::<T>(&invalid.to_string()).is_err());
    }
}

#[test]
fn primitive_models_match_current_variant_indices() {
    let old = fixture();
    check_model(
        "Primitive",
        &[(Primitive::Yang, "Yang"), (Primitive::Yin, "Yin")],
        &old["Primitive"],
    );
    check_model(
        "Element",
        &[
            (Element::Wood, "Wood"),
            (Element::Fire, "Fire"),
            (Element::Earth, "Earth"),
            (Element::Metal, "Metal"),
            (Element::Water, "Water"),
        ],
        &old["Element"],
    );
    check_model(
        "ElementRelation",
        &[
            (ElementRelation::Same, "Same"),
            (ElementRelation::Generates, "Generates"),
            (ElementRelation::Overcomes, "Overcomes"),
            (ElementRelation::OvercomeBy, "OvercomeBy"),
            (ElementRelation::GeneratedBy, "GeneratedBy"),
        ],
        &old["ElementRelation"],
    );
}

#[test]
fn stem_branch_and_rule_models_match_current_variant_indices() {
    let old = fixture();
    check_model(
        "Stem",
        &[
            (Stem::Jia, "Jia"),
            (Stem::Yi, "Yi"),
            (Stem::Bing, "Bing"),
            (Stem::Ding, "Ding"),
            (Stem::Wu, "Wu"),
            (Stem::Ji, "Ji"),
            (Stem::Geng, "Geng"),
            (Stem::Xin, "Xin"),
            (Stem::Ren, "Ren"),
            (Stem::Gui, "Gui"),
        ],
        &old["Stem"],
    );
    check_model(
        "Branch",
        &[
            (Branch::Zi, "Zi"),
            (Branch::Chou, "Chou"),
            (Branch::Yin, "Yin"),
            (Branch::Mao, "Mao"),
            (Branch::Chen, "Chen"),
            (Branch::Si, "Si"),
            (Branch::Wu, "Wu"),
            (Branch::Wei, "Wei"),
            (Branch::Shen, "Shen"),
            (Branch::You, "You"),
            (Branch::Xu, "Xu"),
            (Branch::Hai, "Hai"),
        ],
        &old["Branch"],
    );
    check_model(
        "God",
        &[
            (God::BiJian, "BiJian"),
            (God::JieCai, "JieCai"),
            (God::ShiShen, "ShiShen"),
            (God::ShangGuan, "ShangGuan"),
            (God::PianCai, "PianCai"),
            (God::ZhengCai, "ZhengCai"),
            (God::QiSha, "QiSha"),
            (God::ZhengGuan, "ZhengGuan"),
            (God::PianYin, "PianYin"),
            (God::ZhengYin, "ZhengYin"),
        ],
        &old["God"],
    );
    check_model(
        "Growth",
        &[
            (Growth::ChangSheng, "ChangSheng"),
            (Growth::MuYu, "MuYu"),
            (Growth::GuanDai, "GuanDai"),
            (Growth::LinGuan, "LinGuan"),
            (Growth::DiWang, "DiWang"),
            (Growth::Shuai, "Shuai"),
            (Growth::Bing, "Bing"),
            (Growth::Si, "Si"),
            (Growth::Mu, "Mu"),
            (Growth::Jue, "Jue"),
            (Growth::Tai, "Tai"),
            (Growth::Yang, "Yang"),
        ],
        &old["Growth"],
    );
}

#[test]
fn identity_models_match_current_variant_indices() {
    let old = serde_json::from_str::<Value>(include_str!("fixtures/identities-v1.json")).unwrap();
    check_model(
        "Xun",
        &[
            (Xun::JiaZi, "JiaZi"),
            (Xun::JiaXu, "JiaXu"),
            (Xun::JiaShen, "JiaShen"),
            (Xun::JiaWu, "JiaWu"),
            (Xun::JiaChen, "JiaChen"),
            (Xun::JiaYin, "JiaYin"),
        ],
        &old["Xun"],
    );
    check_model(
        "Nayin",
        &[
            (Nayin::HaiZhongJin, "HaiZhongJin"),
            (Nayin::LuZhongHuo, "LuZhongHuo"),
            (Nayin::DaLinMu, "DaLinMu"),
            (Nayin::LuPangTu, "LuPangTu"),
            (Nayin::JianFengJin, "JianFengJin"),
            (Nayin::ShanTouHuo, "ShanTouHuo"),
            (Nayin::JianXiaShui, "JianXiaShui"),
            (Nayin::ChengTouTu, "ChengTouTu"),
            (Nayin::BaiLaJin, "BaiLaJin"),
            (Nayin::YangLiuMu, "YangLiuMu"),
            (Nayin::QuanZhongShui, "QuanZhongShui"),
            (Nayin::WuShangTu, "WuShangTu"),
            (Nayin::PiLiHuo, "PiLiHuo"),
            (Nayin::SongBaiMu, "SongBaiMu"),
            (Nayin::ChangLiuShui, "ChangLiuShui"),
            (Nayin::ShaZhongJin, "ShaZhongJin"),
            (Nayin::ShanXiaHuo, "ShanXiaHuo"),
            (Nayin::PingDiMu, "PingDiMu"),
            (Nayin::BiShangTu, "BiShangTu"),
            (Nayin::JinBoJin, "JinBoJin"),
            (Nayin::FuDengHuo, "FuDengHuo"),
            (Nayin::TianHeShui, "TianHeShui"),
            (Nayin::DaYiTu, "DaYiTu"),
            (Nayin::ChaiChuanJin, "ChaiChuanJin"),
            (Nayin::SangZheMu, "SangZheMu"),
            (Nayin::DaXiShui, "DaXiShui"),
            (Nayin::ShaZhongTu, "ShaZhongTu"),
            (Nayin::TianShangHuo, "TianShangHuo"),
            (Nayin::ShiLiuMu, "ShiLiuMu"),
            (Nayin::DaHaiShui, "DaHaiShui"),
        ],
        &old["Nayin"],
    );
}

#[test]
fn current_struct_models_use_named_fields_and_explicit_optional_slots() {
    let ganzhi = SexagenaryCycle::new(Stem::Jia, Branch::Zi).unwrap();
    assert_tokens(
        &ganzhi.compact(),
        &[
            Token::Struct {
                name: "SexagenaryCycle",
                len: 2,
            },
            Token::Str("stem"),
            Token::UnitVariant {
                name: "Stem",
                variant: "Jia",
            },
            Token::Str("branch"),
            Token::UnitVariant {
                name: "Branch",
                variant: "Zi",
            },
            Token::StructEnd,
        ],
    );
    for (secondary, tertiary) in [
        (None, None),
        (Some(Stem::Gui), Some(Stem::Xin)),
        (None, Some(Stem::Xin)),
    ] {
        let hidden = HiddenStems {
            primary: Stem::Ji,
            secondary,
            tertiary,
        };
        let mut tokens = vec![
            Token::Struct {
                name: "HiddenStems",
                len: 3,
            },
            Token::Str("primary"),
            Token::UnitVariant {
                name: "Stem",
                variant: "Ji",
            },
            Token::Str("secondary"),
        ];
        if secondary.is_some() {
            tokens.extend([
                Token::Some,
                Token::UnitVariant {
                    name: "Stem",
                    variant: "Gui",
                },
            ]);
        } else {
            tokens.push(Token::None);
        }
        tokens.push(Token::Str("tertiary"));
        if tertiary.is_some() {
            tokens.extend([
                Token::Some,
                Token::UnitVariant {
                    name: "Stem",
                    variant: "Xin",
                },
            ]);
        } else {
            tokens.push(Token::None);
        }
        tokens.push(Token::StructEnd);
        assert_tokens(&hidden.compact(), &tokens);
    }
}

#[test]
fn record_json_rejects_legacy_shapes_unknown_missing_and_duplicate_fields() {
    for input in [
        r#"["Jia","Zi"]"#,
        r#"{"stem":"Jia","branch":"Zi","extra":true}"#,
        r#"{"stem":"Jia"}"#,
        r#"{"branch":"Zi"}"#,
    ] {
        assert!(
            serde_json::from_str::<SexagenaryCycle>(input).is_err(),
            "{input}"
        );
        let value: Value = serde_json::from_str(input).unwrap();
        assert!(
            serde_json::from_value::<SexagenaryCycle>(value).is_err(),
            "{input}"
        );
    }
    // Value 会合并重复键；重复字段必须通过原始 JSON 文本验收。
    for input in [
        r#"{"stem":"Jia","stem":"Jia","branch":"Zi"}"#,
        r#"{"stem":"Jia","branch":"Zi","branch":"Zi"}"#,
    ] {
        assert!(
            serde_json::from_str::<SexagenaryCycle>(input).is_err(),
            "{input}"
        );
    }
    let custom = HiddenStems {
        primary: Stem::Jia,
        secondary: None,
        tertiary: Some(Stem::Jia),
    };
    assert_eq!(
        serde_json::from_str::<HiddenStems>(
            r#"{"tertiary":"Jia","secondary":null,"primary":"Jia"}"#
        )
        .unwrap(),
        custom
    );
    for input in [
        r#"["Jia",null,"Jia"]"#,
        r#"{"primary":"Jia","secondary":null,"residual":"Jia"}"#,
        r#"{"primary":"Jia","secondary":null,"tertiary":"Jia","residual":null}"#,
        r#"{"primary":"Jia","secondary":null,"tertiary":"Jia","extra":true}"#,
        r#"{"secondary":null,"tertiary":null}"#,
        r#"{"primary":"Jia","tertiary":null}"#,
        r#"{"primary":"Jia","secondary":null}"#,
    ] {
        assert!(
            serde_json::from_str::<HiddenStems>(input).is_err(),
            "{input}"
        );
        assert!(
            serde_json::from_value::<HiddenStems>(serde_json::from_str::<Value>(input).unwrap())
                .is_err(),
            "{input}"
        );
    }
    for input in [
        r#"{"primary":"Jia","primary":"Jia","secondary":null,"tertiary":null}"#,
        r#"{"primary":"Jia","secondary":null,"secondary":null,"tertiary":null}"#,
        r#"{"primary":"Jia","secondary":null,"tertiary":null,"tertiary":null}"#,
    ] {
        assert!(
            serde_json::from_str::<HiddenStems>(input).is_err(),
            "{input}"
        );
    }
}

#[test]
fn model_observer_detects_json_invisible_changes() {
    struct Changed(u8);
    impl Serialize for Changed {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            match self.0 {
                0 => serializer.serialize_str("Jia"),
                1 => serializer.serialize_unit_variant("Renamed", 0, "Jia"),
                _ => serializer.serialize_unit_variant("Stem", 1, "Jia"),
            }
        }
    }
    for change in 0..3 {
        let value = Changed(change);
        assert_eq!(serde_json::to_string(&value).unwrap(), r#""Jia""#);
        assert_ne!(value.serialize(VariantModel).ok(), Some(("Stem", 0, "Jia")));
    }
}
