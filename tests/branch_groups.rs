//! 完整关系组的独立固定表、全集输入与稳定编码验收。

use matharts_core::{Branch, Element, InvalidIndex, ThreeCombination, ThreeMeeting};

const BRANCHES: [Branch; 12] = [
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
];

type Case<T> = (T, &'static str, [Branch; 3], Element);
// 手工列出已核验的三支定义；不调用生产查询建立预期。
const COMBINATIONS: [Case<ThreeCombination>; 4] = [
    (
        ThreeCombination::ShenZiChen,
        "ShenZiChen",
        [Branch::Zi, Branch::Chen, Branch::Shen],
        Element::Water,
    ),
    (
        ThreeCombination::SiYouChou,
        "SiYouChou",
        [Branch::Chou, Branch::Si, Branch::You],
        Element::Metal,
    ),
    (
        ThreeCombination::YinWuXu,
        "YinWuXu",
        [Branch::Yin, Branch::Wu, Branch::Xu],
        Element::Fire,
    ),
    (
        ThreeCombination::HaiMaoWei,
        "HaiMaoWei",
        [Branch::Mao, Branch::Wei, Branch::Hai],
        Element::Wood,
    ),
];
const MEETINGS: [Case<ThreeMeeting>; 4] = [
    (
        ThreeMeeting::YinMaoChen,
        "YinMaoChen",
        [Branch::Yin, Branch::Mao, Branch::Chen],
        Element::Wood,
    ),
    (
        ThreeMeeting::SiWuWei,
        "SiWuWei",
        [Branch::Si, Branch::Wu, Branch::Wei],
        Element::Fire,
    ),
    (
        ThreeMeeting::ShenYouXu,
        "ShenYouXu",
        [Branch::Shen, Branch::You, Branch::Xu],
        Element::Metal,
    ),
    (
        ThreeMeeting::HaiZiChou,
        "HaiZiChou",
        [Branch::Zi, Branch::Chou, Branch::Hai],
        Element::Water,
    ),
];

#[test]
fn each_relation_partitions_all_twelve_branches_with_fixed_members_and_elements() {
    assert_eq!(ThreeCombination::ALL, COMBINATIONS.map(|case| case.0));
    assert_eq!(ThreeMeeting::ALL, MEETINGS.map(|case| case.0));
    for branch in BRANCHES {
        let combinations: Vec<_> = COMBINATIONS
            .iter()
            .filter(|case| case.2.contains(&branch))
            .collect();
        let meetings: Vec<_> = MEETINGS
            .iter()
            .filter(|case| case.2.contains(&branch))
            .collect();
        assert_eq!(combinations.len(), 1);
        assert_eq!(meetings.len(), 1);
        assert_eq!(ThreeCombination::from_branch(branch), combinations[0].0);
        assert_eq!(branch.three_combination(), combinations[0].0);
        assert_eq!(ThreeMeeting::from_branch(branch), meetings[0].0);
        assert_eq!(branch.three_meeting(), meetings[0].0);
    }
    for (group, _, members, element) in COMBINATIONS {
        assert_eq!(group.members(), members);
        assert_eq!(group.element(), element);
        for branch in BRANCHES {
            assert_eq!(group.contains(branch), members.contains(&branch));
        }
    }
    for (group, _, members, element) in MEETINGS {
        assert_eq!(group.members(), members);
        assert_eq!(group.element(), element);
        for branch in BRANCHES {
            assert_eq!(group.contains(branch), members.contains(&branch));
        }
    }
}

fn check_triples<T: Copy + Eq + core::fmt::Debug>(
    cases: &[Case<T>; 4],
    lookup: impl Fn([Branch; 3]) -> Option<T>,
) {
    let mut accepted = 0;
    let mut rejected = 0;
    let mut counts = [0; 4];
    for a in BRANCHES {
        for b in BRANCHES {
            for c in BRANCHES {
                let input = [a, b, c];
                let mut sorted = input;
                sorted.sort_unstable_by_key(|branch| *branch as u8);
                let expected = cases.iter().position(|case| case.2 == sorted);
                assert_eq!(
                    lookup(input),
                    expected.map(|index| cases[index].0),
                    "{input:?}"
                );
                if let Some(index) = expected {
                    accepted += 1;
                    counts[index] += 1;
                } else {
                    rejected += 1;
                }
            }
        }
    }
    assert_eq!((accepted, rejected), (24, 1704));
    assert_eq!(counts, [6; 4]);
}

#[test]
fn all_1728_combination_inputs_accept_only_complete_distinct_sets() {
    check_triples(&COMBINATIONS, ThreeCombination::from_branches);
}

#[test]
fn all_1728_meeting_inputs_accept_only_complete_distinct_sets() {
    check_triples(&MEETINGS, ThreeMeeting::from_branches);
}

#[test]
fn all_u8_identity_indices_are_strict_and_fixed() {
    for index in 0..=u8::MAX {
        let error = InvalidIndex {
            index,
            upper_bound: 4,
        };
        let combination = COMBINATIONS
            .get(usize::from(index))
            .map_or(Err(error), |case| Ok(case.0));
        let meeting = MEETINGS
            .get(usize::from(index))
            .map_or(Err(error), |case| Ok(case.0));
        assert_eq!(ThreeCombination::try_from(index), combination);
        assert_eq!(ThreeMeeting::try_from(index), meeting);
    }
    for (index, case) in COMBINATIONS.iter().enumerate() {
        assert_eq!(usize::from(case.0.index()), index);
    }
    for (index, case) in MEETINGS.iter().enumerate() {
        assert_eq!(usize::from(case.0.index()), index);
    }
}

#[test]
fn const_queries_exports_and_distinct_relation_identities_work() {
    use matharts_core::branch::ThreeCombination as ModuleCombination;
    use matharts_core::branch::ThreeMeeting as ModuleMeeting;
    const COMBINATION: ThreeCombination = Branch::Zi.three_combination();
    const MEETING: ThreeMeeting = Branch::Zi.three_meeting();
    const WINTER: [Branch; 3] = ThreeMeeting::HaiZiChou.members();
    const ELEMENT: Element = ThreeMeeting::HaiZiChou.element();
    const INDEX: u8 = ThreeMeeting::HaiZiChou.index();
    assert_eq!(COMBINATION, ModuleCombination::ShenZiChen);
    assert_eq!(MEETING, ModuleMeeting::HaiZiChou);
    assert_eq!(WINTER, [Branch::Zi, Branch::Chou, Branch::Hai]);
    assert_eq!((ELEMENT, INDEX), (Element::Water, 3));
    assert_eq!(
        Branch::Mao.three_combination().element(),
        Branch::Mao.three_meeting().element()
    );
    assert_ne!(
        Branch::Mao.three_combination().members(),
        Branch::Mao.three_meeting().members()
    );
    assert_eq!(
        ThreeCombination::from_branches([Branch::Yin, Branch::Mao, Branch::Chen]),
        None
    );
    assert_eq!(
        ThreeMeeting::from_branches([Branch::Shen, Branch::Zi, Branch::Chen]),
        None
    );
}

#[cfg(feature = "serde")]
mod support;

#[cfg(feature = "serde")]
mod encoding {
    use super::*;
    use serde::{Serialize, de::DeserializeOwned};
    use serde_json::{Value, json};
    use serde_test::{
        Compact, Configure, Token, assert_de_tokens, assert_de_tokens_error, assert_tokens,
    };

    fn check<T: Serialize + DeserializeOwned + Copy + Eq + core::fmt::Debug>(
        name: &'static str,
        cases: &[Case<T>; 4],
    ) {
        let fixture: Value =
            serde_json::from_str(include_str!("fixtures/branch-groups-v1.json")).unwrap();
        assert_eq!(fixture["schema_version"], 1);
        assert_eq!(fixture["member_order"], "ascending_branch_index");
        assert_eq!(fixture["serde_model"], "unit_variant");
        let rows = fixture[name].as_array().unwrap();
        assert_eq!(rows.len(), 4);
        for (index, ((group, code, members, element), row)) in cases.iter().zip(rows).enumerate() {
            let index = u32::try_from(index).unwrap();
            assert_eq!(
                row,
                &json!({"index":index, "code":code, "members":members, "element":element})
            );
            assert_eq!(serde_json::to_value(group).unwrap(), row["code"]);
            assert_eq!(
                serde_json::from_value::<T>(row["code"].clone()).unwrap(),
                *group
            );
            assert_eq!(
                group.serialize(support::VariantModel).unwrap(),
                (name, index, *code)
            );
            assert_tokens(
                &(*group).compact(),
                &[Token::UnitVariant {
                    name,
                    variant: code,
                }],
            );
            for identifier in [
                Token::U32(index),
                Token::U64(u64::from(index)),
                Token::Str(code),
                Token::Bytes(code.as_bytes()),
            ] {
                assert_de_tokens(
                    &(*group).compact(),
                    &[Token::Enum { name }, identifier, Token::Unit],
                );
            }
            assert_de_tokens(&(*group).readable(), &[Token::Str(code)]);
            assert!(serde_json::from_value::<T>(json!({*code:null})).is_err());
            assert!(serde_json::from_str::<T>(&json!({*code:null}).to_string()).is_err());
            assert!(serde_json::from_value::<T>(json!({*code:1})).is_err());
        }
        for invalid in [
            json!("unknown"),
            json!("寅午戌"),
            json!(0),
            json!([]),
            json!(null),
            json!({}),
        ] {
            assert!(serde_json::from_value::<T>(invalid.clone()).is_err());
            assert!(serde_json::from_str::<T>(&invalid.to_string()).is_err());
        }
        for index in [4, 255, 256, u64::MAX] {
            assert_de_tokens_error::<Compact<T>>(
                &[Token::Enum { name }, Token::U64(index)],
                &format!("invalid value: integer `{index}`, expected variant index 0 <= i < 4"),
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
    fn combination_codes_and_models_match_all_four_frozen_rows() {
        check("ThreeCombination", &COMBINATIONS);
    }

    #[test]
    fn meeting_codes_and_models_match_all_four_frozen_rows() {
        check("ThreeMeeting", &MEETINGS);
    }
}
