//! 五合与六合身份、完整配对、严格索引及独立编码样本的全集验收。

use matharts_core::{Branch, Element, FiveCombination, InvalidIndex, SixCombination, Stem};

// 独立写出固定身份与成员，不从生产方法构造预期。
const FIVE: [(FiveCombination, &str, [Stem; 2], Element); 5] = [
    (
        FiveCombination::JiaJi,
        "JiaJi",
        [Stem::Jia, Stem::Ji],
        Element::Earth,
    ),
    (
        FiveCombination::YiGeng,
        "YiGeng",
        [Stem::Yi, Stem::Geng],
        Element::Metal,
    ),
    (
        FiveCombination::BingXin,
        "BingXin",
        [Stem::Bing, Stem::Xin],
        Element::Water,
    ),
    (
        FiveCombination::DingRen,
        "DingRen",
        [Stem::Ding, Stem::Ren],
        Element::Wood,
    ),
    (
        FiveCombination::WuGui,
        "WuGui",
        [Stem::Wu, Stem::Gui],
        Element::Fire,
    ),
];
const SIX: [(SixCombination, &str, [Branch; 2]); 6] = [
    (SixCombination::ZiChou, "ZiChou", [Branch::Zi, Branch::Chou]),
    (SixCombination::YinHai, "YinHai", [Branch::Yin, Branch::Hai]),
    (SixCombination::MaoXu, "MaoXu", [Branch::Mao, Branch::Xu]),
    (
        SixCombination::ChenYou,
        "ChenYou",
        [Branch::Chen, Branch::You],
    ),
    (SixCombination::SiShen, "SiShen", [Branch::Si, Branch::Shen]),
    (SixCombination::WuWei, "WuWei", [Branch::Wu, Branch::Wei]),
];

#[test]
fn fixed_groups_partition_domains_and_check_every_partner_and_membership() {
    assert_eq!(FiveCombination::ALL, FIVE.map(|case| case.0));
    assert_eq!(SixCombination::ALL, SIX.map(|case| case.0));
    let mut stems = [0; 10];
    let mut branches = [0; 12];
    for (group, _, members, element) in FIVE {
        assert_eq!(group.members(), members);
        assert_eq!(group.element(), element);
        for stem in Stem::ALL {
            let expected = if stem == members[0] {
                Some(members[1])
            } else if stem == members[1] {
                Some(members[0])
            } else {
                None
            };
            assert_eq!(group.partner_of(stem), expected);
            assert_eq!(group.contains(stem), members.contains(&stem));
        }
        for stem in members {
            stems[stem as usize] += 1;
            assert_eq!(FiveCombination::from_stem(stem), group);
            assert_eq!(stem.five_combination(), group);
            let expected = if stem == members[0] {
                members[1]
            } else {
                members[0]
            };
            assert_eq!(stem.five_combination_partner(), expected);
        }
    }
    for (group, _, members) in SIX {
        assert_eq!(group.members(), members);
        for branch in Branch::ALL {
            let expected = if branch == members[0] {
                Some(members[1])
            } else if branch == members[1] {
                Some(members[0])
            } else {
                None
            };
            assert_eq!(group.partner_of(branch), expected);
            assert_eq!(group.contains(branch), members.contains(&branch));
        }
        for branch in members {
            branches[branch as usize] += 1;
            assert_eq!(SixCombination::from_branch(branch), group);
            assert_eq!(branch.six_combination(), group);
            let expected = if branch == members[0] {
                members[1]
            } else {
                members[0]
            };
            assert_eq!(branch.six_combination_partner(), expected);
        }
    }
    assert_eq!(stems, [1; 10]);
    assert_eq!(branches, [1; 12]);
}

#[test]
fn all_100_stem_pairs_identify_only_ten_complete_permutations() {
    let mut accepted = 0;
    for a in Stem::ALL {
        for b in Stem::ALL {
            let expected = FIVE
                .iter()
                .find(|case| case.2 == [a, b] || case.2 == [b, a])
                .map(|case| case.0);
            assert_eq!(FiveCombination::from_stems([a, b]), expected);
            accepted += usize::from(expected.is_some());
        }
    }
    assert_eq!(accepted, 10);
}

#[test]
fn all_144_branch_pairs_identify_only_twelve_complete_permutations() {
    let mut accepted = 0;
    for a in Branch::ALL {
        for b in Branch::ALL {
            let expected = SIX
                .iter()
                .find(|case| case.2 == [a, b] || case.2 == [b, a])
                .map(|case| case.0);
            assert_eq!(SixCombination::from_branches([a, b]), expected);
            accepted += usize::from(expected.is_some());
        }
    }
    assert_eq!(accepted, 12);
    // 完整六合身份不排斥其他表中关系，不抹掉刑的参数方向。
    assert_eq!(
        SixCombination::from_branches([Branch::Si, Branch::Shen]),
        Some(SixCombination::SiShen)
    );
    assert!(Branch::Si.is_breaking(Branch::Shen));
    assert!(Branch::Si.is_punishing(Branch::Shen));
    assert!(!Branch::Shen.is_punishing(Branch::Si));
}

#[test]
fn all_u8_indices_are_strict_with_fixed_identity_order() {
    for index in 0..=u8::MAX {
        let five = FIVE
            .get(usize::from(index))
            .map(|case| case.0)
            .ok_or(InvalidIndex {
                index,
                upper_bound: 5,
            });
        let six = SIX
            .get(usize::from(index))
            .map(|case| case.0)
            .ok_or(InvalidIndex {
                index,
                upper_bound: 6,
            });
        assert_eq!(FiveCombination::try_from(index), five);
        assert_eq!(SixCombination::try_from(index), six);
    }
    for (index, (group, _, _, _)) in FIVE.iter().enumerate() {
        assert_eq!(usize::from(group.index()), index);
    }
    for (index, (group, _, _)) in SIX.iter().enumerate() {
        assert_eq!(usize::from(group.index()), index);
    }
}

#[test]
fn const_queries_and_module_exports_work() {
    use matharts_core::{branch::SixCombination as Six, stem::FiveCombination as Five};
    const FIVE_GROUP: Five = Stem::Ji.five_combination();
    const SIX_GROUP: Six = Branch::Hai.six_combination();
    const MEMBERS: [Stem; 2] = FIVE_GROUP.members();
    const ELEMENT: Element = FIVE_GROUP.element();
    const PARTNER: Option<Stem> = FIVE_GROUP.partner_of(Stem::Ji);
    const CONTAINS: bool = SIX_GROUP.contains(Branch::Yin);
    const PAIR: Option<Five> = Five::from_stems([Stem::Ji, Stem::Jia]);
    const BRANCH_PAIR: Option<Six> = Six::from_branches([Branch::Hai, Branch::Yin]);
    assert_eq!(MEMBERS, [Stem::Jia, Stem::Ji]);
    assert_eq!((ELEMENT, PARTNER), (Element::Earth, Some(Stem::Jia)));
    const {
        assert!(CONTAINS);
    }
    assert_eq!(PAIR, Some(Five::JiaJi));
    assert_eq!(BRANCH_PAIR, Some(Six::YinHai));
}

#[cfg(feature = "serde")]
mod support;

#[cfg(feature = "serde")]
mod encoding {
    use super::*;
    use serde::{Serialize, de::DeserializeOwned};
    use serde_json::{Value, json};
    use serde_test::{Compact, Token, assert_de_tokens_error};

    fn fixture() -> Value {
        let f: Value = serde_json::from_str(include_str!("fixtures/pair-groups-v1.json")).unwrap();
        assert_eq!(f["schema_version"], 1);
        assert_eq!(f["member_order"], "ascending_domain_index");
        assert_eq!(f["serde_model"], "unit_variant");
        assert_eq!(f.as_object().unwrap().len(), 6);
        let required = json!([null, 0, 255, [], {}, "", "unknown", "甲己", "子丑"]);
        assert_eq!(
            f["invalid_json"], required,
            "negative sample coverage must not shrink or change"
        );
        f
    }

    fn check<T: Serialize + DeserializeOwned + Copy + Eq + core::fmt::Debug, const N: usize>(
        name: &'static str,
        cases: &[(T, &'static str); N],
        rows: &[Value],
        invalid: &[Value],
    ) {
        assert_eq!(rows.len(), cases.len());
        for (index, ((_, code), row)) in cases.iter().zip(rows).enumerate() {
            let index = u32::try_from(index).unwrap();
            assert_eq!(row["index"], index);
            assert_eq!(row["code"], *code);
        }
        support::assert_enum_contract(name, cases, support::IdentifierProfile { bytes: true });
        for value in invalid {
            support::assert_json_rejected::<T>(value);
        }
        let count = cases.len();
        for index in [u64::try_from(count).unwrap(), 255, 256, u64::MAX] {
            assert_de_tokens_error::<Compact<T>>(
                &[Token::Enum { name }, Token::U64(index)],
                &format!(
                    "invalid value: integer `{index}`, expected variant index 0 <= i < {count}"
                ),
            );
        }
        assert_de_tokens_error::<Compact<T>>(
            &[Token::Enum { name }, Token::Str(cases[0].1), Token::U8(1)],
            "invalid type: integer `1`, expected unit",
        );
        assert_de_tokens_error::<Compact<T>>(
            &[Token::Enum { name }, Token::Bytes(&[255])],
            "invalid value: byte array, expected variant identifier",
        );
    }

    #[test]
    fn five_codes_models_and_members_match_all_five_frozen_rows() {
        let f = fixture();
        let rows = f["FiveCombination"].as_array().unwrap();
        assert_eq!(rows.len(), 5);
        for (index, ((_, code, members, element), row)) in FIVE.iter().zip(rows).enumerate() {
            assert_eq!(
                row,
                &json!({"index":index,"code":code,"members":members,"element":element})
            );
        }
        check(
            "FiveCombination",
            &FIVE.map(|case| (case.0, case.1)),
            rows,
            f["invalid_json"].as_array().unwrap(),
        );
        for code in SIX.map(|case| case.1) {
            assert!(serde_json::from_value::<FiveCombination>(json!(code)).is_err());
        }
    }

    #[test]
    fn six_codes_models_and_members_match_all_six_frozen_rows() {
        let f = fixture();
        let rows = f["SixCombination"].as_array().unwrap();
        assert_eq!(rows.len(), 6);
        for (index, ((_, code, members), row)) in SIX.iter().zip(rows).enumerate() {
            assert_eq!(row, &json!({"index":index,"code":code,"members":members}));
        }
        check(
            "SixCombination",
            &SIX.map(|case| (case.0, case.1)),
            rows,
            f["invalid_json"].as_array().unwrap(),
        );
        for code in FIVE.map(|case| case.1) {
            assert!(serde_json::from_value::<SixCombination>(json!(code)).is_err());
        }
    }
}
