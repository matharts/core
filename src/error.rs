//! 外部输入违反有限领域边界时的错误。
use core::fmt;

/// 索引超出有限集合的取值范围。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidIndex {
    /// 调用方提供的索引。
    pub index: u8,
    /// 不包含在合法范围内的上界。
    pub upper_bound: u8,
}
impl fmt::Display for InvalidIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "index {} is outside 0..{}", self.index, self.upper_bound)
    }
}
impl core::error::Error for InvalidIndex {}

/// 干支阴阳不一致，无法组成六十甲子中的值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidGanzhi;
impl fmt::Display for InvalidGanzhi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("stem and branch must have the same polarity")
    }
}
impl core::error::Error for InvalidGanzhi {}

/// 严格中文文本解析失败；不保存输入文本，也不分配堆内存。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseError {
    /// 输入不是一个精确的中文五行名。
    InvalidElement,
    /// 输入不是一个精确的中文阴阳名。
    InvalidYinYang,
    /// 输入不是一个精确的中文八卦名。
    InvalidTrigram,
    /// 输入不是带“旬”字的精确中文六旬名。
    InvalidXun,
    /// 输入不是采用表中的精确中文纳音名。
    InvalidNayin,
    /// 输入不是一个精确的中文天干名。
    InvalidStem,
    /// 输入不是一个精确的中文地支名。
    InvalidBranch,
    /// 干支文本不是恰好两个 Unicode 字符。
    InvalidGanzhiFormat,
    /// 天干和地支名称有效，但阴阳不匹配。
    InvalidGanzhi(InvalidGanzhi),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidElement => f.write_str("expected one exact Chinese element name"),
            Self::InvalidYinYang => f.write_str("expected one exact Chinese yin-yang name"),
            Self::InvalidTrigram => f.write_str("expected one exact Chinese trigram name"),
            Self::InvalidXun => f.write_str("expected one exact Chinese xun name including 旬"),
            Self::InvalidNayin => f.write_str("expected one exact adopted Chinese nayin name"),
            Self::InvalidStem => f.write_str("expected one exact Chinese heavenly stem name"),
            Self::InvalidBranch => f.write_str("expected one exact Chinese earthly branch name"),
            Self::InvalidGanzhiFormat => {
                f.write_str("expected exactly two characters: a Chinese stem followed by a branch")
            }
            Self::InvalidGanzhi(error) => write!(f, "{error}"),
        }
    }
}

impl core::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::InvalidGanzhi(error) => Some(error),
            _ => None,
        }
    }
}
