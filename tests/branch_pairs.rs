//! 六冲、六害与限定采用表六破的独立身份、配对和编码验收。

use matharts_core::{Branch, InvalidIndex, SixBreak, SixClash, SixHarm};

// 固定样本直接写出成员，不使用生产查询生成预期。
const CLASH: [(SixClash, &str, [Branch; 2]); 6] = [
    (SixClash::ZiWu, "ZiWu", [Branch::Zi, Branch::Wu]),
    (SixClash::ChouWei, "ChouWei", [Branch::Chou, Branch::Wei]),
    (SixClash::YinShen, "YinShen", [Branch::Yin, Branch::Shen]),
    (SixClash::MaoYou, "MaoYou", [Branch::Mao, Branch::You]),
    (SixClash::ChenXu, "ChenXu", [Branch::Chen, Branch::Xu]),
    (SixClash::SiHai, "SiHai", [Branch::Si, Branch::Hai]),
];
const HARM: [(SixHarm, &str, [Branch; 2]); 6] = [
    (SixHarm::ZiWei, "ZiWei", [Branch::Zi, Branch::Wei]),
    (SixHarm::ChouWu, "ChouWu", [Branch::Chou, Branch::Wu]),
    (SixHarm::YinSi, "YinSi", [Branch::Yin, Branch::Si]),
    (SixHarm::MaoChen, "MaoChen", [Branch::Mao, Branch::Chen]),
    (SixHarm::ShenHai, "ShenHai", [Branch::Shen, Branch::Hai]),
    (SixHarm::YouXu, "YouXu", [Branch::You, Branch::Xu]),
];
const BREAK: [(SixBreak, &str, [Branch; 2]); 6] = [
    (SixBreak::ZiYou, "ZiYou", [Branch::Zi, Branch::You]),
    (SixBreak::ChouChen, "ChouChen", [Branch::Chou, Branch::Chen]),
    (SixBreak::YinHai, "YinHai", [Branch::Yin, Branch::Hai]),
    (SixBreak::MaoWu, "MaoWu", [Branch::Mao, Branch::Wu]),
    (SixBreak::SiShen, "SiShen", [Branch::Si, Branch::Shen]),
    (SixBreak::WeiXu, "WeiXu", [Branch::Wei, Branch::Xu]),
];

macro_rules! test_pairs {
    ($test:ident, $ty:ident, $cases:ident, $query:ident, $predicate:ident) => {
        #[test]
        fn $test() {
            assert_eq!($ty::ALL, $cases.map(|case| case.0));
            let mut coverage = [0; 12];
            for (index, (group, _, members)) in $cases.iter().enumerate() {
                assert_eq!(usize::from(group.index()), index);
                assert_eq!(group.members(), *members);
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
                for branch in *members {
                    coverage[branch as usize] += 1;
                    assert_eq!($ty::from_branch(branch), *group);
                    assert_eq!(branch.$query(), *group);
                }
            }
            assert_eq!(coverage, [1; 12]);
            let mut accepted = 0;
            for a in Branch::ALL {
                for b in Branch::ALL {
                    let expected = $cases
                        .iter()
                        .find(|case| case.2 == [a, b] || case.2 == [b, a])
                        .map(|case| case.0);
                    assert_eq!($ty::from_branches([a, b]), expected);
                    assert_eq!(a.$predicate(b), expected.is_some());
                    accepted += usize::from(expected.is_some());
                }
            }
            assert_eq!(accepted, 12);
            for index in 0..=u8::MAX {
                let expected =
                    $cases
                        .get(usize::from(index))
                        .map(|case| case.0)
                        .ok_or(InvalidIndex {
                            index,
                            upper_bound: 6,
                        });
                assert_eq!($ty::try_from(index), expected);
            }
        }
    };
}

test_pairs!(
    clash_exhausts_domain_pairs_and_indices,
    SixClash,
    CLASH,
    six_clash,
    is_clashing_with
);
test_pairs!(
    harm_exhausts_domain_pairs_and_indices,
    SixHarm,
    HARM,
    six_harm,
    is_harming
);
test_pairs!(
    break_exhausts_domain_pairs_and_indices,
    SixBreak,
    BREAK,
    six_break,
    is_breaking
);

#[test]
fn simultaneous_relations_preserve_punishment_direction_and_opposite() {
    use matharts_core::SixCombination;
    for (members, combination, breaking) in [
        (
            [Branch::Si, Branch::Shen],
            SixCombination::SiShen,
            SixBreak::SiShen,
        ),
        (
            [Branch::Yin, Branch::Hai],
            SixCombination::YinHai,
            SixBreak::YinHai,
        ),
    ] {
        assert_eq!(SixCombination::from_branches(members), Some(combination));
        assert_eq!(SixBreak::from_branches(members), Some(breaking));
    }
    assert_eq!(
        SixHarm::from_branches([Branch::Yin, Branch::Si]),
        Some(SixHarm::YinSi)
    );
    assert!(Branch::Yin.is_punishing(Branch::Si));
    assert!(!Branch::Si.is_punishing(Branch::Yin));
    assert!(Branch::Si.is_punishing(Branch::Shen));
    assert!(!Branch::Shen.is_punishing(Branch::Si));
    for (_, _, [a, b]) in CLASH {
        assert_eq!(a.opposite(), b);
        assert_eq!(b.opposite(), a);
    }
}

#[test]
fn const_queries_and_both_export_paths_work() {
    use matharts_core::branch::{SixBreak as Break, SixClash as Clash, SixHarm as Harm};
    const CLASH_GROUP: Clash = Branch::Wu.six_clash();
    const HARM_GROUP: Harm = Branch::Wei.six_harm();
    const BREAK_GROUP: Break = Branch::You.six_break();
    const PAIR: Option<Clash> = Clash::from_branches([Branch::Wu, Branch::Zi]);
    const MEMBERS: [Branch; 2] = HARM_GROUP.members();
    const PARTNER: Option<Branch> = BREAK_GROUP.partner_of(Branch::You);
    const {
        assert!(CLASH_GROUP.contains(Branch::Zi));
    }
    assert_eq!(PAIR, Some(Clash::ZiWu));
    assert_eq!(MEMBERS, [Branch::Zi, Branch::Wei]);
    assert_eq!(PARTNER, Some(Branch::Zi));
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
        cases: &[(T, &'static str, [Branch; 2])],
        other_codes: &[&str],
    ) {
        let f: Value = serde_json::from_str(include_str!("fixtures/branch-pairs-v1.json")).unwrap();
        assert_eq!(f.as_object().unwrap().len(), 7);
        assert_eq!(f["schema_version"], 1);
        assert_eq!(f["member_order"], "ascending_domain_index");
        assert_eq!(f["serde_model"], "unit_variant");
        let negatives = json!([null, 0, 255, [], {}, "", "unknown", "子午", "子未", "子酉"]);
        assert_eq!(
            f["invalid_json"], negatives,
            "negative coverage cannot shrink"
        );
        let rows = f[name].as_array().unwrap();
        assert_eq!(rows.len(), 6);
        assert_eq!(cases.len(), 6);
        for (index, ((group, code, members), row)) in cases.iter().zip(rows).enumerate() {
            assert_eq!(row, &json!({"index":index,"code":code,"members":members}));
            assert_eq!(serde_json::to_value(group).unwrap(), row["code"]);
            assert_eq!(
                serde_json::from_value::<T>(row["code"].clone()).unwrap(),
                *group
            );
            assert_eq!(
                serde_json::from_str::<T>(&format!("\"{code}\"")).unwrap(),
                *group
            );
            let index = u32::try_from(index).unwrap();
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
            assert_tokens(
                &(*group).readable(),
                &[Token::UnitVariant {
                    name,
                    variant: code,
                }],
            );
            assert_de_tokens(&(*group).readable(), &[Token::Str(code)]);
            for id in [
                Token::U32(index),
                Token::U64(u64::from(index)),
                Token::Str(code),
                Token::Bytes(code.as_bytes()),
            ] {
                assert_de_tokens(
                    &(*group).compact(),
                    &[Token::Enum { name }, id, Token::Unit],
                );
            }
            for value in [json!({*code:null}), json!({*code:1}), json!([code])] {
                assert!(serde_json::from_value::<T>(value.clone()).is_err());
                assert!(serde_json::from_str::<T>(&value.to_string()).is_err());
            }
        }
        for value in negatives.as_array().unwrap() {
            assert!(serde_json::from_value::<T>(value.clone()).is_err());
            assert!(serde_json::from_str::<T>(&value.to_string()).is_err());
        }
        for code in other_codes {
            assert!(serde_json::from_value::<T>(json!(code)).is_err());
        }
        for index in [6, 255, 256, u64::MAX] {
            assert_de_tokens_error::<Compact<T>>(
                &[Token::Enum { name }, Token::U64(index)],
                &format!("invalid value: integer `{index}`, expected variant index 0 <= i < 6"),
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
    fn clash_matches_all_frozen_codes_and_models() {
        let others: Vec<_> = HARM
            .iter()
            .map(|case| case.1)
            .chain(BREAK.iter().map(|case| case.1))
            .collect();
        check("SixClash", &CLASH, &others);
    }

    #[test]
    fn harm_matches_all_frozen_codes_and_models() {
        let others: Vec<_> = CLASH
            .iter()
            .map(|case| case.1)
            .chain(BREAK.iter().map(|case| case.1))
            .collect();
        check("SixHarm", &HARM, &others);
    }

    #[test]
    fn break_matches_all_frozen_codes_and_models() {
        let others: Vec<_> = CLASH
            .iter()
            .map(|case| case.1)
            .chain(HARM.iter().map(|case| case.1))
            .collect();
        check("SixBreak", &BREAK, &others);
    }
}
