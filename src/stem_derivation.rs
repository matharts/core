//! 天干推导：五虎遁与五鼠遁的固定映射；调用方负责历法边界。

use crate::branch::Branch;
use crate::math::CyclicRing;
use crate::stem::Stem;

/// 五虎遁：年上起月干（以正月寅月为基准推算目标月支的天干）
///
/// `year_stem` 是调用方已经确定的、该寅月起始序列所属的年干。
/// `target_branch` 按寅、卯、辰、巳、午、未、申、酉、戌、亥、子、丑排列；
/// 子、丑是同一序列的最后两个月，不是该寅月之前的两个月。
/// 查询该寅月之前的子、丑时，调用方须传入前一序列所属的年干。
/// 本函数不接收日期，不判定节气、农历月份或民用年份的归属。
///
/// ```
/// use matharts_core::{Branch, Stem, derive_month_stem};
///
/// // 癸年序列的末尾，接到甲年序列的寅月。
/// assert_eq!(derive_month_stem(Stem::Gui, Branch::Zi), Stem::Jia);
/// assert_eq!(derive_month_stem(Stem::Gui, Branch::Chou), Stem::Yi);
/// assert_eq!(derive_month_stem(Stem::Jia, Branch::Yin), Stem::Bing);
/// // 甲年序列的子、丑在其寅月之后，不能与前一序列混用。
/// assert_eq!(derive_month_stem(Stem::Jia, Branch::Zi), Stem::Bing);
/// assert_eq!(derive_month_stem(Stem::Jia, Branch::Chou), Stem::Ding);
/// ```
///
/// 口诀法则：
/// - 甲己之年丙作首 (Bing = 2)
/// - 乙庚之岁戊为头 (Wu = 4)
/// - 丙辛必定寻庚起 (Geng = 6)
/// - 丁壬壬位顺行流 (Ren = 8)
/// - 戊癸甲寅好追求 (Jia = 0)
#[inline]
pub fn derive_month_stem(year_stem: Stem, target_branch: Branch) -> Stem {
    // 起始序数：(year_stem % 5) * 2 + 2，由 from_index 统一按十干周期回绕。
    let start_idx = (year_stem.index() % 5) * 2 + 2;
    // 从寅(2)顺推到目标地支的正向步长
    let delta = Branch::Yin.distance_to(target_branch);
    Stem::from_index(start_idx + delta)
}

/// 五鼠遁：日上起时干（以子时为基准推算目标时支的天干）
///
/// `day_stem` 是调用方已经按所选换日规则确定的日干。
/// `target_branch` 在该日干对应的子至亥序列内查询；函数不区分早子、晚子，
/// 不根据钟表时间调整日干，也不决定子时归属哪一天。
/// 同一时支传入不同日干可以得到不同结果，不能用本函数反推换日规则。
///
/// ```
/// use matharts_core::{Branch, Stem, derive_hour_stem};
///
/// // 调用方已确定从癸日序列转入甲日序列。
/// assert_eq!(derive_hour_stem(Stem::Gui, Branch::Hai), Stem::Gui);
/// assert_eq!(derive_hour_stem(Stem::Jia, Branch::Zi), Stem::Jia);
/// // 如果调用方确定子时仍采用癸日干，函数就按癸日起子时。
/// assert_eq!(derive_hour_stem(Stem::Gui, Branch::Zi), Stem::Ren);
/// ```
///
/// 口诀法则：
/// - 甲己还加甲 (Jia = 0)
/// - 乙庚丙作初 (Bing = 2)
/// - 丙辛从戊起 (Wu = 4)
/// - 丁壬庚子居 (Geng = 6)
/// - 戊癸何方发，壬子是真途 (Ren = 8)
#[inline]
pub fn derive_hour_stem(day_stem: Stem, target_branch: Branch) -> Stem {
    // 起始序数：(day_stem % 5) * 2，由 from_index 统一按十干周期回绕。
    let start_idx = (day_stem.index() % 5) * 2;
    // 从子(0)顺推到目标地支的正向步长
    let delta = Branch::Zi.distance_to(target_branch);
    Stem::from_index(start_idx + delta)
}
