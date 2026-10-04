//! 六爻卦的纯结构；下卦在前，上卦在后，不包含起卦或占断信息。

use crate::{InvalidIndex, Primitive, Trigram};

/// 两个八卦构成的六爻结构。所有8×8组合均合法。
///
/// 爻数组从下到上；位编码最低位代表初爻。类型不保存动爻、卦序或流派规则。
///
/// ```
/// use matharts_core::{Hexagram, HexagramPosition, Primitive, Trigram};
/// let tai = Hexagram::from_trigrams(Trigram::Qian, Trigram::Kun);
/// assert_eq!(tai.lines(), [Primitive::Yang, Primitive::Yang, Primitive::Yang,
///     Primitive::Yin, Primitive::Yin, Primitive::Yin]);
/// assert_eq!(tai.bits(), 7);
/// let pi = Hexagram::from_trigrams(Trigram::Kun, Trigram::Qian);
/// assert_eq!(tai.reverse_lines(), pi);
/// let qian = Hexagram::from_trigrams(Trigram::Qian, Trigram::Qian);
/// let gou = qian.toggle_line(HexagramPosition::First);
/// assert_eq!((gou.lower(), gou.upper()), (Trigram::Xun, Trigram::Qian));
/// assert_eq!(gou.bits(), 62);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename = "Hexagram"))]
#[must_use]
pub struct Hexagram {
    #[cfg_attr(feature = "serde", serde(rename = "lower"))]
    lower: Trigram,
    #[cfg_attr(feature = "serde", serde(rename = "upper"))]
    upper: Trigram,
}
impl Hexagram {
    /// 全部64种结构，按 bits 升序排列，不代表任何传统卦序。
    pub const ALL: [Self; 64] = {
        let mut values = [Self::from_trigrams(Trigram::Kun, Trigram::Kun); 64];
        let mut i = 0;
        while i < 64 {
            values[i] = Self::from_trigrams(Trigram::ALL[i % 8], Trigram::ALL[i / 8]);
            i += 1;
        }
        values
    };
    /// 从下卦、上卦构造；所有组合都合法。
    pub const fn from_trigrams(lower: Trigram, upper: Trigram) -> Self {
        Self { lower, upper }
    }
    /// 下方三个爻构成的卦。
    pub const fn lower(self) -> Trigram {
        self.lower
    }
    /// 上方三个爻构成的卦。
    pub const fn upper(self) -> Trigram {
        self.upper
    }
    /// 从下到上的六个阴阳爻构造。
    pub const fn from_lines(lines: [Primitive; 6]) -> Self {
        Self::from_trigrams(
            Trigram::from_lines([lines[0], lines[1], lines[2]]),
            Trigram::from_lines([lines[3], lines[4], lines[5]]),
        )
    }
    /// 返回从下到上的六个爻。
    pub const fn lines(self) -> [Primitive; 6] {
        let lower = self.lower.lines();
        let upper = self.upper.lines();
        [lower[0], lower[1], lower[2], upper[0], upper[1], upper[2]]
    }
    /// 结构位编码，阳为1、阴为0；不是文王卦序。
    pub const fn bits(self) -> u8 {
        self.lower.bits() | (self.upper.bits() << 3)
    }
    /// 严格解析结构位编码，不屏蔽超出六爻的高位。
    ///
    /// # Errors
    /// bits 不在0至63时返回 `InvalidIndex`；其 index 是结构编码，上界为64。
    pub const fn try_from_bits(bits: u8) -> Result<Self, InvalidIndex> {
        if bits < 64 {
            Ok(Self::ALL[bits as usize])
        } else {
            Err(InvalidIndex {
                index: bits,
                upper_bound: 64,
            })
        }
    }
    /// 查询指定爻位的阴阳。
    pub const fn line(self, position: HexagramPosition) -> Primitive {
        self.lines()[position.index() as usize]
    }
    /// 返回指定爻位替换为给定阴阳后的卦。
    pub const fn with_line(self, position: HexagramPosition, value: Primitive) -> Self {
        let mut lines = self.lines();
        lines[position.index() as usize] = value;
        Self::from_lines(lines)
    }
    /// 只交换指定爻位的阴阳；不选择动爻。
    pub const fn toggle_line(self, position: HexagramPosition) -> Self {
        self.with_line(position, self.line(position).invert())
    }
    /// 逐爻交换阴阳，保持爻位不变。
    pub const fn complement(self) -> Self {
        Self::from_trigrams(self.lower.complement(), self.upper.complement())
    }
    /// 倒置完整六爻顺序，同时交换上下卦并倒置各自三爻。
    pub const fn reverse_lines(self) -> Self {
        Self::from_trigrams(self.upper.reverse_lines(), self.lower.reverse_lines())
    }
}

/// 从下向上的六爻位置；索引从0开始，不循环回绕。
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[must_use]
pub enum HexagramPosition {
    /// 从下向上第1爻。
    First = 0,
    /// 从下向上第2爻。
    Second = 1,
    /// 从下向上第3爻。
    Third = 2,
    /// 从下向上第4爻。
    Fourth = 3,
    /// 从下向上第5爻。
    Fifth = 4,
    /// 从下向上第6爻。
    Sixth = 5,
}
impl HexagramPosition {
    /// 全部爻位，按从下到上排列。
    pub const ALL: [Self; 6] = [
        Self::First,
        Self::Second,
        Self::Third,
        Self::Fourth,
        Self::Fifth,
        Self::Sixth,
    ];
    /// 返回从下到上的零基索引。
    pub const fn index(self) -> u8 {
        self as u8
    }
}
impl TryFrom<u8> for HexagramPosition {
    type Error = InvalidIndex;
    fn try_from(index: u8) -> Result<Self, Self::Error> {
        Self::ALL
            .get(usize::from(index))
            .copied()
            .ok_or(InvalidIndex {
                index,
                upper_bound: 6,
            })
    }
}
// 名称、代码和索引属于稳定编码契约，独立于 Rust 声明顺序。

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for HexagramPosition {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use core::fmt;
        use serde::de::{self, EnumAccess, VariantAccess, Visitor};
        const CODES: &[&str] = &["First", "Second", "Third", "Fourth", "Fifth", "Sixth"];
        struct CodeVisitor;
        impl Visitor<'_> for CodeVisitor {
            type Value = HexagramPosition;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("HexagramPosition code string")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<HexagramPosition, E> {
                match value {
                    "First" => Ok(HexagramPosition::First),
                    "Second" => Ok(HexagramPosition::Second),
                    "Third" => Ok(HexagramPosition::Third),
                    "Fourth" => Ok(HexagramPosition::Fourth),
                    "Fifth" => Ok(HexagramPosition::Fifth),
                    "Sixth" => Ok(HexagramPosition::Sixth),
                    _ => Err(E::unknown_variant(value, CODES)),
                }
            }
        }
        // 索引只用于非人类可读枚举标识，不让 JSON 数字通过字符串入口。
        struct Identifier(HexagramPosition);
        impl<'de> serde::Deserialize<'de> for Identifier {
            fn deserialize<I: serde::Deserializer<'de>>(d: I) -> Result<Self, I::Error> {
                struct IdentifierVisitor;
                impl Visitor<'_> for IdentifierVisitor {
                    type Value = Identifier;
                    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        f.write_str("HexagramPosition variant code or index")
                    }
                    fn visit_str<E: de::Error>(self, value: &str) -> Result<Identifier, E> {
                        CodeVisitor.visit_str(value).map(Identifier)
                    }
                    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Identifier, E> {
                        match value {
                            0 => Ok(Identifier(HexagramPosition::First)),
                            1 => Ok(Identifier(HexagramPosition::Second)),
                            2 => Ok(Identifier(HexagramPosition::Third)),
                            3 => Ok(Identifier(HexagramPosition::Fourth)),
                            4 => Ok(Identifier(HexagramPosition::Fifth)),
                            5 => Ok(Identifier(HexagramPosition::Sixth)),
                            _ => Err(E::invalid_value(de::Unexpected::Unsigned(value), &self)),
                        }
                    }
                }
                d.deserialize_identifier(IdentifierVisitor)
            }
        }
        struct EnumVisitor;
        impl<'de> Visitor<'de> for EnumVisitor {
            type Value = HexagramPosition;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("HexagramPosition unit variant")
            }
            fn visit_enum<A: EnumAccess<'de>>(self, a: A) -> Result<HexagramPosition, A::Error> {
                let (Identifier(value), payload) = a.variant()?;
                payload.unit_variant()?;
                Ok(value)
            }
        }
        if d.is_human_readable() {
            d.deserialize_str(CodeVisitor)
        } else {
            d.deserialize_enum("HexagramPosition", CODES, EnumVisitor)
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Hexagram {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use core::fmt;
        use serde::de::{self, MapAccess, SeqAccess, Visitor};
        const FIELDS: &[&str] = &["lower", "upper"];
        #[derive(serde::Deserialize)]
        #[serde(field_identifier)]
        enum Field {
            #[serde(rename = "lower")]
            Lower,
            #[serde(rename = "upper")]
            Upper,
        }
        struct ShapeVisitor {
            human_readable: bool,
        }
        impl<'de> Visitor<'de> for ShapeVisitor {
            type Value = Hexagram;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("Hexagram fields lower and upper")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Hexagram, A::Error> {
                if self.human_readable {
                    return Err(de::Error::custom(
                        "human-readable Hexagram requires an object",
                    ));
                }
                let lower = a
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let upper = a
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                if a.next_element::<Trigram>()?.is_some() {
                    return Err(de::Error::invalid_length(3, &self));
                }
                Ok(Hexagram { lower, upper })
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Hexagram, A::Error> {
                let (mut lower, mut upper) = (None, None);
                while let Some(field) = a.next_key()? {
                    match field {
                        Field::Lower => {
                            if lower.is_some() {
                                return Err(de::Error::duplicate_field("lower"));
                            }
                            lower = Some(a.next_value()?);
                        }
                        Field::Upper => {
                            if upper.is_some() {
                                return Err(de::Error::duplicate_field("upper"));
                            }
                            upper = Some(a.next_value()?);
                        }
                    }
                }
                Ok(Hexagram {
                    lower: lower.ok_or_else(|| de::Error::missing_field("lower"))?,
                    upper: upper.ok_or_else(|| de::Error::missing_field("upper"))?,
                })
            }
        }
        let human_readable = d.is_human_readable();
        d.deserialize_struct("Hexagram", FIELDS, ShapeVisitor { human_readable })
    }
}
