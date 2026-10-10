//! 已复现故障的回归测试；必须在 debug 和 release 执行。
use matharts_core::{Branch, CyclicSequence, Ganzhi, GrowthPhase, Stem};

fn check_extreme_offsets<T: CyclicSequence>() {
    for index in 0..T::MODULUS.get() {
        for delta in [
            i32::MIN,
            i32::MIN + 1,
            -121,
            -1,
            0,
            1,
            121,
            i32::MAX - 1,
            i32::MAX,
        ] {
            let expected =
                (i64::from(index) + i64::from(delta)).rem_euclid(i64::from(T::MODULUS.get()));
            assert_eq!(
                i64::from(T::from_index(index).offset(delta).index()),
                expected
            );
        }
    }
}

#[test]
fn all_cycle_types_handle_extreme_offsets() {
    check_extreme_offsets::<Stem>();
    check_extreme_offsets::<Branch>();
    check_extreme_offsets::<GrowthPhase>();
    check_extreme_offsets::<Ganzhi>();
}

#[test]
fn exactly_sixty_pairs_are_valid_and_roundtrip() {
    let mut valid = 0;
    for s in 0..10 {
        for b in 0..12 {
            let stem = Stem::from_index(s);
            let branch = Branch::from_index(b);
            let value = Ganzhi::new(stem, branch);
            assert_eq!(value.is_some(), s % 2 == b % 2);
            if let Some(value) = value {
                assert_eq!(value.stem(), stem);
                assert_eq!(value.branch(), branch);
                assert_eq!(Ganzhi::from_index(value.index()), value);
                valid += 1;
            }
        }
    }
    assert_eq!(valid, 60);
}

#[cfg(feature = "serde")]
#[test]
fn serde_validates_all_pairs() {
    use serde::Deserialize;
    use serde::de::value::{Error, MapDeserializer};

    let stems = [
        "Jia", "Yi", "Bing", "Ding", "Wu", "Ji", "Geng", "Xin", "Ren", "Gui",
    ];
    let branches = [
        "Zi", "Chou", "Yin", "Mao", "Chen", "Si", "Wu", "Wei", "Shen", "You", "Xu", "Hai",
    ];
    let mut accepted = 0;
    for (s, stem) in stems.into_iter().enumerate() {
        for (b, branch) in branches.into_iter().enumerate() {
            let value = Ganzhi::deserialize(MapDeserializer::<_, Error>::new(
                [("stem", stem), ("branch", branch)].into_iter(),
            ));
            assert_eq!(value.is_ok(), s % 2 == b % 2, "{stem}/{branch}");
            if let Ok(value) = value {
                assert_eq!(usize::from(value.stem().index()), s);
                assert_eq!(usize::from(value.branch().index()), b);
                assert_eq!(Ganzhi::from_index(value.index()), value);
                accepted += 1;
            }
        }
    }
    assert_eq!(accepted, 60);
}
