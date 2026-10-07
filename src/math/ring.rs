//! 有限周期序列的索引和步进，不定义数学环的加法与乘法。

use crate::InvalidIndex;
use core::num::NonZeroU8;

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) const fn wrap(value: i64, period: NonZeroU8) -> u8 {
    // 余数严格位于 0..=254，转换不会丢失信息。
    value.rem_euclid(period.get() as i64) as u8
}

/// 从 `from` 到 `to` 的正向距离；索引允许回绕，周期由类型保证非零。
pub const fn forward_distance(from: u8, to: u8, period: NonZeroU8) -> u8 {
    wrap(to as i64 - from as i64, period)
}

/// 从外部整数周期计算距离；周期为零时返回 `None`。
pub const fn checked_forward_distance(from: u8, to: u8, period: u8) -> Option<u8> {
    match NonZeroU8::new(period) {
        Some(period) => Some(forward_distance(from, to, period)),
        None => None,
    }
}

/// 具有固定非零周期的值序列；索引顺序不代表强弱或吉凶。
///
/// 实现者必须保证 `index() < MODULUS.get()` 且索引与值一一对应。
/// `from_index` 对所有 `u8` 都必须按周期约减；严格构造使用 `try_from_index`。
/// 周期的非零性由关联常量的类型保证：
///
/// ```compile_fail
/// use matharts_core::CyclicRing;
/// use core::num::NonZeroU8;
/// #[derive(Clone, Copy, PartialEq, Eq)]
/// struct Invalid;
/// impl CyclicRing for Invalid {
///     const MODULUS: NonZeroU8 = NonZeroU8::new(0).unwrap();
///     fn from_index(_: u8) -> Self { Self }
///     fn index(self) -> u8 { 0 }
/// }
/// let _ = Invalid.offset(1);
/// ```
pub trait CyclicRing: Copy + Eq + Sized {
    /// 非零周期。
    const MODULUS: NonZeroU8;

    /// 按周期约减索引，允许回绕。
    fn from_index(idx: u8) -> Self;

    /// 严格索引构造，越界不自动归一化。
    ///
    /// # Errors
    /// 当 `idx >= MODULUS` 时返回索引错误。
    fn try_from_index(idx: u8) -> Result<Self, InvalidIndex> {
        let upper_bound = Self::MODULUS.get();
        if idx < upper_bound {
            Ok(Self::from_index(idx))
        } else {
            Err(InvalidIndex {
                index: idx,
                upper_bound,
            })
        }
    }

    /// 获取 `0..MODULUS` 内的索引。
    fn index(self) -> u8;

    /// 循环步进，支持全部 `i32` 位移，包括两端极值。
    #[must_use]
    fn offset(self, delta: i32) -> Self {
        Self::from_index(wrap(
            i64::from(self.index()) + i64::from(delta),
            Self::MODULUS,
        ))
    }

    /// 从自身到目标的正向距离，范围为 `0..MODULUS`；不是有符号差值。
    ///
    /// ```
    /// use matharts_core::{Branch, CyclicRing, Stem};
    /// assert_eq!(Stem::Jia.distance_to(Stem::Gui), 9);
    /// assert_eq!(Stem::Gui.distance_to(Stem::Jia), 1);
    /// assert_eq!(Branch::Hai.distance_to(Branch::Zi), 1);
    /// ```
    ///
    /// 周期值之间不使用减法表达距离：
    /// ```compile_fail
    /// use matharts_core::Stem;
    /// let _ = Stem::Jia - Stem::Gui;
    /// ```
    /// ```compile_fail
    /// use matharts_core::Branch;
    /// let _ = Branch::Zi - Branch::Hai;
    /// ```
    fn distance_to(self, target: Self) -> u8 {
        forward_distance(self.index(), target.index(), Self::MODULUS)
    }
}
