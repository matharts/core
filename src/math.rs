//! 有限周期的数值运算。
pub mod ring;

// 统筹导出环代数的核心 Trait 与独立纯函数
pub use ring::{CyclicRing, checked_forward_distance, forward_distance};
