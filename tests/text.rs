//! 中文文本契约：独立预期、严格边界与全部干支配对。
use core::{error::Error, fmt, fmt::Write};
use matharts_core::{
    Branch, CyclicRing, Element, InvalidGanzhi, Nayin, ParseError, Primitive, SexagenaryCycle,
    Stem, Trigram, Xun,
};
use proptest::prelude::*;

const STEMS: [(Stem, &str); 10] = [
    (Stem::Jia, "甲"),
    (Stem::Yi, "乙"),
    (Stem::Bing, "丙"),
    (Stem::Ding, "丁"),
    (Stem::Wu, "戊"),
    (Stem::Ji, "己"),
    (Stem::Geng, "庚"),
    (Stem::Xin, "辛"),
    (Stem::Ren, "壬"),
    (Stem::Gui, "癸"),
];

const BRANCHES: [(Branch, &str); 12] = [
    (Branch::Zi, "子"),
    (Branch::Chou, "丑"),
    (Branch::Yin, "寅"),
    (Branch::Mao, "卯"),
    (Branch::Chen, "辰"),
    (Branch::Si, "巳"),
    (Branch::Wu, "午"),
    (Branch::Wei, "未"),
    (Branch::Shen, "申"),
    (Branch::You, "酉"),
    (Branch::Xu, "戌"),
    (Branch::Hai, "亥"),
];

const GANZHI: [&str; 60] = [
    "甲子", "乙丑", "丙寅", "丁卯", "戊辰", "己巳", "庚午", "辛未", "壬申", "癸酉", "甲戌", "乙亥",
    "丙子", "丁丑", "戊寅", "己卯", "庚辰", "辛巳", "壬午", "癸未", "甲申", "乙酉", "丙戌", "丁亥",
    "戊子", "己丑", "庚寅", "辛卯", "壬辰", "癸巳", "甲午", "乙未", "丙申", "丁酉", "戊戌", "己亥",
    "庚子", "辛丑", "壬寅", "癸卯", "甲辰", "乙巳", "丙午", "丁未", "戊申", "己酉", "庚戌", "辛亥",
    "壬子", "癸丑", "甲寅", "乙卯", "丙辰", "丁巳", "戊午", "己未", "庚申", "辛酉", "壬戌", "癸亥",
];

// 身份序列由测试显式定义；生产 ALL 与名称表不能共同生成预期。
const PRIMITIVES: [Primitive; 2] = [Primitive::Yang, Primitive::Yin];
const ELEMENTS: [Element; 5] = [
    Element::Wood,
    Element::Fire,
    Element::Earth,
    Element::Metal,
    Element::Water,
];
const TRIGRAMS: [Trigram; 8] = [
    Trigram::Kun,
    Trigram::Zhen,
    Trigram::Kan,
    Trigram::Dui,
    Trigram::Gen,
    Trigram::Li,
    Trigram::Xun,
    Trigram::Qian,
];
const XUNS: [Xun; 6] = [
    Xun::JiaZi,
    Xun::JiaXu,
    Xun::JiaShen,
    Xun::JiaWu,
    Xun::JiaChen,
    Xun::JiaYin,
];
const NAYIN: [Nayin; 30] = [
    Nayin::HaiZhongJin,
    Nayin::LuZhongHuo,
    Nayin::DaLinMu,
    Nayin::LuPangTu,
    Nayin::JianFengJin,
    Nayin::ShanTouHuo,
    Nayin::JianXiaShui,
    Nayin::ChengTouTu,
    Nayin::BaiLaJin,
    Nayin::YangLiuMu,
    Nayin::QuanZhongShui,
    Nayin::WuShangTu,
    Nayin::PiLiHuo,
    Nayin::SongBaiMu,
    Nayin::ChangLiuShui,
    Nayin::ShaZhongJin,
    Nayin::ShanXiaHuo,
    Nayin::PingDiMu,
    Nayin::BiShangTu,
    Nayin::JinBoJin,
    Nayin::FuDengHuo,
    Nayin::TianHeShui,
    Nayin::DaYiTu,
    Nayin::ChaiChuanJin,
    Nayin::SangZheMu,
    Nayin::DaXiShui,
    Nayin::ShaZhongTu,
    Nayin::TianShangHuo,
    Nayin::ShiLiuMu,
    Nayin::DaHaiShui,
];

#[test]
fn all_stems_and_branches_match_independent_chinese_names() {
    for (value, text) in STEMS {
        assert_eq!(value.to_string(), text);
        assert_eq!(text.parse::<Stem>(), Ok(value));
        assert_eq!(value.to_string().parse::<Stem>(), Ok(value));
    }
    for (value, text) in BRANCHES {
        assert_eq!(value.to_string(), text);
        assert_eq!(text.parse::<Branch>(), Ok(value));
        assert_eq!(value.to_string().parse::<Branch>(), Ok(value));
    }
}

#[test]
fn all_sixty_ganzhi_match_independent_chinese_names() {
    for (index, text) in (0_u8..60).zip(GANZHI) {
        let value = SexagenaryCycle::try_from(index).unwrap();
        assert_eq!(value.to_string(), text);
        assert_eq!(text.parse::<SexagenaryCycle>(), Ok(value));
        assert_eq!(value.to_string().parse::<SexagenaryCycle>(), Ok(value));
    }
}

#[test]
fn all_120_stem_branch_texts_accept_only_the_sixty_named_pairs() {
    let mut accepted = 0;
    let mut rejected = 0;
    for (stem, stem_text) in STEMS {
        for (branch, branch_text) in BRANCHES {
            let text = format!("{stem_text}{branch_text}");
            let parsed = text.parse::<SexagenaryCycle>();
            if let Some(index) = GANZHI.iter().position(|expected| *expected == text) {
                let value = parsed.unwrap();
                assert_eq!(usize::from(value.index()), index);
                assert_eq!((value.stem(), value.branch()), (stem, branch));
                accepted += 1;
            } else {
                assert_eq!(parsed, Err(ParseError::InvalidGanzhi(InvalidGanzhi)));
                rejected += 1;
            }
        }
    }
    assert_eq!((accepted, rejected), (60, 60));
}

#[test]
fn aliases_whitespace_and_extra_characters_are_rejected() {
    for text in ["", "Jia", "jia", "0", "子", "甲子", "甲甲", "假", "😀"] {
        assert_eq!(text.parse::<Stem>(), Err(ParseError::InvalidStem));
    }
    for text in ["", "Zi", "zi", "0", "甲", "甲子", "子子", "醜", "😀"] {
        assert_eq!(text.parse::<Branch>(), Err(ParseError::InvalidBranch));
    }
    for padding in [
        " ", "\t", "\n", "\r", "\u{a0}", "\u{3000}", "\u{200b}", "\0",
    ] {
        for (_, text) in STEMS {
            for input in [format!("{padding}{text}"), format!("{text}{padding}")] {
                assert_eq!(input.parse::<Stem>(), Err(ParseError::InvalidStem));
            }
        }
        for (_, text) in BRANCHES {
            for input in [format!("{padding}{text}"), format!("{text}{padding}")] {
                assert_eq!(input.parse::<Branch>(), Err(ParseError::InvalidBranch));
            }
        }
        for text in GANZHI {
            for input in [format!("{padding}{text}"), format!("{text}{padding}")] {
                assert_eq!(
                    input.parse::<SexagenaryCycle>(),
                    Err(ParseError::InvalidGanzhiFormat)
                );
            }
        }
    }
}

#[test]
fn ganzhi_errors_distinguish_format_names_and_invalid_pairs() {
    for text in [
        "",
        "甲",
        "甲子丑",
        "甲 子",
        "甲-子",
        "JiaZi",
        "甲子年",
        "甲子\u{301}",
    ] {
        assert_eq!(
            text.parse::<SexagenaryCycle>(),
            Err(ParseError::InvalidGanzhiFormat)
        );
    }
    // 这些输入都是两个 Unicode 字符；不能按固定字节位置切片。
    for text in ["子甲", "A子", "😀子", "𠀀子", "\u{301}子", "\0子", "😃😀"] {
        assert_eq!(
            text.parse::<SexagenaryCycle>(),
            Err(ParseError::InvalidStem)
        );
    }
    for text in ["甲甲", "甲A", "甲😀", "甲𠀀", "甲\u{301}", "甲\0", "甲醜"] {
        assert_eq!(
            text.parse::<SexagenaryCycle>(),
            Err(ParseError::InvalidBranch)
        );
    }
    assert_eq!(
        "甲丑".parse::<SexagenaryCycle>(),
        Err(ParseError::InvalidGanzhi(InvalidGanzhi))
    );
}

#[test]
fn parse_errors_are_typed_and_preserve_the_pair_error_source() {
    for error in [
        ParseError::InvalidElement,
        ParseError::InvalidPrimitive,
        ParseError::InvalidTrigram,
        ParseError::InvalidXun,
        ParseError::InvalidNayin,
        ParseError::InvalidStem,
        ParseError::InvalidBranch,
        ParseError::InvalidGanzhiFormat,
    ] {
        assert_ne!(error.to_string(), "");
        assert!(error.source().is_none());
    }
    let error = "甲丑".parse::<SexagenaryCycle>().unwrap_err();
    assert_eq!(error.to_string(), InvalidGanzhi.to_string());
    assert!(error.source().unwrap().is::<InvalidGanzhi>());
}

struct Buffer<const N: usize> {
    bytes: [u8; N],
    len: usize,
}

fn named_text_contract<T>(
    values: &[T],
    names: &serde_json::Value,
    name: fn(T) -> &'static str,
    error: ParseError,
    aliases: &[&str],
) where
    T: Copy + Eq + fmt::Debug + fmt::Display + core::str::FromStr<Err = ParseError>,
{
    let names = names.as_array().expect("必需名称数组");
    assert_eq!(names.len(), values.len(), "样本不能缺失或缩水");
    let mut seen = std::collections::BTreeSet::new();
    for (&value, expected) in values.iter().zip(names) {
        let expected = expected.as_str().expect("名称必须是字符串");
        assert_ne!(expected, "");
        assert!(seen.insert(expected), "重复名称不能替代缺失样本");
        assert_eq!(name(value), expected);
        assert_eq!(value.to_string(), expected);
        assert_eq!(expected.parse::<T>(), Ok(value));
        for padding in [" ", "\t", "\n", "\u{a0}", "\u{3000}", "\u{200b}", "\0"] {
            for input in [
                format!("{padding}{expected}"),
                format!("{expected}{padding}"),
            ] {
                assert_eq!(input.parse::<T>(), Err(error));
            }
        }
        for input in [
            format!("{value:?}"),
            format!("{expected}{expected}"),
            format!("{expected}\u{301}"),
        ] {
            assert_eq!(input.parse::<T>(), Err(error));
        }
        let mut buffer = Buffer {
            bytes: [0; 16],
            len: 0,
        };
        write!(buffer, "{value}").unwrap();
        assert_eq!(&buffer.bytes[..buffer.len], expected.as_bytes());
        let mut empty = Buffer { bytes: [], len: 0 };
        assert!(write!(empty, "{value}").is_err());
    }
    for input in ["", "0", "😀"].into_iter().chain(aliases.iter().copied()) {
        assert_eq!(input.parse::<T>(), Err(error));
    }
}

#[test]
fn named_values_match_all_51_frozen_names_and_reject_non_exact_input() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/text-values-v1.json")).unwrap();
    assert_eq!(fixture["version"], 1);
    assert_eq!(fixture.as_object().unwrap().len(), 6);
    assert_eq!(Primitive::ALL, PRIMITIVES);
    assert_eq!(Element::ALL, ELEMENTS);
    assert_eq!(Trigram::ALL, TRIGRAMS);
    assert_eq!(Xun::ALL, XUNS);
    assert_eq!(Nayin::ALL, NAYIN);
    named_text_contract(
        &PRIMITIVES,
        &fixture["primitive"],
        Primitive::name,
        ParseError::InvalidPrimitive,
        &["陰", "陽", "阴阳"],
    );
    named_text_contract(
        &ELEMENTS,
        &fixture["element"],
        Element::name,
        ParseError::InvalidElement,
        &["木行", "五行"],
    );
    named_text_contract(
        &TRIGRAMS,
        &fixture["trigram"],
        Trigram::name,
        ParseError::InvalidTrigram,
        &["乾卦", "兌", "離"],
    );
    named_text_contract(
        &XUNS,
        &fixture["xun"],
        Xun::name,
        ParseError::InvalidXun,
        &["甲子", "甲子旬旬"],
    );
    named_text_contract(
        &NAYIN,
        &fixture["nayin"],
        Nayin::name,
        ParseError::InvalidNayin,
        &["海中金命", "佛灯火", "覆燈火", "沙中土命"],
    );
}

#[cfg(feature = "serde")]
#[test]
fn chinese_text_names_do_not_become_serde_codes() {
    fn rejects<T: serde::de::DeserializeOwned>(names: &serde_json::Value) {
        for name in names.as_array().unwrap() {
            assert!(serde_json::from_value::<T>(name.clone()).is_err());
        }
    }
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/text-values-v1.json")).unwrap();
    rejects::<Primitive>(&fixture["primitive"]);
    rejects::<Element>(&fixture["element"]);
    rejects::<Trigram>(&fixture["trigram"]);
    rejects::<Xun>(&fixture["xun"]);
    rejects::<Nayin>(&fixture["nayin"]);
}

impl<const N: usize> fmt::Write for Buffer<N> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.len + text.len();
        let output = self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?;
        output.copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}

#[test]
fn display_works_with_stack_buffers_and_propagates_writer_errors() {
    let mut buffer = Buffer {
        bytes: [0; 6],
        len: 0,
    };
    let value = SexagenaryCycle::new(Stem::Jia, Branch::Zi).unwrap();
    write!(buffer, "{value}").unwrap();
    assert_eq!(&buffer.bytes[..buffer.len], "甲子".as_bytes());
    let mut short = Buffer {
        bytes: [0; 3],
        len: 0,
    };
    assert!(write!(short, "{value}").is_err());
    let mut empty = Buffer { bytes: [], len: 0 };
    assert!(write!(empty, "{}", Stem::Jia).is_err());
    assert!(write!(empty, "{}", Branch::Zi).is_err());
}

proptest! {
    #[test]
    fn arbitrary_utf8_accepts_only_exact_names(input in any::<String>()) {
        let expected_stem = STEMS.iter().find(|(_, name)| *name == input).map(|(value, _)| *value);
        let expected_branch = BRANCHES.iter().find(|(_, name)| *name == input).map(|(value, _)| *value);
        let expected_ganzhi = GANZHI.iter().position(|name| *name == input);
        prop_assert_eq!(input.parse::<Stem>().ok(), expected_stem);
        prop_assert_eq!(input.parse::<Branch>().ok(), expected_branch);
        prop_assert_eq!(input.parse::<SexagenaryCycle>().map(|value| usize::from(value.index())).ok(), expected_ganzhi);
    }
}
