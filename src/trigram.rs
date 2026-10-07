//! 八卦的三爻结构；数组从下到上，位编码最低位代表最下爻。

use crate::{InvalidIndex, Primitive};
use core::{fmt, str::FromStr};

/// 八种三爻结构。位编码阳为1、阴为0，不是先天、后天或文王卦序。
///
/// ```
/// use matharts_core::{Primitive, Trigram, TrigramPosition};
/// let zhen = Trigram::from_lines([Primitive::Yang, Primitive::Yin, Primitive::Yin]);
/// assert_eq!(zhen, Trigram::Zhen);
/// assert_eq!(zhen.bits(), 1);
/// assert_eq!(zhen.reverse_lines(), Trigram::Gen);
/// assert_eq!(zhen.complement(), Trigram::Xun);
/// assert_eq!(zhen.toggle_line(TrigramPosition::First), Trigram::Kun);
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[must_use]
pub enum Trigram {
    /// 坤卦。
    Kun = 0,
    /// 震卦。
    Zhen = 1,
    /// 坎卦。
    Kan = 2,
    /// 兑卦。
    Dui = 3,
    /// 艮卦。
    Gen = 4,
    /// 离卦。
    Li = 5,
    /// 巽卦。
    Xun = 6,
    /// 乾卦。
    Qian = 7,
}
/// 输出精确中文名称，与 [`FromStr`] 的输入一致。
impl fmt::Display for Trigram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// 只接受 [`Trigram::name`] 的精确名称，不接受“乾卦”等别名。
impl FromStr for Trigram {
    type Err = crate::ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|value| value.name() == input)
            .ok_or(Self::Err::InvalidTrigram)
    }
}

impl Trigram {
    /// 全部八卦，按 bits 升序排列，仅作技术遍历顺序。
    pub const ALL: [Self; 8] = [
        Self::Kun,
        Self::Zhen,
        Self::Kan,
        Self::Dui,
        Self::Gen,
        Self::Li,
        Self::Xun,
        Self::Qian,
    ];
    /// 中文显示名；机器编码使用固定 Serde 代码。
    pub const fn name(self) -> &'static str {
        match self {
            Self::Kun => "坤",
            Self::Zhen => "震",
            Self::Kan => "坎",
            Self::Dui => "兑",
            Self::Gen => "艮",
            Self::Li => "离",
            Self::Xun => "巽",
            Self::Qian => "乾",
        }
    }
    /// 从下到上的三个阴阳爻构造卦。
    pub const fn from_lines(lines: [Primitive; 3]) -> Self {
        let mut bits = 0;
        let mut index = 0;
        while index < 3 {
            if matches!(lines[index], Primitive::Yang) {
                bits |= 1 << index;
            }
            index += 1;
        }
        Self::ALL[bits]
    }
    /// 返回从下到上的三个爻。
    pub const fn lines(self) -> [Primitive; 3] {
        [
            self.line(TrigramPosition::First),
            self.line(TrigramPosition::Second),
            self.line(TrigramPosition::Third),
        ]
    }
    /// 结构位编码，最低位为最下爻，阳为1、阴为0。
    pub const fn bits(self) -> u8 {
        self as u8
    }
    /// 严格解析结构位编码，不屏蔽超出三爻的高位。
    ///
    /// # Errors
    /// bits 不在0至7时返回 `InvalidIndex`；其 index 是结构编码，上界为8。
    pub const fn try_from_bits(bits: u8) -> Result<Self, InvalidIndex> {
        if bits < 8 {
            Ok(Self::ALL[bits as usize])
        } else {
            Err(InvalidIndex {
                index: bits,
                upper_bound: 8,
            })
        }
    }
    /// 查询指定爻位的阴阳。
    pub const fn line(self, position: TrigramPosition) -> Primitive {
        if self.bits() & (1 << position.index()) == 0 {
            Primitive::Yin
        } else {
            Primitive::Yang
        }
    }
    /// 返回指定爻位替换为给定阴阳后的卦。
    pub const fn with_line(self, position: TrigramPosition, value: Primitive) -> Self {
        let mask = 1 << position.index();
        let bits = match value {
            Primitive::Yang => self.bits() | mask,
            Primitive::Yin => self.bits() & !mask,
        };
        Self::ALL[bits as usize]
    }
    /// 只交换指定爻位的阴阳；不选择动爻。
    pub const fn toggle_line(self, position: TrigramPosition) -> Self {
        Self::ALL[(self.bits() ^ (1 << position.index())) as usize]
    }
    /// 逐爻交换阴阳，保持爻位不变。
    pub const fn complement(self) -> Self {
        Self::ALL[(self.bits() ^ 7) as usize]
    }
    /// 倒置三个爻的顺序，保持各爻本身的阴阳。
    pub const fn reverse_lines(self) -> Self {
        let [first, second, third] = self.lines();
        Self::from_lines([third, second, first])
    }
}

/// 从下向上的三爻位置；索引从0开始，不循环回绕。
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[must_use]
pub enum TrigramPosition {
    /// 从下向上第1爻。
    First = 0,
    /// 从下向上第2爻。
    Second = 1,
    /// 从下向上第3爻。
    Third = 2,
}
impl TrigramPosition {
    /// 全部爻位，按从下到上排列。
    pub const ALL: [Self; 3] = [Self::First, Self::Second, Self::Third];
    /// 返回从下到上的零基索引。
    pub const fn index(self) -> u8 {
        self as u8
    }
}
impl TryFrom<u8> for TrigramPosition {
    type Error = InvalidIndex;
    fn try_from(index: u8) -> Result<Self, Self::Error> {
        Self::ALL
            .get(usize::from(index))
            .copied()
            .ok_or(InvalidIndex {
                index,
                upper_bound: 3,
            })
    }
}
// 名称、代码和索引属于稳定编码契约，独立于 Rust 声明顺序。

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for TrigramPosition {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use core::fmt;
        use serde::de::{self, EnumAccess, VariantAccess, Visitor};
        const CODES: &[&str] = &["First", "Second", "Third"];
        struct CodeVisitor;
        impl Visitor<'_> for CodeVisitor {
            type Value = TrigramPosition;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("TrigramPosition code string")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<TrigramPosition, E> {
                match value {
                    "First" => Ok(TrigramPosition::First),
                    "Second" => Ok(TrigramPosition::Second),
                    "Third" => Ok(TrigramPosition::Third),
                    _ => Err(E::unknown_variant(value, CODES)),
                }
            }
        }
        // 索引只用于非人类可读枚举标识，不让 JSON 数字通过字符串入口。
        struct Identifier(TrigramPosition);
        impl<'de> serde::Deserialize<'de> for Identifier {
            fn deserialize<I: serde::Deserializer<'de>>(d: I) -> Result<Self, I::Error> {
                struct IdentifierVisitor;
                impl Visitor<'_> for IdentifierVisitor {
                    type Value = Identifier;
                    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        f.write_str("TrigramPosition variant code or index")
                    }
                    fn visit_str<E: de::Error>(self, value: &str) -> Result<Identifier, E> {
                        CodeVisitor.visit_str(value).map(Identifier)
                    }
                    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Identifier, E> {
                        match value {
                            0 => Ok(Identifier(TrigramPosition::First)),
                            1 => Ok(Identifier(TrigramPosition::Second)),
                            2 => Ok(Identifier(TrigramPosition::Third)),
                            _ => Err(E::invalid_value(de::Unexpected::Unsigned(value), &self)),
                        }
                    }
                }
                d.deserialize_identifier(IdentifierVisitor)
            }
        }
        struct EnumVisitor;
        impl<'de> Visitor<'de> for EnumVisitor {
            type Value = TrigramPosition;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("TrigramPosition unit variant")
            }
            fn visit_enum<A: EnumAccess<'de>>(self, a: A) -> Result<TrigramPosition, A::Error> {
                let (Identifier(value), payload) = a.variant()?;
                payload.unit_variant()?;
                Ok(value)
            }
        }
        if d.is_human_readable() {
            d.deserialize_str(CodeVisitor)
        } else {
            d.deserialize_enum("TrigramPosition", CODES, EnumVisitor)
        }
    }
}

// 名称、代码和索引属于稳定编码契约，独立于 Rust 声明顺序。

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Trigram {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use core::fmt;
        use serde::de::{self, EnumAccess, VariantAccess, Visitor};
        const CODES: &[&str] = &["Kun", "Zhen", "Kan", "Dui", "Gen", "Li", "Xun", "Qian"];
        struct CodeVisitor;
        impl Visitor<'_> for CodeVisitor {
            type Value = Trigram;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("Trigram code string")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Trigram, E> {
                match value {
                    "Kun" => Ok(Trigram::Kun),
                    "Zhen" => Ok(Trigram::Zhen),
                    "Kan" => Ok(Trigram::Kan),
                    "Dui" => Ok(Trigram::Dui),
                    "Gen" => Ok(Trigram::Gen),
                    "Li" => Ok(Trigram::Li),
                    "Xun" => Ok(Trigram::Xun),
                    "Qian" => Ok(Trigram::Qian),
                    _ => Err(E::unknown_variant(value, CODES)),
                }
            }
        }
        // 索引只用于非人类可读枚举标识，不让 JSON 数字通过字符串入口。
        struct Identifier(Trigram);
        impl<'de> serde::Deserialize<'de> for Identifier {
            fn deserialize<I: serde::Deserializer<'de>>(d: I) -> Result<Self, I::Error> {
                struct IdentifierVisitor;
                impl Visitor<'_> for IdentifierVisitor {
                    type Value = Identifier;
                    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        f.write_str("Trigram variant code or index")
                    }
                    fn visit_str<E: de::Error>(self, value: &str) -> Result<Identifier, E> {
                        CodeVisitor.visit_str(value).map(Identifier)
                    }
                    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Identifier, E> {
                        match value {
                            0 => Ok(Identifier(Trigram::Kun)),
                            1 => Ok(Identifier(Trigram::Zhen)),
                            2 => Ok(Identifier(Trigram::Kan)),
                            3 => Ok(Identifier(Trigram::Dui)),
                            4 => Ok(Identifier(Trigram::Gen)),
                            5 => Ok(Identifier(Trigram::Li)),
                            6 => Ok(Identifier(Trigram::Xun)),
                            7 => Ok(Identifier(Trigram::Qian)),
                            _ => Err(E::invalid_value(de::Unexpected::Unsigned(value), &self)),
                        }
                    }
                }
                d.deserialize_identifier(IdentifierVisitor)
            }
        }
        struct EnumVisitor;
        impl<'de> Visitor<'de> for EnumVisitor {
            type Value = Trigram;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("Trigram unit variant")
            }
            fn visit_enum<A: EnumAccess<'de>>(self, a: A) -> Result<Trigram, A::Error> {
                let (Identifier(value), payload) = a.variant()?;
                payload.unit_variant()?;
                Ok(value)
            }
        }
        if d.is_human_readable() {
            d.deserialize_str(CodeVisitor)
        } else {
            d.deserialize_enum("Trigram", CODES, EnumVisitor)
        }
    }
}
