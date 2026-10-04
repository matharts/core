//! 具名规则的固定回归表；锁定选用定义，不宣称适用于所有术数。
use matharts_core::{
    Branch, CyclicRing, Element, God, Growth, SexagenaryCycle, Stem, derive_hour_stem,
    derive_month_stem,
};

#[test]
fn ten_gods_cover_all_hundred_ordered_pairs() {
    use God::{
        BiJian as B, JieCai as J, PianCai as C, PianYin as Y, QiSha as Q, ShangGuan as H,
        ShiShen as S, ZhengCai as D, ZhengGuan as G, ZhengYin as Z,
    };
    let table = [
        [B, J, S, H, C, D, Q, G, Y, Z],
        [J, B, H, S, D, C, G, Q, Z, Y],
        [Y, Z, B, J, S, H, C, D, Q, G],
        [Z, Y, J, B, H, S, D, C, G, Q],
        [Q, G, Y, Z, B, J, S, H, C, D],
        [G, Q, Z, Y, J, B, H, S, D, C],
        [C, D, Q, G, Y, Z, B, J, S, H],
        [D, C, G, Q, Z, Y, J, B, H, S],
        [S, H, C, D, Q, G, Y, Z, B, J],
        [H, S, D, C, G, Q, Z, Y, J, B],
    ];
    for (a, row) in (0_u8..10).zip(table) {
        for (b, expected) in (0_u8..10).zip(row) {
            assert_eq!(
                Stem::try_from(a)
                    .unwrap()
                    .ten_god_of(Stem::try_from(b).unwrap()),
                expected
            );
        }
    }
}

#[test]
fn named_growth_rule_covers_all_hundred_twenty_positions() {
    let table = [
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0],
        [6, 5, 4, 3, 2, 1, 0, 11, 10, 9, 8, 7],
        [10, 11, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        [9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 11, 10],
        [10, 11, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        [9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 11, 10],
        [7, 8, 9, 10, 11, 0, 1, 2, 3, 4, 5, 6],
        [0, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1],
        [4, 5, 6, 7, 8, 9, 10, 11, 0, 1, 2, 3],
        [3, 2, 1, 0, 11, 10, 9, 8, 7, 6, 5, 4],
    ];
    for (s, row) in (0_u8..10).zip(table) {
        for (b, expected) in (0_u8..12).zip(row) {
            let stem = Stem::try_from(s).unwrap();
            let branch = Branch::try_from(b).unwrap();
            assert_eq!(
                stem.growth_phase_at(branch),
                Growth::try_from(expected).unwrap()
            );
        }
    }
}

#[test]
fn stem_derivations_cover_year_day_and_branch_pairs() {
    // 每行对应年干/日干的一组五合；列严格按子至亥，不从被测算法生成。
    let month = [
        [2, 3, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1],
        [4, 5, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3],
        [6, 7, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5],
        [8, 9, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7],
        [0, 1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
    ];
    let hour = [
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1],
        [2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3],
        [4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5],
        [6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7],
        [8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
    ];
    for s in 0_u8..10 {
        for b in 0_u8..12 {
            let stem = Stem::try_from(s).unwrap();
            let branch = Branch::try_from(b).unwrap();
            assert_eq!(
                derive_month_stem(stem, branch).index(),
                month[usize::from(s % 5)][usize::from(b)]
            );
            assert_eq!(
                derive_hour_stem(stem, branch).index(),
                hour[usize::from(s % 5)][usize::from(b)]
            );
        }
    }
}

#[test]
fn every_nayin_pair_has_the_frozen_element() {
    use Element::{Earth as E, Fire as F, Metal as M, Water as A, Wood as W};
    let expected = [
        M, F, W, E, M, F, A, E, M, W, A, E, F, W, A, M, F, W, E, M, F, A, E, M, W, A, E, F, W, A,
    ];
    for (slot, element) in (0_u8..30).zip(expected) {
        for i in [slot * 2, slot * 2 + 1] {
            let value = SexagenaryCycle::try_from(i).unwrap();
            assert_eq!(value.nayin().element(), element);
        }
    }
}

#[test]
fn all_branch_pairs_respect_distinct_relation_contracts() {
    let harms = [(0, 7), (1, 6), (2, 5), (3, 4), (8, 11), (9, 10)];
    let breaks = [(0, 9), (3, 6), (4, 1), (5, 8), (2, 11), (7, 10)];
    let punishments = [
        (0, 3),
        (3, 0),
        (2, 5),
        (5, 8),
        (8, 2),
        (1, 10),
        (10, 7),
        (7, 1),
        (4, 4),
        (6, 6),
        (9, 9),
        (11, 11),
    ];
    let combinations = [(0, 1), (2, 11), (3, 10), (4, 9), (5, 8), (6, 7)];
    for a in 0_u8..12 {
        for b in 0_u8..12 {
            let x = Branch::try_from(a).unwrap();
            let y = Branch::try_from(b).unwrap();
            let symmetric = |pairs: &[(u8, u8)]| pairs.contains(&(a, b)) || pairs.contains(&(b, a));
            assert_eq!(x.is_harming(y), symmetric(&harms));
            assert_eq!(x.is_breaking(y), symmetric(&breaks));
            assert_eq!(x.is_punishing(y), punishments.contains(&(a, b)));
            assert_eq!(
                x.six_combination().partner_of(x) == Some(y),
                symmetric(&combinations)
            );
            assert_eq!(x.is_clashing_with(y), a.abs_diff(b) == 6);
        }
    }
}

#[test]
fn hidden_stems_and_combination_elements_match_selected_tables() {
    use Stem::{Bing, Ding, Geng, Gui, Ji, Jia, Ren, Wu, Xin, Yi};
    let expected = [
        (Gui, None, None),
        (Ji, Some(Gui), Some(Xin)),
        (Jia, Some(Bing), Some(Wu)),
        (Yi, None, None),
        (Wu, Some(Yi), Some(Gui)),
        (Bing, Some(Geng), Some(Wu)),
        (Ding, Some(Ji), None),
        (Ji, Some(Ding), Some(Yi)),
        (Geng, Some(Ren), Some(Wu)),
        (Xin, None, None),
        (Wu, Some(Xin), Some(Ding)),
        (Ren, Some(Jia), None),
    ];
    for (i, (primary, secondary, tertiary)) in (0_u8..12).zip(expected) {
        let value = Branch::try_from(i).unwrap().hidden_stems();
        assert_eq!(
            (value.primary, value.secondary, value.tertiary),
            (primary, secondary, tertiary)
        );
    }
    let elements = [
        Element::Earth,
        Element::Metal,
        Element::Water,
        Element::Wood,
        Element::Fire,
    ];
    for i in 0_u8..10 {
        assert_eq!(
            Stem::try_from(i).unwrap().five_combination().element(),
            elements[usize::from(i % 5)]
        );
    }
}
