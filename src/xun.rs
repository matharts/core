//! 六十甲子中的六旬身份与成员集合，不解释空亡的实际作用。

use crate::{Branch, CyclicRing, InvalidIndex, SexagenaryCycle};

/// 由甲起始的十个连续干支组成的一旬。
///
/// 固定序列为甲子、甲戌、甲申、甲午、甲辰、甲寅；不表示民用月份的上中下旬。
/// 旬内位置取决于具体干支，不能仅由旬身份确定。
///
/// ```
/// use matharts_core::{Branch, SexagenaryCycle, Stem, Xun};
/// let gui_you = SexagenaryCycle::new(Stem::Gui, Branch::You).unwrap();
/// assert_eq!(gui_you.xun(), Xun::JiaZi);
/// assert!(Xun::JiaZi.contains(gui_you));
/// assert_eq!(Xun::JiaZi.members()[9], gui_you);
/// assert_eq!(Xun::JiaZi.void_branches(), (Branch::Xu, Branch::Hai));
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[must_use]
pub enum Xun {
    /// 甲子至癸酉。
    JiaZi = 0,
    /// 甲戌至癸未。
    JiaXu = 1,
    /// 甲申至癸巳。
    JiaShen = 2,
    /// 甲午至癸卯。
    JiaWu = 3,
    /// 甲辰至癸丑。
    JiaChen = 4,
    /// 甲寅至癸亥。
    JiaYin = 5,
}

impl Xun {
    /// 全部六旬，按六十甲子的分段顺序排列。
    pub const ALL: [Self; 6] = [
        Self::JiaZi,
        Self::JiaXu,
        Self::JiaShen,
        Self::JiaWu,
        Self::JiaChen,
        Self::JiaYin,
    ];

    /// 中文显示名称；机器编码使用显式固定的 Serde 代码。
    pub const fn name(self) -> &'static str {
        match self {
            Self::JiaZi => "甲子旬",
            Self::JiaXu => "甲戌旬",
            Self::JiaShen => "甲申旬",
            Self::JiaWu => "甲午旬",
            Self::JiaChen => "甲辰旬",
            Self::JiaYin => "甲寅旬",
        }
    }

    /// 旬首干支，天干恒为甲。
    pub fn leader(self) -> SexagenaryCycle {
        SexagenaryCycle::from_index(self.index() * 10)
    }

    /// 从旬首到旬末的十个干支，无堆分配。
    pub fn members(self) -> [SexagenaryCycle; 10] {
        let mut next = self.leader();
        core::array::from_fn(|_| {
            let current = next;
            next = next.offset(1);
            current
        })
    }

    /// 判断干支是否属于本旬。
    pub fn contains(self, value: SexagenaryCycle) -> bool {
        value.xun() == self
    }

    /// 旬内未出现的两支，依次为旬首地支前两位、前一位。
    ///
    /// 只返回集合缺位，不判断命盘中的空亡效应。
    pub fn void_branches(self) -> (Branch, Branch) {
        let leader = self.leader().branch();
        (leader.offset(-2), leader.offset(-1))
    }

    /// 判断地支是否属于本旬缺位集合，不判断命盘中的实际作用。
    pub fn is_void_branch(self, branch: Branch) -> bool {
        let (first, second) = self.void_branches();
        branch == first || branch == second
    }
}

impl CyclicRing for Xun {
    const MODULUS: core::num::NonZeroU8 = core::num::NonZeroU8::new(6).unwrap();

    fn from_index(index: u8) -> Self {
        Self::ALL[usize::from(index % Self::MODULUS.get())]
    }

    fn index(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for Xun {
    type Error = InvalidIndex;

    fn try_from(index: u8) -> Result<Self, Self::Error> {
        Self::try_from_index(index)
    }
}

// 可读格式只接受代码字符串；紧凑格式使用原生枚举模型。

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Xun {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{self, EnumAccess, VariantAccess};
        const CODES: &[&str] = &["JiaZi", "JiaXu", "JiaShen", "JiaWu", "JiaChen", "JiaYin"];
        struct Identifier;
        impl<'de> de::DeserializeSeed<'de> for Identifier {
            type Value = Xun;
            fn deserialize<D: serde::Deserializer<'de>>(
                self,
                d: D,
            ) -> Result<Self::Value, D::Error> {
                d.deserialize_identifier(self)
            }
        }
        impl de::Visitor<'_> for Identifier {
            type Value = Xun;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("variant identifier")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                match value {
                    "JiaZi" => Ok(Xun::JiaZi),
                    "JiaXu" => Ok(Xun::JiaXu),
                    "JiaShen" => Ok(Xun::JiaShen),
                    "JiaWu" => Ok(Xun::JiaWu),
                    "JiaChen" => Ok(Xun::JiaChen),
                    "JiaYin" => Ok(Xun::JiaYin),
                    _ => Err(E::unknown_variant(value, CODES)),
                }
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                match value {
                    0 => Ok(Xun::JiaZi),
                    1 => Ok(Xun::JiaXu),
                    2 => Ok(Xun::JiaShen),
                    3 => Ok(Xun::JiaWu),
                    4 => Ok(Xun::JiaChen),
                    5 => Ok(Xun::JiaYin),
                    _ => Err(E::invalid_value(
                        de::Unexpected::Unsigned(value),
                        &"variant index 0 <= i < 6",
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
            type Value = Xun;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("Xun code string")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                de::Visitor::visit_str(Identifier, value)
            }
        }
        struct EnumVisitor;
        impl<'de> de::Visitor<'de> for EnumVisitor {
            type Value = Xun;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("enum Xun")
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
            deserializer.deserialize_enum("Xun", CODES, EnumVisitor)
        }
    }
}
