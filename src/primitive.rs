//! 阴阳值；排序仅表示固定编码顺序。

/// 基础二元极性（阴阳原语）
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[must_use]
pub enum Primitive {
    /// 阳
    Yang = 0,
    /// 阴
    Yin = 1,
}

impl Primitive {
    /// 取反（阴阳互换）
    #[inline]
    pub const fn invert(self) -> Self {
        match self {
            Self::Yang => Self::Yin,
            Self::Yin => Self::Yang,
        }
    }

    /// 是否为阳
    #[inline]
    pub const fn is_yang(self) -> bool {
        matches!(self, Self::Yang)
    }

    /// 是否为阴
    #[inline]
    pub const fn is_yin(self) -> bool {
        matches!(self, Self::Yin)
    }
}

// 可读格式只接受代码字符串；紧凑格式使用原生枚举模型。

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Primitive {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{self, EnumAccess, VariantAccess};
        const CODES: &[&str] = &["Yang", "Yin"];
        struct Identifier;
        impl<'de> de::DeserializeSeed<'de> for Identifier {
            type Value = Primitive;
            fn deserialize<D: serde::Deserializer<'de>>(
                self,
                d: D,
            ) -> Result<Self::Value, D::Error> {
                d.deserialize_identifier(self)
            }
        }
        impl de::Visitor<'_> for Identifier {
            type Value = Primitive;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("variant identifier")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                match value {
                    "Yang" => Ok(Primitive::Yang),
                    "Yin" => Ok(Primitive::Yin),
                    _ => Err(E::unknown_variant(value, CODES)),
                }
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                match value {
                    0 => Ok(Primitive::Yang),
                    1 => Ok(Primitive::Yin),
                    _ => Err(E::invalid_value(
                        de::Unexpected::Unsigned(value),
                        &"variant index 0 <= i < 2",
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
            type Value = Primitive;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("Primitive code string")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                de::Visitor::visit_str(Identifier, value)
            }
        }
        struct EnumVisitor;
        impl<'de> de::Visitor<'de> for EnumVisitor {
            type Value = Primitive;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("enum Primitive")
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
            deserializer.deserialize_enum("Primitive", CODES, EnumVisitor)
        }
    }
}
