//! 八卦与六爻结构的独立固定预期及全集验收。

use YinYang::{Yang, Yin};
use matharts_core::{Hexagram, HexagramPosition, InvalidIndex, Trigram, TrigramPosition, YinYang};
use std::collections::HashSet;

// 不从生产查询生成三爻预期；排列仅用作 bits 升序的技术遍历。
const CASES: [(Trigram, &str, [YinYang; 3]); 8] = [
    (Trigram::Kun, "坤", [Yin, Yin, Yin]),
    (Trigram::Zhen, "震", [Yang, Yin, Yin]),
    (Trigram::Kan, "坎", [Yin, Yang, Yin]),
    (Trigram::Dui, "兑", [Yang, Yang, Yin]),
    (Trigram::Gen, "艮", [Yin, Yin, Yang]),
    (Trigram::Li, "离", [Yang, Yin, Yang]),
    (Trigram::Xun, "巽", [Yin, Yang, Yang]),
    (Trigram::Qian, "乾", [Yang, Yang, Yang]),
];

// Positions are an independent bounded domain, not expected values from production ALL.
const TRIGRAM_POSITIONS: [TrigramPosition; 3] = [
    TrigramPosition::First,
    TrigramPosition::Second,
    TrigramPosition::Third,
];
const HEXAGRAM_POSITIONS: [HexagramPosition; 6] = [
    HexagramPosition::First,
    HexagramPosition::Second,
    HexagramPosition::Third,
    HexagramPosition::Fourth,
    HexagramPosition::Fifth,
    HexagramPosition::Sixth,
];

// 编译时调用，避免只在运行时验证后误称接口支持 const。
const TAI: Hexagram = Hexagram::from_trigrams(Trigram::Qian, Trigram::Kun);
const CONST_LINES: [YinYang; 6] = TAI
    .reverse_lines()
    .complement()
    .with_line(HexagramPosition::First, Yin)
    .toggle_line(HexagramPosition::First)
    .lines();
const CONST_TRIGRAM: Trigram = Trigram::from_lines([Yang, Yin, Yin])
    .reverse_lines()
    .complement()
    .with_line(TrigramPosition::First, Yang)
    .toggle_line(TrigramPosition::Third);
const CONST_BITS: Result<Hexagram, InvalidIndex> = Hexagram::try_from_bits(7);
const CONST_TRI_BITS: Result<Trigram, InvalidIndex> = Trigram::try_from_bits(1);

#[test]
fn fixed_shapes_and_const_construction() {
    assert_eq!(Trigram::ALL, CASES.map(|c| c.0));
    for (bits, (value, name, lines)) in (0_u8..8).zip(CASES) {
        assert_eq!(value.name(), name);
        assert_eq!(value.lines(), lines);
        assert_eq!(value.bits(), bits);
        assert_eq!(Trigram::from_lines(lines), value);
        assert_eq!(Trigram::try_from_bits(bits), Ok(value));
    }
    assert_eq!(CONST_BITS, Ok(TAI));
    assert_eq!(CONST_TRI_BITS, Ok(Trigram::Zhen));
    assert_eq!(CONST_LINES, [Yang, Yang, Yang, Yin, Yin, Yin]);
    assert_eq!(CONST_TRIGRAM, Trigram::Qian);
    // 已有阴阳编码不可为了新 bits 约定而倒置。
    assert_eq!(YinYang::Yang as u8, 0);
    assert_eq!(YinYang::Yin as u8, 1);
    assert_eq!(TAI.bits(), 7);
    assert_eq!(TAI.reverse_lines().bits(), 56);
    assert_eq!(Trigram::Zhen.reverse_lines(), Trigram::Gen);
    assert_eq!(Trigram::Zhen.complement(), Trigram::Xun);
}

#[test]
fn sixty_four_combinations_have_independent_array_shapes() {
    let mut seen = HashSet::new();
    let mut seen_bits = [false; 64];
    for (lower, _, lower_lines) in CASES {
        for (upper, _, upper_lines) in CASES {
            let lines = [
                lower_lines[0],
                lower_lines[1],
                lower_lines[2],
                upper_lines[0],
                upper_lines[1],
                upper_lines[2],
            ];
            let value = Hexagram::from_trigrams(lower, upper);
            assert!(seen.insert(value));
            assert_eq!(value.lower(), lower);
            assert_eq!(value.upper(), upper);
            assert_eq!(value.lines(), lines);
            assert_eq!(Hexagram::from_lines(lines), value);
            // 数组参考编码，不调用两个三爻卦的 bits() 合成预期。
            let expected = lines.iter().enumerate().fold(0_u8, |bits, (i, line)| {
                bits + if *line == Yang {
                    2_u8.pow(u32::try_from(i).unwrap())
                } else {
                    0
                }
            });
            assert_eq!(value.bits(), expected);
            assert!(!seen_bits[usize::from(expected)]);
            seen_bits[usize::from(expected)] = true;
            assert_eq!(Hexagram::ALL[usize::from(expected)], value);
            assert_eq!(Hexagram::try_from_bits(expected), Ok(value));
        }
    }
    assert_eq!(seen.len(), 64);
    assert_eq!(Hexagram::ALL.into_iter().collect::<HashSet<_>>(), seen);
    assert!(seen_bits.into_iter().all(|v| v));
}

#[test]
fn all_u8_inputs_are_checked_without_wraparound() {
    assert_eq!(TrigramPosition::ALL, TRIGRAM_POSITIONS);
    assert_eq!(HexagramPosition::ALL, HEXAGRAM_POSITIONS);
    for input in 0..=u8::MAX {
        let index = usize::from(input);
        assert_eq!(
            Trigram::try_from_bits(input),
            CASES.get(index).map(|c| c.0).ok_or(InvalidIndex {
                index: input,
                upper_bound: 8
            })
        );
        assert_eq!(
            Hexagram::try_from_bits(input),
            Hexagram::ALL.get(index).copied().ok_or(InvalidIndex {
                index: input,
                upper_bound: 64
            })
        );
        assert_eq!(
            TrigramPosition::try_from(input),
            TRIGRAM_POSITIONS.get(index).copied().ok_or(InvalidIndex {
                index: input,
                upper_bound: 3
            })
        );
        assert_eq!(
            HexagramPosition::try_from(input),
            HEXAGRAM_POSITIONS.get(index).copied().ok_or(InvalidIndex {
                index: input,
                upper_bound: 6
            })
        );
    }
    for (expected, value) in (0_u8..3).zip(TRIGRAM_POSITIONS) {
        assert_eq!(value.index(), expected);
    }
    for (expected, value) in (0_u8..6).zip(HEXAGRAM_POSITIONS) {
        assert_eq!(value.index(), expected);
    }
}

#[test]
fn all_line_queries_updates_and_transformations_match_arrays() {
    for (value, _, lines) in CASES {
        let mut reversed = lines;
        reversed.reverse();
        assert_eq!(value.reverse_lines().lines(), reversed);
        assert_eq!(value.complement().lines(), lines.map(YinYang::invert));
        assert_eq!(value.reverse_lines().reverse_lines(), value);
        assert_eq!(value.complement().complement(), value);
        for p in TRIGRAM_POSITIONS {
            let i = usize::from(p.index());
            assert_eq!(value.line(p), lines[i]);
            let mut changed = lines;
            changed[i] = changed[i].invert();
            assert_eq!(value.toggle_line(p).lines(), changed);
            assert_eq!(value.toggle_line(p).toggle_line(p), value);
            for polarity in [Yin, Yang] {
                changed[i] = polarity;
                assert_eq!(value.with_line(p, polarity).lines(), changed);
                for q in TRIGRAM_POSITIONS {
                    if p != q {
                        assert_eq!(
                            value.with_line(p, polarity).with_line(q, polarity.invert()),
                            value.with_line(q, polarity.invert()).with_line(p, polarity)
                        );
                    }
                }
            }
        }
    }
    for value in Hexagram::ALL {
        let lines = value.lines();
        let mut reversed = lines;
        reversed.reverse();
        assert_eq!(value.reverse_lines().lines(), reversed);
        assert_eq!(value.complement().lines(), lines.map(YinYang::invert));
        assert_eq!(value.reverse_lines().reverse_lines(), value);
        assert_eq!(value.complement().complement(), value);
        for p in HEXAGRAM_POSITIONS {
            let i = usize::from(p.index());
            assert_eq!(value.line(p), lines[i]);
            let mut changed = lines;
            changed[i] = changed[i].invert();
            assert_eq!(value.toggle_line(p).lines(), changed);
            assert_eq!(value.toggle_line(p).toggle_line(p), value);
            for polarity in [Yin, Yang] {
                changed[i] = polarity;
                assert_eq!(value.with_line(p, polarity).lines(), changed);
                for q in HEXAGRAM_POSITIONS {
                    if p != q {
                        assert_eq!(
                            value.with_line(p, polarity).with_line(q, polarity.invert()),
                            value.with_line(q, polarity.invert()).with_line(p, polarity)
                        );
                    }
                }
            }
        }
    }
}
