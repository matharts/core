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
/// # 采用定义与证据状态
/// 来源版本：[《三命通会》四库全书本卷二《论遁月时》转录](https://zh.wikisource.org/wiki/三命通會_(四庫全書本)/卷02#論遁月時)。
/// 采用该篇从寅起、顺行十二月的五组年干映射；消费场景是四柱调用方
/// 已确定年干序列与目标月支，再查询月干。文献的“正月”不替调用方决定
/// 农历月份或节气月的日期边界。
/// 同输入对照：甲、己年干配寅支均返回丙，配卯支均返回丁，与该篇起月示例相符；
/// 子、丑固定在本序列末尾，跨序列时须更换输入年干。
/// 转录在甲己组处出现“甲巳”字样；本库沿用天干五合的甲己组，
/// 与该篇“取天干合数”的定义及[另一转录的起月示例](https://zh.wikisource.org/wiki/三命通會/卷二#论遁月时)
/// 对照，不把转录字形认作底本已校勘的结论。
/// 全部 120 个年干、月支输入由 `tests/rules.rs` 独立表验收；
/// 影印本字形校勘及不同体系的历法前提、实际消费方同输入对照证据待补。
///
/// # Examples
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
/// # 采用定义与证据状态
/// 来源版本：[《三命通会》四库全书本卷二《论遁月时》转录](https://zh.wikisource.org/wiki/三命通會_(四庫全書本)/卷02#論遁月時)。
/// 采用从子起、顺行十二时的五组日干映射；消费场景是四柱调用方
/// 按自己的换日规则确定日干后，由时支查询时干。
/// 同输入对照：甲日干配子支返回甲、配丑支返回乙，与该篇甲子日起时示例相符；
/// 己日干使用相同起点。早子、晚子的差异由输入日干体现，函数不选择换日策略。
/// 甲己组的转录字形差异及本库采用依据见 [`derive_month_stem`]；
/// 全部 120 个日干、时支输入由 `tests/rules.rs` 独立表验收。
/// 影印本校勘及不同体系的换日前提、实际消费方同输入对照证据待补；
/// 本库示例与表测试不能替代调用方历法边界的端到端验证。
///
/// # Examples
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
