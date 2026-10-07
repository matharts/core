//! 五行及有向生克关系。

use core::{fmt, str::FromStr};

/// 五行 (木、火、土、金、水)
/// 遵循模 5 循环顺生次序：木(0) -> 火(1) -> 土(2) -> 金(3) -> 水(4)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[must_use]
pub enum Element {
    /// 木
    Wood = 0,
    /// 火
    Fire = 1,
    /// 土
    Earth = 2,
    /// 金
    Metal = 3,
    /// 水
    Water = 4,
}

/// 从自身指向目标的五行关系，不附加十神或六亲解释
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ElementRelation {
    /// 同五行
    Same,
    /// 我生目标
    Generates,
    /// 我克目标
    Overcomes,
    /// 目标克我
    OvercomeBy,
    /// 目标生我
    GeneratedBy,
}

impl Element {
    /// 精确中文名称；机器编码仍使用 Serde 代码。
    pub const fn name(self) -> &'static str {
        match self {
            Self::Wood => "木",
            Self::Fire => "火",
            Self::Earth => "土",
            Self::Metal => "金",
            Self::Water => "水",
        }
    }

    /// 全部五行，按索引对应的相生顺序排列：木、火、土、金、水。
    pub const ALL: [Self; 5] = [
        Self::Wood,
        Self::Fire,
        Self::Earth,
        Self::Metal,
        Self::Water,
    ];

    /// 按模 5 约减索引，明确允许回绕。
    #[inline]
    pub const fn from_index(idx: u8) -> Self {
        Self::ALL[(idx % 5) as usize]
    }

    #[inline]
    /// 固定相生序列中的零基索引。
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// 生我者（逆退 1 步，水生木、木生火...）
    #[inline]
    pub const fn generated_by(self) -> Self {
        Self::from_index(self.index() + 4)
    }

    /// 我生者（顺进 1 步，木生火、火生土...）
    #[inline]
    pub const fn generates(self) -> Self {
        Self::from_index(self.index() + 1)
    }

    /// 克我者（逆退 2 步，金克木、水克火...）
    #[inline]
    pub const fn overcome_by(self) -> Self {
        Self::from_index(self.index() + 3)
    }

    /// 我克者（顺进 2 步，木克土、土克水...）
    #[inline]
    pub const fn overcomes(self) -> Self {
        Self::from_index(self.index() + 2)
    }

    /// 计算 target 相对于 self 的生克关系
    #[inline]
    pub const fn relation_to(self, target: Self) -> ElementRelation {
        let diff = (target.index() + 5 - self.index()) % 5;
        match diff {
            0 => ElementRelation::Same,        // 同五行
            1 => ElementRelation::Generates,   // 我生目标
            2 => ElementRelation::Overcomes,   // 我克目标
            3 => ElementRelation::OvercomeBy,  // 目标克我
            4 => ElementRelation::GeneratedBy, // 目标生我
            _ => unreachable!(),
        }
    }
}

/// 输出精确中文名称，与 [`FromStr`] 的输入一致。
impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// 只接受精确中文五行名，不裁剪空白或接受别名。
impl FromStr for Element {
    type Err = crate::ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|value| value.name() == input)
            .ok_or(Self::Err::InvalidElement)
    }
}

impl TryFrom<u8> for Element {
    type Error = crate::InvalidIndex;
    fn try_from(index: u8) -> Result<Self, Self::Error> {
        if index < 5 {
            Ok(Self::from_index(index))
        } else {
            Err(crate::InvalidIndex {
                index,
                upper_bound: 5,
            })
        }
    }
}

// 可读格式只接受代码字符串；紧凑格式使用原生枚举模型。

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Element {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{self, EnumAccess, VariantAccess};
        const CODES: &[&str] = &["Wood", "Fire", "Earth", "Metal", "Water"];
        struct Identifier;
        impl<'de> de::DeserializeSeed<'de> for Identifier {
            type Value = Element;
            fn deserialize<D: serde::Deserializer<'de>>(
                self,
                d: D,
            ) -> Result<Self::Value, D::Error> {
                d.deserialize_identifier(self)
            }
        }
        impl de::Visitor<'_> for Identifier {
            type Value = Element;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("variant identifier")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                match value {
                    "Wood" => Ok(Element::Wood),
                    "Fire" => Ok(Element::Fire),
                    "Earth" => Ok(Element::Earth),
                    "Metal" => Ok(Element::Metal),
                    "Water" => Ok(Element::Water),
                    _ => Err(E::unknown_variant(value, CODES)),
                }
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                match value {
                    0 => Ok(Element::Wood),
                    1 => Ok(Element::Fire),
                    2 => Ok(Element::Earth),
                    3 => Ok(Element::Metal),
                    4 => Ok(Element::Water),
                    _ => Err(E::invalid_value(
                        de::Unexpected::Unsigned(value),
                        &"variant index 0 <= i < 5",
                    )),
                }
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
            type Value = Element;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("Element code string")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                de::Visitor::visit_str(Identifier, value)
            }
        }
        struct EnumVisitor;
        impl<'de> de::Visitor<'de> for EnumVisitor {
            type Value = Element;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("enum Element")
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
            deserializer.deserialize_enum("Element", CODES, EnumVisitor)
        }
    }
}

// 可读格式只接受代码字符串；紧凑格式使用原生枚举模型。

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ElementRelation {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{self, EnumAccess, VariantAccess};
        const CODES: &[&str] = &[
            "Same",
            "Generates",
            "Overcomes",
            "OvercomeBy",
            "GeneratedBy",
        ];
        struct Identifier;
        impl<'de> de::DeserializeSeed<'de> for Identifier {
            type Value = ElementRelation;
            fn deserialize<D: serde::Deserializer<'de>>(
                self,
                d: D,
            ) -> Result<Self::Value, D::Error> {
                d.deserialize_identifier(self)
            }
        }
        impl de::Visitor<'_> for Identifier {
            type Value = ElementRelation;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("variant identifier")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                match value {
                    "Same" => Ok(ElementRelation::Same),
                    "Generates" => Ok(ElementRelation::Generates),
                    "Overcomes" => Ok(ElementRelation::Overcomes),
                    "OvercomeBy" => Ok(ElementRelation::OvercomeBy),
                    "GeneratedBy" => Ok(ElementRelation::GeneratedBy),
                    _ => Err(E::unknown_variant(value, CODES)),
                }
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                match value {
                    0 => Ok(ElementRelation::Same),
                    1 => Ok(ElementRelation::Generates),
                    2 => Ok(ElementRelation::Overcomes),
                    3 => Ok(ElementRelation::OvercomeBy),
                    4 => Ok(ElementRelation::GeneratedBy),
                    _ => Err(E::invalid_value(
                        de::Unexpected::Unsigned(value),
                        &"variant index 0 <= i < 5",
                    )),
                }
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
            type Value = ElementRelation;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("ElementRelation code string")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                de::Visitor::visit_str(Identifier, value)
            }
        }
        struct EnumVisitor;
        impl<'de> de::Visitor<'de> for EnumVisitor {
            type Value = ElementRelation;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("enum ElementRelation")
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
            deserializer.deserialize_enum("ElementRelation", CODES, EnumVisitor)
        }
    }
}
