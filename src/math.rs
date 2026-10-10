//! 有限周期的数值运算。
pub mod sequence;

// 导出有限周期序列的索引、步进与距离工具。
pub use sequence::{CyclicSequence, checked_forward_distance, forward_distance};
