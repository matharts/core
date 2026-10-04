//! 旬与纳音身份的固定样本、全集关系、索引边界及编码契约。

use matharts_core::{Branch, CyclicRing, Element, InvalidIndex, Nayin, SexagenaryCycle, Stem, Xun};

// 固定验收表独立写出身份、名称、五行和两柱，不由实现查询生成预期。
// 纳音采用当前显示用字；文献异写不作为另一个身份。
type NayinCase = (Nayin, &'static str, Element, [(Stem, Branch); 2]);

const NAYIN_CASES: [NayinCase; 30] = {
    use Branch::{Chen, Chou, Hai, Mao, Shen, Si, Wei, Wu, Xu, Yin, You, Zi};
    use Element::{Earth, Fire, Metal, Water, Wood};
    use Nayin::{
        BaiLaJin, BiShangTu, ChaiChuanJin, ChangLiuShui, ChengTouTu, DaHaiShui, DaLinMu, DaXiShui,
        DaYiTu, FuDengHuo, HaiZhongJin, JianFengJin, JianXiaShui, JinBoJin, LuPangTu, LuZhongHuo,
        PiLiHuo, PingDiMu, QuanZhongShui, SangZheMu, ShaZhongJin, ShaZhongTu, ShanTouHuo,
        ShanXiaHuo, ShiLiuMu, SongBaiMu, TianHeShui, TianShangHuo, WuShangTu, YangLiuMu,
    };
    use Stem::{Bing, Ding, Geng, Gui, Ji, Jia, Ren, Wu as WuStem, Xin, Yi};
    [
        (HaiZhongJin, "海中金", Metal, [(Jia, Zi), (Yi, Chou)]),
        (LuZhongHuo, "炉中火", Fire, [(Bing, Yin), (Ding, Mao)]),
        (DaLinMu, "大林木", Wood, [(WuStem, Chen), (Ji, Si)]),
        (LuPangTu, "路旁土", Earth, [(Geng, Wu), (Xin, Wei)]),
        (JianFengJin, "剑锋金", Metal, [(Ren, Shen), (Gui, You)]),
        (ShanTouHuo, "山头火", Fire, [(Jia, Xu), (Yi, Hai)]),
        (JianXiaShui, "涧下水", Water, [(Bing, Zi), (Ding, Chou)]),
        (ChengTouTu, "城头土", Earth, [(WuStem, Yin), (Ji, Mao)]),
        (BaiLaJin, "白蜡金", Metal, [(Geng, Chen), (Xin, Si)]),
        (YangLiuMu, "杨柳木", Wood, [(Ren, Wu), (Gui, Wei)]),
        (QuanZhongShui, "泉中水", Water, [(Jia, Shen), (Yi, You)]),
        (WuShangTu, "屋上土", Earth, [(Bing, Xu), (Ding, Hai)]),
        (PiLiHuo, "霹雳火", Fire, [(WuStem, Zi), (Ji, Chou)]),
        (SongBaiMu, "松柏木", Wood, [(Geng, Yin), (Xin, Mao)]),
        (ChangLiuShui, "长流水", Water, [(Ren, Chen), (Gui, Si)]),
        (ShaZhongJin, "沙中金", Metal, [(Jia, Wu), (Yi, Wei)]),
        (ShanXiaHuo, "山下火", Fire, [(Bing, Shen), (Ding, You)]),
        (PingDiMu, "平地木", Wood, [(WuStem, Xu), (Ji, Hai)]),
        (BiShangTu, "壁上土", Earth, [(Geng, Zi), (Xin, Chou)]),
        (JinBoJin, "金箔金", Metal, [(Ren, Yin), (Gui, Mao)]),
        (FuDengHuo, "覆灯火", Fire, [(Jia, Chen), (Yi, Si)]),
        (TianHeShui, "天河水", Water, [(Bing, Wu), (Ding, Wei)]),
        (DaYiTu, "大驿土", Earth, [(WuStem, Shen), (Ji, You)]),
        (ChaiChuanJin, "钗钏金", Metal, [(Geng, Xu), (Xin, Hai)]),
        (SangZheMu, "桑柘木", Wood, [(Ren, Zi), (Gui, Chou)]),
        (DaXiShui, "大溪水", Water, [(Jia, Yin), (Yi, Mao)]),
        (ShaZhongTu, "沙中土", Earth, [(Bing, Chen), (Ding, Si)]),
        (TianShangHuo, "天上火", Fire, [(WuStem, Wu), (Ji, Wei)]),
        (ShiLiuMu, "石榴木", Wood, [(Geng, Shen), (Xin, You)]),
        (DaHaiShui, "大海水", Water, [(Ren, Xu), (Gui, Hai)]),
    ]
};

const XUNS: [Xun; 6] = [
    Xun::JiaZi,
    Xun::JiaXu,
    Xun::JiaShen,
    Xun::JiaWu,
    Xun::JiaChen,
    Xun::JiaYin,
];

#[test]
fn thirty_identities_cover_all_sixty_ganzhi_with_fixed_names_and_elements() {
    assert_eq!(Nayin::ALL, NAYIN_CASES.map(|case| case.0));
    let mut seen = [false; 60];
    for (index, (identity, name, element, pairs)) in (0_u8..30).zip(NAYIN_CASES) {
        assert_eq!(identity.index(), index);
        assert_eq!(Nayin::try_from(index).unwrap(), identity);
        assert_eq!(identity.name(), name);
        assert_eq!(identity.element(), element);
        let expected = pairs.map(|(stem, branch)| SexagenaryCycle::new(stem, branch).unwrap());
        assert_eq!(identity.ganzhi_pair(), expected);
        for value in expected {
            assert_eq!(value.nayin(), identity);
            assert_eq!(value.nayin().element(), element);
            let slot = usize::from(value.index());
            assert!(!seen[slot], "duplicate {value:?}");
            seen[slot] = true;
        }
    }
    assert!(seen.into_iter().all(|present| present));
}

#[test]
fn six_xun_partition_the_cycle_and_cover_void_queries() {
    use Branch::{Chen, Chou, Hai, Mao, Shen, Si, Wei, Wu, Xu, Yin, You, Zi};
    let branches: [[Branch; 10]; 6] = [
        [Zi, Chou, Yin, Mao, Chen, Si, Wu, Wei, Shen, You],
        [Xu, Hai, Zi, Chou, Yin, Mao, Chen, Si, Wu, Wei],
        [Shen, You, Xu, Hai, Zi, Chou, Yin, Mao, Chen, Si],
        [Wu, Wei, Shen, You, Xu, Hai, Zi, Chou, Yin, Mao],
        [Chen, Si, Wu, Wei, Shen, You, Xu, Hai, Zi, Chou],
        [Yin, Mao, Chen, Si, Wu, Wei, Shen, You, Xu, Hai],
    ];
    let stems = [
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
    ];
    let missing = [
        (Xu, Hai),
        (Shen, You),
        (Wu, Wei),
        (Chen, Si),
        (Yin, Mao),
        (Zi, Chou),
    ];
    let names = ["甲子旬", "甲戌旬", "甲申旬", "甲午旬", "甲辰旬", "甲寅旬"];
    assert_eq!(Xun::ALL, XUNS);
    let mut seen = [false; 60];
    for (i, xun) in XUNS.into_iter().enumerate() {
        let expected: [SexagenaryCycle; 10] =
            core::array::from_fn(|j| SexagenaryCycle::new(stems[j], branches[i][j]).unwrap());
        assert_eq!(xun.name(), names[i]);
        assert_eq!(xun.leader(), expected[0]);
        assert_eq!(xun.members(), expected);
        assert_eq!(xun.void_branches(), missing[i]);
        for value in expected {
            assert_eq!(value.xun(), xun);
            assert_eq!(value.xun().leader().branch(), branches[i][0]);
            assert_eq!(value.xun().void_branches(), missing[i]);
            for b in 0_u8..12 {
                let branch = Branch::try_from(b).unwrap();
                assert_eq!(
                    value.xun().is_void_branch(branch),
                    branch == missing[i].0 || branch == missing[i].1
                );
            }
            let slot = usize::from(value.index());
            assert!(!seen[slot], "duplicate {value:?}");
            seen[slot] = true;
        }
        // 同时验证所有非成员，而非只验证正例。
        for index in 0_u8..60 {
            let value = SexagenaryCycle::try_from(index).unwrap();
            assert_eq!(xun.contains(value), expected.contains(&value));
        }
    }
    assert!(seen.into_iter().all(|present| present));
}

#[test]
fn identity_indices_reject_all_out_of_range_inputs() {
    for index in 0..=u8::MAX {
        if let Some(value) = XUNS.get(usize::from(index)) {
            assert_eq!(Xun::try_from(index), Ok(*value));
            assert_eq!(Xun::try_from_index(index), Ok(*value));
            assert_eq!(value.index(), index);
        } else {
            let error = InvalidIndex {
                index,
                upper_bound: 6,
            };
            assert_eq!(Xun::try_from(index), Err(error));
            assert_eq!(Xun::try_from_index(index), Err(error));
        }
        assert_eq!(Xun::from_index(index), XUNS[usize::from(index % 6)]);
        match NAYIN_CASES.get(usize::from(index)) {
            Some(case) => assert_eq!(Nayin::try_from(index), Ok(case.0)),
            None => assert_eq!(
                Nayin::try_from(index),
                Err(InvalidIndex {
                    index,
                    upper_bound: 30
                })
            ),
        }
    }
}

#[test]
fn xun_steps_handle_extreme_offsets_and_directed_distances() {
    let deltas = [
        i32::MIN,
        i32::MIN + 1,
        -7,
        -6,
        -1,
        0,
        1,
        6,
        7,
        i32::MAX - 1,
        i32::MAX,
    ];
    for (index, xun) in XUNS.into_iter().enumerate() {
        for delta in deltas {
            let target = (i64::try_from(index).unwrap() + i64::from(delta)).rem_euclid(6);
            assert_eq!(xun.offset(delta), XUNS[usize::try_from(target).unwrap()]);
        }
        for (target_index, target) in XUNS.into_iter().enumerate() {
            let distance = (target_index + 6 - index) % 6;
            assert_eq!(usize::from(xun.distance_to(target)), distance);
        }
    }
}

#[cfg(feature = "serde")]
mod encoding {
    use super::{NAYIN_CASES, Nayin, XUNS, Xun};
    use serde::{Serialize, de::DeserializeOwned};
    use serde_json::{Value, json};

    fn check<T: Serialize + DeserializeOwned + PartialEq + core::fmt::Debug>(
        values: &[T],
        wires: &Value,
    ) {
        let wires = wires.as_array().unwrap();
        assert_eq!(values.len(), wires.len());
        for (value, wire) in values.iter().zip(wires) {
            assert_eq!(serde_json::to_value(value).unwrap(), *wire);
            assert_eq!(serde_json::from_value::<T>(wire.clone()).unwrap(), *value);
        }
    }

    #[test]
    fn identities_keep_their_frozen_string_codes() {
        let fixture: Value =
            serde_json::from_str(include_str!("fixtures/identities-v1.json")).unwrap();
        check(&XUNS, &fixture["Xun"]);
        check(&NAYIN_CASES.map(|case| case.0), &fixture["Nayin"]);
    }

    #[test]
    fn identities_reject_unknown_codes_wrong_shapes_and_display_aliases() {
        for value in [
            json!(null),
            json!(0),
            json!(255),
            json!([]),
            json!({}),
            json!(""),
            json!("unknown"),
        ] {
            assert!(serde_json::from_value::<Xun>(value.clone()).is_err());
            assert!(serde_json::from_value::<Nayin>(value).is_err());
        }
        for value in ["JiaChou", "甲子旬", "jiazi", "HaiZhongJin"] {
            assert!(serde_json::from_value::<Xun>(json!(value)).is_err());
        }
        for value in ["JiaZi", "海中金", "Metal", "JingQuanShui", "QuanZhongShui "] {
            assert!(serde_json::from_value::<Nayin>(json!(value)).is_err());
        }
    }
}
