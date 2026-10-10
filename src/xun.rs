//! 六十甲子中的六旬身份与成员集合，不解释空亡的实际作用。

use crate::{Branch, CyclicSequence, Ganzhi, InvalidIndex};
use core::{fmt, str::FromStr};

/// 输出带“旬”字的精确中文名称，与 [`FromStr`] 的输入一致。
impl fmt::Display for Xun {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// 只接受 [`Xun::name`] 的完整名称；“甲子”是干支名，不是“甲子旬”的别名。
impl FromStr for Xun {
    type Err = crate::ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|value| value.name() == input)
            .ok_or(Self::Err::InvalidXun)
    }
}

/// 由甲起始的十个连续干支组成的一旬。
///
/// 固定序列为甲子、甲戌、甲申、甲午、甲辰、甲寅；不表示民用月份的上中下旬。
/// 旬内位置取决于具体干支，不能仅由旬身份确定。
///
/// ```
/// use matharts_core::{Branch, Ganzhi, Stem, Xun};
/// let gui_you = Ganzhi::new(Stem::Gui, Branch::You).unwrap();
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
    pub fn leader(self) -> Ganzhi {
        Ganzhi::from_index(self.index() * 10)
    }

    /// 从旬首到旬末的十个干支，无堆分配。
    pub fn members(self) -> [Ganzhi; 10] {
        let mut index = self.index() * 10;
        core::array::from_fn(|_| {
            let current = Ganzhi::from_index(index);
            index += 1;
            current
        })
    }

    /// 判断干支是否属于本旬。
    pub fn contains(self, value: Ganzhi) -> bool {
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

/// 向前步进整数位移；与 [`CyclicSequence::offset`] 相同。
impl core::ops::Add<i32> for Xun {
    type Output = Self;
    fn add(self, rhs: i32) -> Self::Output {
        self.offset(rhs)
    }
}

/// 向后步进整数位移；支持全部 `i32`，包括 `i32::MIN`。
impl core::ops::Sub<i32> for Xun {
    type Output = Self;
    fn sub(self, rhs: i32) -> Self::Output {
        Self::from_index(crate::math::sequence::wrap(
            i64::from(self.index()) - i64::from(rhs),
            Self::MODULUS,
        ))
    }
}

/// 将整数位移的向前步进结果写回自身。
impl core::ops::AddAssign<i32> for Xun {
    fn add_assign(&mut self, rhs: i32) {
        *self = *self + rhs;
    }
}

/// 将整数位移的向后步进结果写回自身。
impl core::ops::SubAssign<i32> for Xun {
    fn sub_assign(&mut self, rhs: i32) {
        *self = *self - rhs;
    }
}

impl CyclicSequence for Xun {
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
        const CODES: &[&str; Xun::ALL.len()] =
            &["JiaZi", "JiaXu", "JiaShen", "JiaWu", "JiaChen", "JiaYin"];
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
                CODES
                    .iter()
                    .position(|code| *code == value)
                    .map(|index| Xun::ALL[index])
                    .ok_or_else(|| E::unknown_variant(value, CODES))
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                usize::try_from(value)
                    .ok()
                    .and_then(|index| Xun::ALL.get(index))
                    .copied()
                    .ok_or_else(|| {
                        E::invalid_value(
                            de::Unexpected::Unsigned(value),
                            &"variant index 0 <= i < 6",
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
