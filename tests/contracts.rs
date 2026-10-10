//! 严格构造、周期运算、固定属性及当前入口的公共契约。
use core::num::NonZeroU8;
use matharts_core::{
    Branch, CyclicSequence, Element, ElementRelation, Ganzhi, GrowthPhase, Stem, Xun, YinYang,
    checked_forward_distance, forward_distance,
};
use proptest::prelude::*;

const STEMS: [Stem; 10] = [
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

fn check_cycle_boundary<T: CyclicSequence + TryFrom<u8>>() {
    for index in 0..=u8::MAX {
        assert_eq!(T::try_from(index).is_ok(), index < T::MODULUS.get());
        assert_eq!(T::try_from_index(index).is_ok(), index < T::MODULUS.get());
        assert_eq!(T::from_index(index).index(), index % T::MODULUS.get());
    }
}

#[test]
fn strict_and_wrapping_construction_are_distinct() {
    check_cycle_boundary::<Stem>();
    check_cycle_boundary::<Branch>();
    check_cycle_boundary::<GrowthPhase>();
    check_cycle_boundary::<Ganzhi>();
    for index in 0..=u8::MAX {
        assert_eq!(Element::try_from(index).is_ok(), index < 5);
        assert_eq!(Element::from_index(index).index(), index % 5);
    }
}

#[test]
fn zero_period_is_rejected_and_distance_is_directed() {
    assert_eq!(checked_forward_distance(0, 1, 0), None);
    assert_eq!(checked_forward_distance(11, 0, 12), Some(1));
    assert_eq!(forward_distance(0, 11, NonZeroU8::new(12).unwrap()), 11);
    assert_eq!(checked_forward_distance(255, 0, 12), Some(9));
    assert_eq!(checked_forward_distance(0, 255, 1), Some(0));
}

#[test]
fn yin_yang_and_elements_match_fixed_attributes() {
    use Element::{Earth, Fire, Metal, Water, Wood};
    let stems: [Element; 10] = [
        Wood, Wood, Fire, Fire, Earth, Earth, Metal, Metal, Water, Water,
    ];
    let branches: [Element; 12] = [
        Water, Earth, Wood, Wood, Earth, Fire, Fire, Earth, Metal, Metal, Earth, Water,
    ];
    for (index, expected) in (0_u8..10).zip(stems) {
        assert_eq!(Stem::try_from(index).unwrap().element(), expected);
        assert_eq!(
            Stem::try_from(index).unwrap().yin_yang().is_yang(),
            index.is_multiple_of(2)
        );
    }
    for (index, expected) in (0_u8..12).zip(branches) {
        assert_eq!(Branch::try_from(index).unwrap().element(), expected);
        assert_eq!(
            Branch::try_from(index).unwrap().yin_yang().is_yang(),
            index.is_multiple_of(2)
        );
    }
    assert_eq!(YinYang::Yang.invert(), YinYang::Yin);
    assert_eq!(YinYang::Yin.invert(), YinYang::Yang);
    assert!(!YinYang::Yang.is_yin());
    assert!(YinYang::Yin.is_yin());
}

#[test]
fn all_element_relations_have_the_documented_direction() {
    use ElementRelation::{
        GeneratedBy as B, Generates as G, OvercomeBy as O, Overcomes as K, Same as S,
    };
    let expected: [[ElementRelation; 5]; 5] = [
        [S, G, K, O, B],
        [B, S, G, K, O],
        [O, B, S, G, K],
        [K, O, B, S, G],
        [G, K, O, B, S],
    ];
    for (i, row) in (0_u8..5).zip(expected) {
        for (j, relation) in (0_u8..5).zip(row) {
            assert_eq!(
                Element::try_from(i)
                    .unwrap()
                    .relation_to(Element::try_from(j).unwrap()),
                relation
            );
        }
    }
}

#[test]
fn element_convenience_methods_match_fixed_relations() {
    use Element::{Earth, Fire, Metal, Water, Wood};
    // 每行依次为自身、生我、我生、克我、我克，不以 relation_to 生成预期。
    let cases: [(Element, Element, Element, Element, Element); 5] = [
        (Wood, Water, Fire, Metal, Earth),
        (Fire, Wood, Earth, Water, Metal),
        (Earth, Fire, Metal, Wood, Water),
        (Metal, Earth, Water, Fire, Wood),
        (Water, Metal, Wood, Earth, Fire),
    ];
    for (element, generated_by, generates, overcome_by, overcomes) in cases {
        assert_eq!(element.generated_by(), generated_by, "{element:?}");
        assert_eq!(element.generates(), generates, "{element:?}");
        assert_eq!(element.overcome_by(), overcome_by, "{element:?}");
        assert_eq!(element.overcomes(), overcomes, "{element:?}");
    }
}

#[test]
fn stem_combinations_and_clashes_match_fixed_pairs() {
    use Stem::{Bing, Ding, Geng, Gui, Ji, Jia, Ren, Wu, Xin, Yi};
    let combinations: [Stem; 10] = [Ji, Geng, Xin, Ren, Gui, Jia, Yi, Bing, Ding, Wu];
    let clashes: [(Stem, Stem); 4] = [(Jia, Geng), (Yi, Xin), (Bing, Ren), (Ding, Gui)];
    for (stem, expected) in STEMS.into_iter().zip(combinations) {
        assert_eq!(
            stem.five_combination().partner_of(stem),
            Some(expected),
            "{stem:?}"
        );
        for target in STEMS {
            let expected = clashes.contains(&(stem, target)) || clashes.contains(&(target, stem));
            assert_eq!(
                stem.is_clashing_with(target),
                expected,
                "{stem:?}, {target:?}"
            );
        }
    }
}

#[test]
fn branch_opposites_match_fixed_pairs() {
    use Branch::{Chen, Chou, Hai, Mao, Shen, Si, Wei, Wu, Xu, Yin, You, Zi};
    let opposites: [Branch; 12] = [Wu, Wei, Shen, You, Xu, Hai, Zi, Chou, Yin, Mao, Chen, Si];
    for (branch, opposite) in BRANCHES.into_iter().zip(opposites) {
        assert_eq!(branch.opposite(), opposite, "{branch:?}");
    }
}

#[test]
fn distance_to_preserves_direction_for_every_pair() {
    for (left_index, left) in (0_u8..10).zip(STEMS) {
        for (right_index, right) in (0_u8..10).zip(STEMS) {
            let expected = (i16::from(left_index) - i16::from(right_index)).rem_euclid(10);
            assert_eq!(
                i16::from(right.distance_to(left)),
                expected,
                "{right:?} to {left:?}"
            );
        }
    }
    for (left_index, left) in (0_u8..12).zip(BRANCHES) {
        for (right_index, right) in (0_u8..12).zip(BRANCHES) {
            let expected = (i16::from(left_index) - i16::from(right_index)).rem_euclid(12);
            assert_eq!(
                i16::from(right.distance_to(left)),
                expected,
                "{right:?} to {left:?}"
            );
        }
    }
}

fn check_integer_steps<T>(values: &[T], delta: i32)
where
    T: CyclicSequence
        + core::fmt::Debug
        + core::ops::Add<i32, Output = T>
        + core::ops::Sub<i32, Output = T>
        + core::ops::AddAssign<i32>
        + core::ops::SubAssign<i32>,
{
    let period = i64::try_from(values.len()).unwrap();
    for (index, &value) in values.iter().enumerate() {
        let index = i64::try_from(index).unwrap();
        let forward =
            values[usize::try_from((index + i64::from(delta)).rem_euclid(period)).unwrap()];
        let backward =
            values[usize::try_from((index - i64::from(delta)).rem_euclid(period)).unwrap()];
        assert_eq!(value.offset(delta), forward);
        assert_eq!(value + delta, forward);
        assert_eq!(value - delta, backward);
        let mut assigned = value;
        assigned += delta;
        assert_eq!(assigned, forward);
        assigned -= delta;
        assert_eq!(assigned, value);
        assigned -= delta;
        assert_eq!(assigned, backward);
        assigned += delta;
        assert_eq!(assigned, value);
    }
}

fn check_all_integer_steps(delta: i32) {
    check_integer_steps(&STEMS, delta);
    check_integer_steps(&BRANCHES, delta);
    check_integer_steps(
        &[
            GrowthPhase::ChangSheng,
            GrowthPhase::MuYu,
            GrowthPhase::GuanDai,
            GrowthPhase::LinGuan,
            GrowthPhase::DiWang,
            GrowthPhase::Shuai,
            GrowthPhase::Bing,
            GrowthPhase::Si,
            GrowthPhase::Mu,
            GrowthPhase::Jue,
            GrowthPhase::Tai,
            GrowthPhase::Yang,
        ],
        delta,
    );
    check_integer_steps(
        &[
            Xun::JiaZi,
            Xun::JiaXu,
            Xun::JiaShen,
            Xun::JiaWu,
            Xun::JiaChen,
            Xun::JiaYin,
        ],
        delta,
    );
    // 独立的两个周期计数器构造合法甲子序列；不调用被测步进生成预期。
    let ganzhi: [Ganzhi; 60] =
        core::array::from_fn(|i| Ganzhi::new(STEMS[i % 10], BRANCHES[i % 12]).unwrap());
    check_integer_steps(&ganzhi, delta);
}

#[test]
fn all_five_cycle_types_add_subtract_and_assign_at_signed_boundaries() {
    for delta in [
        i32::MIN,
        i32::MIN + 1,
        -121,
        -60,
        -12,
        -10,
        -6,
        -1,
        0,
        1,
        6,
        10,
        12,
        60,
        121,
        i32::MAX - 1,
        i32::MAX,
    ] {
        check_all_integer_steps(delta);
    }
}

proptest! {
    #[test]
    fn all_five_cycle_types_handle_arbitrary_signed_steps(delta in any::<i32>()) {
        check_all_integer_steps(delta);
    }
}

#[test]
fn all_sixty_values_have_consistent_xun_membership() {
    use Branch::{Chen, Chou, Hai, Mao, Shen, Si, Wei, Wu, Xu, Yin, You, Zi};
    let leaders: [Branch; 6] = [Zi, Xu, Shen, Wu, Chen, Yin];
    let missing: [(Branch, Branch); 6] = [
        (Xu, Hai),
        (Shen, You),
        (Wu, Wei),
        (Chen, Si),
        (Yin, Mao),
        (Zi, Chou),
    ];
    for index in 0_u8..60 {
        let value = Ganzhi::try_from(index).unwrap();
        let xun = usize::from(index / 10);
        assert_eq!(value.xun().leader().branch(), leaders[xun]);
        assert_eq!(value.xun().void_branches(), missing[xun]);
        for b in 0_u8..12 {
            let branch = Branch::try_from(b).unwrap();
            assert_eq!(
                value.xun().is_void_branch(branch),
                branch == missing[xun].0 || branch == missing[xun].1
            );
        }
    }
}

proptest! {
    #[test]
    fn arbitrary_signed_offsets_match_wide_integer_math(index in 0_u8..60, delta in any::<i32>()) {
        let actual = Ganzhi::try_from(index).unwrap().offset(delta);
        let expected = (i64::from(index) + i64::from(delta)).rem_euclid(60);
        prop_assert_eq!(i64::from(actual.index()), expected);
    }
}

#[test]
fn custom_cycle_uses_nonzero_period_and_strict_indices() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Weekday(u8);

    impl CyclicSequence for Weekday {
        const MODULUS: NonZeroU8 = NonZeroU8::new(7).unwrap();
        fn from_index(idx: u8) -> Self {
            Self(idx % Self::MODULUS.get())
        }
        fn index(self) -> u8 {
            self.0
        }
    }

    assert_eq!(Weekday::from_index(8), Weekday(1));
    assert_eq!(Weekday(6).offset(i32::MAX), Weekday(0));
    assert_eq!(Weekday(0).offset(i32::MIN), Weekday(5));
    assert_eq!(Weekday(6).distance_to(Weekday(1)), 2);
    assert_eq!(Weekday::try_from_index(6).unwrap(), Weekday(6));
    assert!(Weekday::try_from_index(7).is_err());
}
