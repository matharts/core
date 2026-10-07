//! 三十种纳音身份及其干支配对、五行；不包含取象吉凶规则。

use crate::{CyclicRing, Element, InvalidIndex, SexagenaryCycle};
use core::{fmt, str::FromStr};

/// 输出采用表的精确中文名称，与 [`FromStr`] 的输入一致。
impl fmt::Display for Nayin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// 只接受 [`Nayin::name`] 的采用用字，不接受文献异写或拼音代码。
impl FromStr for Nayin {
    type Err = crate::ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|value| value.name() == input)
            .ok_or(Self::Err::InvalidNayin)
    }
}

/// 三十种纳音身份，按甲子乙丑起的干支配对顺序编号。
///
/// 一种身份对应两个干支；五行相同不代表纳音身份相同。
/// 名称采用当前纳音表的显示用字；文献异写不作为编码别名。
/// 身份索引只作稳定编号，本类型不提供循环操作。
///
/// ```
/// use matharts_core::{Branch, Element, Nayin, SexagenaryCycle, Stem};
/// let jia_zi = SexagenaryCycle::new(Stem::Jia, Branch::Zi).unwrap();
/// assert_eq!(jia_zi.nayin(), Nayin::HaiZhongJin);
/// assert_eq!(jia_zi.nayin().element(), Element::Metal);
/// assert_eq!(Nayin::HaiZhongJin.name(), "海中金");
/// assert_eq!(Nayin::HaiZhongJin.ganzhi_pair()[0], jia_zi);
/// assert_eq!(Nayin::JianFengJin.element(), Element::Metal);
/// assert_ne!(Nayin::HaiZhongJin, Nayin::JianFengJin);
/// assert!(Nayin::try_from(30).is_err());
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[must_use]
pub enum Nayin {
    /// 甲子乙丑：海中金。
    HaiZhongJin = 0,
    /// 丙寅丁卯：炉中火。
    LuZhongHuo = 1,
    /// 戊辰己巳：大林木。
    DaLinMu = 2,
    /// 庚午辛未：路旁土。
    LuPangTu = 3,
    /// 壬申癸酉：剑锋金。
    JianFengJin = 4,
    /// 甲戌乙亥：山头火。
    ShanTouHuo = 5,
    /// 丙子丁丑：涧下水。
    JianXiaShui = 6,
    /// 戊寅己卯：城头土。
    ChengTouTu = 7,
    /// 庚辰辛巳：白蜡金。
    BaiLaJin = 8,
    /// 壬午癸未：杨柳木。
    YangLiuMu = 9,
    /// 甲申乙酉：泉中水。
    QuanZhongShui = 10,
    /// 丙戌丁亥：屋上土。
    WuShangTu = 11,
    /// 戊子己丑：霹雳火。
    PiLiHuo = 12,
    /// 庚寅辛卯：松柏木。
    SongBaiMu = 13,
    /// 壬辰癸巳：长流水。
    ChangLiuShui = 14,
    /// 甲午乙未：沙中金。
    ShaZhongJin = 15,
    /// 丙申丁酉：山下火。
    ShanXiaHuo = 16,
    /// 戊戌己亥：平地木。
    PingDiMu = 17,
    /// 庚子辛丑：壁上土。
    BiShangTu = 18,
    /// 壬寅癸卯：金箔金。
    JinBoJin = 19,
    /// 甲辰乙巳：覆灯火。
    FuDengHuo = 20,
    /// 丙午丁未：天河水。
    TianHeShui = 21,
    /// 戊申己酉：大驿土。
    DaYiTu = 22,
    /// 庚戌辛亥：钗钏金。
    ChaiChuanJin = 23,
    /// 壬子癸丑：桑柘木。
    SangZheMu = 24,
    /// 甲寅乙卯：大溪水。
    DaXiShui = 25,
    /// 丙辰丁巳：沙中土。
    ShaZhongTu = 26,
    /// 戊午己未：天上火。
    TianShangHuo = 27,
    /// 庚申辛酉：石榴木。
    ShiLiuMu = 28,
    /// 壬戌癸亥：大海水。
    DaHaiShui = 29,
}

impl Nayin {
    /// 全部三十种身份，顺序与六十甲子的相邻配对一致。
    pub const ALL: [Self; 30] = [
        Self::HaiZhongJin,
        Self::LuZhongHuo,
        Self::DaLinMu,
        Self::LuPangTu,
        Self::JianFengJin,
        Self::ShanTouHuo,
        Self::JianXiaShui,
        Self::ChengTouTu,
        Self::BaiLaJin,
        Self::YangLiuMu,
        Self::QuanZhongShui,
        Self::WuShangTu,
        Self::PiLiHuo,
        Self::SongBaiMu,
        Self::ChangLiuShui,
        Self::ShaZhongJin,
        Self::ShanXiaHuo,
        Self::PingDiMu,
        Self::BiShangTu,
        Self::JinBoJin,
        Self::FuDengHuo,
        Self::TianHeShui,
        Self::DaYiTu,
        Self::ChaiChuanJin,
        Self::SangZheMu,
        Self::DaXiShui,
        Self::ShaZhongTu,
        Self::TianShangHuo,
        Self::ShiLiuMu,
        Self::DaHaiShui,
    ];

    /// 配对顺序的零基索引，范围为 `0..30`，不表示强弱。
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// 精确中文显示与解析名称，不作为 Serde 机器代码。
    pub const fn name(self) -> &'static str {
        match self {
            Self::HaiZhongJin => "海中金",
            Self::LuZhongHuo => "炉中火",
            Self::DaLinMu => "大林木",
            Self::LuPangTu => "路旁土",
            Self::JianFengJin => "剑锋金",
            Self::ShanTouHuo => "山头火",
            Self::JianXiaShui => "涧下水",
            Self::ChengTouTu => "城头土",
            Self::BaiLaJin => "白蜡金",
            Self::YangLiuMu => "杨柳木",
            Self::QuanZhongShui => "泉中水",
            Self::WuShangTu => "屋上土",
            Self::PiLiHuo => "霹雳火",
            Self::SongBaiMu => "松柏木",
            Self::ChangLiuShui => "长流水",
            Self::ShaZhongJin => "沙中金",
            Self::ShanXiaHuo => "山下火",
            Self::PingDiMu => "平地木",
            Self::BiShangTu => "壁上土",
            Self::JinBoJin => "金箔金",
            Self::FuDengHuo => "覆灯火",
            Self::TianHeShui => "天河水",
            Self::DaYiTu => "大驿土",
            Self::ChaiChuanJin => "钗钏金",
            Self::SangZheMu => "桑柘木",
            Self::DaXiShui => "大溪水",
            Self::ShaZhongTu => "沙中土",
            Self::TianShangHuo => "天上火",
            Self::ShiLiuMu => "石榴木",
            Self::DaHaiShui => "大海水",
        }
    }

    /// 本纳音对应的五行，不包含强弱或吉凶判断。
    pub const fn element(self) -> Element {
        match self {
            Self::HaiZhongJin
            | Self::JianFengJin
            | Self::BaiLaJin
            | Self::ShaZhongJin
            | Self::JinBoJin
            | Self::ChaiChuanJin => Element::Metal,
            Self::LuZhongHuo
            | Self::ShanTouHuo
            | Self::PiLiHuo
            | Self::ShanXiaHuo
            | Self::FuDengHuo
            | Self::TianShangHuo => Element::Fire,
            Self::DaLinMu
            | Self::YangLiuMu
            | Self::SongBaiMu
            | Self::PingDiMu
            | Self::SangZheMu
            | Self::ShiLiuMu => Element::Wood,
            Self::LuPangTu
            | Self::ChengTouTu
            | Self::WuShangTu
            | Self::BiShangTu
            | Self::DaYiTu
            | Self::ShaZhongTu => Element::Earth,
            Self::JianXiaShui
            | Self::QuanZhongShui
            | Self::ChangLiuShui
            | Self::TianHeShui
            | Self::DaXiShui
            | Self::DaHaiShui => Element::Water,
        }
    }

    /// 对应的两个干支，按六十甲子顺序返回。
    pub fn ganzhi_pair(self) -> [SexagenaryCycle; 2] {
        let first = self.index() * 2;
        [
            SexagenaryCycle::from_index(first),
            SexagenaryCycle::from_index(first + 1),
        ]
    }
}

impl TryFrom<u8> for Nayin {
    type Error = InvalidIndex;

    fn try_from(index: u8) -> Result<Self, Self::Error> {
        Self::ALL
            .get(usize::from(index))
            .copied()
            .ok_or(InvalidIndex {
                index,
                upper_bound: 30,
            })
    }
}

// 可读格式只接受代码字符串；紧凑格式使用原生枚举模型。

#[cfg(feature = "serde")]
use serde::de::{self, EnumAccess, VariantAccess};

#[cfg(feature = "serde")]
const NAYIN_CODES: &[&str] = &[
    "HaiZhongJin",
    "LuZhongHuo",
    "DaLinMu",
    "LuPangTu",
    "JianFengJin",
    "ShanTouHuo",
    "JianXiaShui",
    "ChengTouTu",
    "BaiLaJin",
    "YangLiuMu",
    "QuanZhongShui",
    "WuShangTu",
    "PiLiHuo",
    "SongBaiMu",
    "ChangLiuShui",
    "ShaZhongJin",
    "ShanXiaHuo",
    "PingDiMu",
    "BiShangTu",
    "JinBoJin",
    "FuDengHuo",
    "TianHeShui",
    "DaYiTu",
    "ChaiChuanJin",
    "SangZheMu",
    "DaXiShui",
    "ShaZhongTu",
    "TianShangHuo",
    "ShiLiuMu",
    "DaHaiShui",
];

#[cfg(feature = "serde")]
struct NayinIdentifier;
#[cfg(feature = "serde")]
impl<'de> de::DeserializeSeed<'de> for NayinIdentifier {
    type Value = Nayin;
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        d.deserialize_identifier(self)
    }
}
#[cfg(feature = "serde")]
impl de::Visitor<'_> for NayinIdentifier {
    type Value = Nayin;
    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("variant identifier")
    }
    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        match value {
            "HaiZhongJin" => Ok(Nayin::HaiZhongJin),
            "LuZhongHuo" => Ok(Nayin::LuZhongHuo),
            "DaLinMu" => Ok(Nayin::DaLinMu),
            "LuPangTu" => Ok(Nayin::LuPangTu),
            "JianFengJin" => Ok(Nayin::JianFengJin),
            "ShanTouHuo" => Ok(Nayin::ShanTouHuo),
            "JianXiaShui" => Ok(Nayin::JianXiaShui),
            "ChengTouTu" => Ok(Nayin::ChengTouTu),
            "BaiLaJin" => Ok(Nayin::BaiLaJin),
            "YangLiuMu" => Ok(Nayin::YangLiuMu),
            "QuanZhongShui" => Ok(Nayin::QuanZhongShui),
            "WuShangTu" => Ok(Nayin::WuShangTu),
            "PiLiHuo" => Ok(Nayin::PiLiHuo),
            "SongBaiMu" => Ok(Nayin::SongBaiMu),
            "ChangLiuShui" => Ok(Nayin::ChangLiuShui),
            "ShaZhongJin" => Ok(Nayin::ShaZhongJin),
            "ShanXiaHuo" => Ok(Nayin::ShanXiaHuo),
            "PingDiMu" => Ok(Nayin::PingDiMu),
            "BiShangTu" => Ok(Nayin::BiShangTu),
            "JinBoJin" => Ok(Nayin::JinBoJin),
            "FuDengHuo" => Ok(Nayin::FuDengHuo),
            "TianHeShui" => Ok(Nayin::TianHeShui),
            "DaYiTu" => Ok(Nayin::DaYiTu),
            "ChaiChuanJin" => Ok(Nayin::ChaiChuanJin),
            "SangZheMu" => Ok(Nayin::SangZheMu),
            "DaXiShui" => Ok(Nayin::DaXiShui),
            "ShaZhongTu" => Ok(Nayin::ShaZhongTu),
            "TianShangHuo" => Ok(Nayin::TianShangHuo),
            "ShiLiuMu" => Ok(Nayin::ShiLiuMu),
            "DaHaiShui" => Ok(Nayin::DaHaiShui),
            _ => Err(E::unknown_variant(value, NAYIN_CODES)),
        }
    }
    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
        u8::try_from(value)
            .ok()
            .and_then(|index| Nayin::try_from(index).ok())
            .ok_or_else(|| {
                E::invalid_value(
                    de::Unexpected::Unsigned(value),
                    &"variant index 0 <= i < 30",
                )
            })
    }
    fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<Self::Value, E> {
        match core::str::from_utf8(value) {
            Ok(code) => self.visit_str(code),
            Err(_) => Err(E::invalid_value(de::Unexpected::Bytes(value), &self)),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Nayin {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct CodeVisitor;
        impl de::Visitor<'_> for CodeVisitor {
            type Value = Nayin;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("Nayin code string")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                de::Visitor::visit_str(NayinIdentifier, value)
            }
        }
        struct EnumVisitor;
        impl<'de> de::Visitor<'de> for EnumVisitor {
            type Value = Nayin;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("enum Nayin")
            }
            fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<Self::Value, A::Error> {
                let (value, variant) = data.variant_seed(NayinIdentifier)?;
                variant.unit_variant()?;
                Ok(value)
            }
        }
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(CodeVisitor)
        } else {
            deserializer.deserialize_enum("Nayin", NAYIN_CODES, EnumVisitor)
        }
    }
}
