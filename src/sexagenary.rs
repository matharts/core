//! 合法六十甲子值、旬内集合及纳音五行查询。

use crate::branch::Branch;
use crate::math::CyclicRing;
use crate::stem::Stem;
use crate::{InvalidGanzhi, InvalidIndex, Nayin, ParseError, Xun};
use core::{fmt, str::FromStr};

/// 六十甲子中的一个合法干支值，周期为 60。
///
/// 内部保证阴阳同性公理：天干与地支同阳或同阴。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SexagenaryCycle {
    stem: Stem,
    branch: Branch,
}

#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
#[serde(rename = "SexagenaryCycle", deny_unknown_fields)]
struct RawGanzhi {
    stem: Stem,
    branch: Branch,
}

#[cfg(feature = "serde")]
impl TryFrom<RawGanzhi> for SexagenaryCycle {
    type Error = InvalidGanzhi;
    fn try_from(raw: RawGanzhi) -> Result<Self, Self::Error> {
        Self::try_from((raw.stem, raw.branch))
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for SexagenaryCycle {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::{self, MapAccess, Visitor};
        struct RecordVisitor;
        impl<'de> Visitor<'de> for RecordVisitor {
            type Value = RawGanzhi;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("SexagenaryCycle object with stem and branch")
            }
            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                serde::Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
            }
        }
        let raw = if d.is_human_readable() {
            d.deserialize_map(RecordVisitor)?
        } else {
            serde::Deserialize::deserialize(d)?
        };
        Self::try_from(raw).map_err(de::Error::custom)
    }
}

impl TryFrom<(Stem, Branch)> for SexagenaryCycle {
    type Error = InvalidGanzhi;
    fn try_from((stem, branch): (Stem, Branch)) -> Result<Self, Self::Error> {
        Self::new(stem, branch).ok_or(InvalidGanzhi)
    }
}

impl TryFrom<u8> for SexagenaryCycle {
    type Error = InvalidIndex;
    fn try_from(index: u8) -> Result<Self, Self::Error> {
        Self::try_from_index(index)
    }
}

impl SexagenaryCycle {
    /// 构造干支，强制校验阴阳同性
    #[inline]
    pub const fn new(stem: Stem, branch: Branch) -> Option<Self> {
        if (stem as u8 % 2) == (branch as u8 % 2) {
            Some(Self { stem, branch })
        } else {
            None
        }
    }

    /// 获取天干
    #[inline]
    pub const fn stem(self) -> Stem {
        self.stem
    }

    /// 获取地支
    #[inline]
    pub const fn branch(self) -> Branch {
        self.branch
    }

    /// 所属的六旬身份，不附加空亡效应。
    pub fn xun(self) -> Xun {
        Xun::from_index(self.index() / 10)
    }

    /// 所属的三十类纳音身份；固定五行通过 [`Nayin::element`] 查询。
    pub fn nayin(self) -> Nayin {
        Nayin::ALL[usize::from(self.index() / 2)]
    }
}

impl CyclicRing for SexagenaryCycle {
    const MODULUS: core::num::NonZeroU8 = core::num::NonZeroU8::new(60).unwrap();

    /// 按 60 周期回绕构造 (0 = 甲子, 1 = 乙丑, ..., 59 = 癸亥)
    #[inline]
    fn from_index(idx: u8) -> Self {
        // 10 和 12 均整除 60，分别回绕与先按 60 回绕等价。
        let stem = Stem::from_index(idx);
        let branch = Branch::from_index(idx);
        Self { stem, branch }
    }

    /// 导出当前干支在六十甲子环中的序数 (0..=59)
    ///
    /// 利用中国剩余定理同余解算：s ≡ x (mod 10), b ≡ x (mod 12)
    /// 化简得：x = (6 * s - 5 * b) mod 60
    #[inline]
    fn index(self) -> u8 {
        let s = i64::from(self.stem.index());
        let b = i64::from(self.branch.index());
        crate::math::ring::wrap(6 * s - 5 * b, Self::MODULUS)
    }
}

/// 输出天干、地支的中文名称，不带分隔符；不改变 Serde 的对象表示。
impl fmt::Display for SexagenaryCycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.stem, self.branch)
    }
}

/// 严格解析两个中文字符组成的合法干支，不裁剪空白或接受别名。
///
/// ```
/// use matharts_core::{InvalidGanzhi, ParseError, SexagenaryCycle};
/// let value: SexagenaryCycle = "甲子".parse().unwrap();
/// assert_eq!(value.to_string(), "甲子");
/// assert_eq!(
///     "甲丑".parse::<SexagenaryCycle>(),
///     Err(ParseError::InvalidGanzhi(InvalidGanzhi)),
/// );
/// ```
impl FromStr for SexagenaryCycle {
    type Err = ParseError;

    /// # Errors
    /// 先检查字符数，非两个字符返回 [`ParseError::InvalidGanzhiFormat`]；
    /// 再依次检查天干、地支名称，返回 [`ParseError::InvalidStem`] 或
    /// [`ParseError::InvalidBranch`]；最后以 [`ParseError::InvalidGanzhi`]
    /// 报告阴阳不匹配的配对。
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut chars = input.char_indices();
        let (Some(_), Some((branch_start, _)), None) = (chars.next(), chars.next(), chars.next())
        else {
            return Err(ParseError::InvalidGanzhiFormat);
        };
        let stem: Stem = input[..branch_start].parse()?;
        let branch: Branch = input[branch_start..].parse()?;
        Self::try_from((stem, branch)).map_err(ParseError::InvalidGanzhi)
    }
}
