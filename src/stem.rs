//! 天干值、固有属性和固定配对。

use crate::element::Element;
use crate::error::InvalidIndex;
use crate::math::CyclicRing;
use crate::primitive::Primitive;
use core::{
    fmt,
    ops::{Add, Sub},
    str::FromStr,
};

/// 十天干 (0..=9)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[must_use]
pub enum Stem {
    /// 甲
    Jia = 0,
    /// 乙
    Yi = 1,
    /// 丙
    Bing = 2,
    /// 丁
    Ding = 3,
    /// 戊
    Wu = 4,
    /// 己
    Ji = 5,
    /// 庚
    Geng = 6,
    /// 辛
    Xin = 7,
    /// 壬
    Ren = 8,
    /// 癸
    Gui = 9,
}

impl CyclicRing for Stem {
    const MODULUS: core::num::NonZeroU8 = core::num::NonZeroU8::new(10).unwrap();

    #[inline]
    fn from_index(idx: u8) -> Self {
        Self::ALL[(idx % Self::MODULUS.get()) as usize]
    }

    #[inline]
    fn index(self) -> u8 {
        self as u8
    }
}

impl Stem {
    /// 全部十干，按索引顺序由甲至癸排列。
    pub const ALL: [Self; 10] = [
        Self::Jia,
        Self::Yi,
        Self::Bing,
        Self::Ding,
        Self::Wu,
        Self::Ji,
        Self::Geng,
        Self::Xin,
        Self::Ren,
        Self::Gui,
    ];

    /// 阴阳极性：传统一基序数奇阳偶阴，对应零基索引偶阳奇阴。
    #[inline]
    pub const fn primitive(self) -> Primitive {
        if (self as u8).is_multiple_of(2) {
            Primitive::Yang
        } else {
            Primitive::Yin
        }
    }

    /// 正五行属性
    #[inline]
    pub const fn element(self) -> Element {
        match self {
            Self::Jia | Self::Yi => Element::Wood,
            Self::Bing | Self::Ding => Element::Fire,
            Self::Wu | Self::Ji => Element::Earth,
            Self::Geng | Self::Xin => Element::Metal,
            Self::Ren | Self::Gui => Element::Water,
        }
    }

    /// 本干所属的完整五合组，不表示另一成员已出现或合化成立。
    /// 成员、伙伴及固定对应五行由 [`crate::FiveCombination`] 查询。
    pub const fn five_combination(self) -> crate::FiveCombination {
        crate::FiveCombination::from_stem(self)
    }

    /// 五合配对中的另一干；沿用 [`Self::five_combination`] 的固定表，不判断合化。
    pub const fn five_combination_partner(self) -> Self {
        let [first, second] = self.five_combination().members();
        if self as u8 == first as u8 {
            second
        } else {
            first
        }
    }

    /// 天干相冲 (甲庚冲、乙辛冲、丙壬冲、丁癸冲)
    ///
    /// # 采用定义与证据状态
    /// 来源版本：[南秉吉《选择纪要》上编（1867）《天干相冲》](https://zh.wikisource.org/wiki/選擇紀要/上編)。
    /// 采用四对双向配对，戊己无冲；
    /// 消费场景是择日或命理调用方识别两干是否属于此表，不判断冲的实际作用。
    /// 同输入对照：甲庚返回 `true`，戊己返回 `false`，对应上述四对及戊己无冲一说。
    /// 相冲不等于所有五行相克：甲戊相克，但本方法返回 `false`。
    /// 全部 100 个有序输入由 `tests/contracts.rs` 独立配对验收。
    /// 转录的影印本校勘及跨体系消费方的同输入对照证据待补；保持已有表，
    /// 不把天干冲表作为所有体系的无条件默认规则。
    #[inline]
    pub const fn is_clashing_with(self, target: Self) -> bool {
        matches!(
            (self, target),
            (Self::Jia, Self::Geng)
                | (Self::Geng, Self::Jia)
                | (Self::Yi, Self::Xin)
                | (Self::Xin, Self::Yi)
                | (Self::Bing, Self::Ren)
                | (Self::Ren, Self::Bing)
                | (Self::Ding, Self::Gui)
                | (Self::Gui, Self::Ding)
        )
    }
}

/// 将整数位移的向前步进结果写回自身。
impl core::ops::AddAssign<i32> for Stem {
    fn add_assign(&mut self, rhs: i32) {
        *self = *self + rhs;
    }
}

/// 将整数位移的向后步进结果写回自身。
impl core::ops::SubAssign<i32> for Stem {
    fn sub_assign(&mut self, rhs: i32) {
        *self = *self - rhs;
    }
}

impl Add<i32> for Stem {
    type Output = Self;
    #[inline]
    fn add(self, rhs: i32) -> Self::Output {
        self.offset(rhs)
    }
}

/// 向后步进整数位移；支持全部 `i32`，包括 `i32::MIN`。
impl Sub<i32> for Stem {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: i32) -> Self::Output {
        Self::from_index(crate::math::ring::wrap(
            i64::from(self.index()) - i64::from(rhs),
            Self::MODULUS,
        ))
    }
}

impl TryFrom<u8> for Stem {
    type Error = crate::InvalidIndex;
    fn try_from(index: u8) -> Result<Self, Self::Error> {
        Self::try_from_index(index)
    }
}

/// 输出精确中文天干名，与 [`FromStr`] 的输入一致；不改变 Serde 编码。
impl fmt::Display for Stem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Jia => "甲",
            Self::Yi => "乙",
            Self::Bing => "丙",
            Self::Ding => "丁",
            Self::Wu => "戊",
            Self::Ji => "己",
            Self::Geng => "庚",
            Self::Xin => "辛",
            Self::Ren => "壬",
            Self::Gui => "癸",
        })
    }
}

/// 只接受精确中文天干名，不裁剪空白，不接受拼音或其他别名。
///
/// ```
/// use matharts_core::{ParseError, Stem};
/// assert_eq!("甲".parse::<Stem>(), Ok(Stem::Jia));
/// assert_eq!(Stem::Jia.to_string(), "甲");
/// assert_eq!(" 甲".parse::<Stem>(), Err(ParseError::InvalidStem));
/// ```
impl FromStr for Stem {
    type Err = crate::ParseError;

    /// # Errors
    /// 输入不是精确中文天干名时返回 [`crate::ParseError::InvalidStem`]。
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "甲" => Ok(Self::Jia),
            "乙" => Ok(Self::Yi),
            "丙" => Ok(Self::Bing),
            "丁" => Ok(Self::Ding),
            "戊" => Ok(Self::Wu),
            "己" => Ok(Self::Ji),
            "庚" => Ok(Self::Geng),
            "辛" => Ok(Self::Xin),
            "壬" => Ok(Self::Ren),
            "癸" => Ok(Self::Gui),
            _ => Err(Self::Err::InvalidStem),
        }
    }
}

// 可读格式只接受代码字符串；紧凑格式使用原生枚举模型。

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Stem {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{self, EnumAccess, VariantAccess};
        const CODES: &[&str] = &[
            "Jia", "Yi", "Bing", "Ding", "Wu", "Ji", "Geng", "Xin", "Ren", "Gui",
        ];
        struct Identifier;
        impl<'de> de::DeserializeSeed<'de> for Identifier {
            type Value = Stem;
            fn deserialize<D: serde::Deserializer<'de>>(
                self,
                d: D,
            ) -> Result<Self::Value, D::Error> {
                d.deserialize_identifier(self)
            }
        }
        impl de::Visitor<'_> for Identifier {
            type Value = Stem;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("variant identifier")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                match value {
                    "Jia" => Ok(Stem::Jia),
                    "Yi" => Ok(Stem::Yi),
                    "Bing" => Ok(Stem::Bing),
                    "Ding" => Ok(Stem::Ding),
                    "Wu" => Ok(Stem::Wu),
                    "Ji" => Ok(Stem::Ji),
                    "Geng" => Ok(Stem::Geng),
                    "Xin" => Ok(Stem::Xin),
                    "Ren" => Ok(Stem::Ren),
                    "Gui" => Ok(Stem::Gui),
                    _ => Err(E::unknown_variant(value, CODES)),
                }
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                u8::try_from(value)
                    .ok()
                    .and_then(|index| Stem::try_from(index).ok())
                    .ok_or_else(|| {
                        E::invalid_value(
                            de::Unexpected::Unsigned(value),
                            &"variant index 0 <= i < 10",
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
        struct CodeVisitor;
        impl de::Visitor<'_> for CodeVisitor {
            type Value = Stem;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("Stem code string")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                de::Visitor::visit_str(Identifier, value)
            }
        }
        struct EnumVisitor;
        impl<'de> de::Visitor<'de> for EnumVisitor {
            type Value = Stem;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("enum Stem")
            }
            fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<Self::Value, A::Error> {
                let (value, variant) = data.variant_seed(Identifier)?;
                variant.unit_variant()?;
                Ok(value)
            }
        }
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(CodeVisitor)
        } else {
            deserializer.deserialize_enum("Stem", CODES, EnumVisitor)
        }
    }
}

// 天干五合身份及其固定对应。
/// 五种天干五合组，五行只是采用表的固定对应。
///
/// 成员按天干索引升序；单干归属不表示另一成员已经出现。
///
/// # 采用定义与证据状态
/// 来源版本：[南秉吉《选择纪要》上编（1867）《天干五合化气》](https://zh.wikisource.org/wiki/選擇紀要/上編)，
/// 成员另见[《三命通会》四库全书本卷二《论十干合》](https://zh.wikisource.org/wiki/三命通會_(四庫全書本)/卷02)。
/// 采用甲己土、乙庚金、丙辛水、丁壬木、戊癸火；消费场景是择日或命理中
/// 两干的固定组身份及对应五行查询。
/// 同输入对照：乙庚在两处均属于合的成员，本库返回 `YiGeng`，固定对应金；
/// 甲己返回 `JiaJi`，固定对应土。古籍还讨论化与不化的条件，
/// 本类型只保留配对和固定对应，不执行条件判断。
/// 独立成员、五行及编码见 `tests/pair_groups.rs` 的冻结样本验收。
/// 转录的影印本校勘、跨体系实际消费方及同输入应用结果对照证据待补；
/// 保留现有定义，不宣称择日的合与命理的合化已经具有相同应用语义。
///
/// ```
/// use matharts_core::{Element, FiveCombination, Stem};
/// let group = Stem::Jia.five_combination();
/// assert_eq!(group, FiveCombination::JiaJi);
/// assert_eq!(group.members(), [Stem::Jia, Stem::Ji]);
/// assert_eq!(group.element(), Element::Earth);
/// assert_eq!(group.partner_of(Stem::Jia), Some(Stem::Ji));
/// assert_eq!(group.partner_of(Stem::Yi), None);
/// assert_eq!(FiveCombination::from_stems([Stem::Ji, Stem::Jia]), Some(group));
/// assert_eq!(FiveCombination::from_stems([Stem::Jia; 2]), None);
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[must_use]
pub enum FiveCombination {
    /// 甲己组，固定对应土。
    JiaJi = 0,
    /// 乙庚组，固定对应金。
    YiGeng = 1,
    /// 丙辛组，固定对应水。
    BingXin = 2,
    /// 丁壬组，固定对应木。
    DingRen = 3,
    /// 戊癸组，固定对应火。
    WuGui = 4,
}

impl FiveCombination {
    /// 全部五组，按身份索引排列；不表示强弱或合化顺序。
    pub const ALL: [Self; 5] = [
        Self::JiaJi,
        Self::YiGeng,
        Self::BingXin,
        Self::DingRen,
        Self::WuGui,
    ];

    /// 固定零基身份索引，范围为 `0..5`；不提供周期回绕。
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// 完整两干，按天干索引升序返回，不保留输入次序。
    pub const fn members(self) -> [Stem; 2] {
        match self {
            Self::JiaJi => [Stem::Jia, Stem::Ji],
            Self::YiGeng => [Stem::Yi, Stem::Geng],
            Self::BingXin => [Stem::Bing, Stem::Xin],
            Self::DingRen => [Stem::Ding, Stem::Ren],
            Self::WuGui => [Stem::Wu, Stem::Gui],
        }
    }

    /// 固定对应五行，不改变成员本气，也不判断实际合化条件。
    pub const fn element(self) -> Element {
        match self {
            Self::JiaJi => Element::Earth,
            Self::YiGeng => Element::Metal,
            Self::BingXin => Element::Water,
            Self::DingRen => Element::Wood,
            Self::WuGui => Element::Fire,
        }
    }

    /// 本干是否属于此固定配对。
    pub const fn contains(self, stem: Stem) -> bool {
        Self::from_stem(stem).index() == self.index()
    }

    /// 查询给定成员的另一干；非本组成员返回 `None`。
    pub const fn partner_of(self, stem: Stem) -> Option<Stem> {
        let [a, b] = self.members();
        if stem as u8 == a as u8 {
            Some(b)
        } else if stem as u8 == b as u8 {
            Some(a)
        } else {
            None
        }
    }

    /// 单干的唯一所属组，不表示完整配对已经出现。
    pub const fn from_stem(stem: Stem) -> Self {
        match stem {
            Stem::Jia | Stem::Ji => Self::JiaJi,
            Stem::Yi | Stem::Geng => Self::YiGeng,
            Stem::Bing | Stem::Xin => Self::BingXin,
            Stem::Ding | Stem::Ren => Self::DingRen,
            Stem::Wu | Stem::Gui => Self::WuGui,
        }
    }

    /// 识别完整无序配对，拒绝重复成员与混组；不判断合化。
    pub const fn from_stems([a, b]: [Stem; 2]) -> Option<Self> {
        let group = Self::from_stem(a);
        if a as u8 != b as u8 && group.contains(b) {
            Some(group)
        } else {
            None
        }
    }
}

impl TryFrom<u8> for FiveCombination {
    type Error = InvalidIndex;

    /// # Errors
    /// 索引不在 `0..5` 时返回错误，不做周期约减。
    fn try_from(index: u8) -> Result<Self, Self::Error> {
        Self::ALL
            .get(usize::from(index))
            .copied()
            .ok_or(InvalidIndex {
                index,
                upper_bound: 5,
            })
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for FiveCombination {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::{self, EnumAccess, VariantAccess};
        const CODES: &[&str] = &["JiaJi", "YiGeng", "BingXin", "DingRen", "WuGui"];
        struct Identifier;
        impl<'de> de::DeserializeSeed<'de> for Identifier {
            type Value = FiveCombination;
            fn deserialize<I: serde::Deserializer<'de>>(
                self,
                d: I,
            ) -> Result<FiveCombination, I::Error> {
                d.deserialize_identifier(self)
            }
        }
        impl de::Visitor<'_> for Identifier {
            type Value = FiveCombination;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("variant identifier")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<FiveCombination, E> {
                match value {
                    "JiaJi" => Ok(FiveCombination::JiaJi),
                    "YiGeng" => Ok(FiveCombination::YiGeng),
                    "BingXin" => Ok(FiveCombination::BingXin),
                    "DingRen" => Ok(FiveCombination::DingRen),
                    "WuGui" => Ok(FiveCombination::WuGui),
                    _ => Err(E::unknown_variant(value, CODES)),
                }
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<FiveCombination, E> {
                u8::try_from(value)
                    .ok()
                    .and_then(|index| FiveCombination::try_from(index).ok())
                    .ok_or_else(|| {
                        E::invalid_value(
                            de::Unexpected::Unsigned(value),
                            &"variant index 0 <= i < 5",
                        )
                    })
            }
            fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<FiveCombination, E> {
                match core::str::from_utf8(value) {
                    Ok(code) => self.visit_str(code),
                    Err(_) => Err(E::invalid_value(de::Unexpected::Bytes(value), &self)),
                }
            }
        }
        struct CodeVisitor;
        impl de::Visitor<'_> for CodeVisitor {
            type Value = FiveCombination;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("FiveCombination code string")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<FiveCombination, E> {
                de::Visitor::visit_str(Identifier, value)
            }
        }
        struct EnumVisitor;
        impl<'de> de::Visitor<'de> for EnumVisitor {
            type Value = FiveCombination;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("enum FiveCombination")
            }
            fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<FiveCombination, A::Error> {
                let (value, variant) = data.variant_seed(Identifier)?;
                variant.unit_variant()?;
                Ok(value)
            }
        }
        if d.is_human_readable() {
            d.deserialize_str(CodeVisitor)
        } else {
            d.deserialize_enum("FiveCombination", CODES, EnumVisitor)
        }
    }
}
