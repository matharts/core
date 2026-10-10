//! `MathArts` 通用有限领域值：阴阳、五行、干支及显式命名的固定关系。
//!
//! 严格索引构造使用 `TryFrom<u8>`，循环操作使用 [`CyclicSequence`]。
//! 固定关系的返回值不判断实际成局、成化或吉凶。
#![no_std]
#![forbid(unsafe_code)]

// 通用周期机制。
pub mod math;

// 领域值与固定关系。
pub mod branch;
pub mod element;
pub mod error;
pub mod ganzhi;
pub mod growth_phase;
pub mod hexagram;
pub mod nayin;
pub mod stem;
pub mod stem_derivation;
pub mod ten_god;
pub mod trigram;
pub mod xun;
pub mod yin_yang;

// crate 根入口。
pub use branch::{
    Branch, HiddenStems, SixBreak, SixClash, SixCombination, SixHarm, ThreeCombination,
    ThreeMeeting,
};
pub use element::{Element, ElementRelation};
pub use error::{InvalidGanzhi, InvalidIndex, ParseError};
pub use ganzhi::Ganzhi;
pub use growth_phase::GrowthPhase;
pub use hexagram::{Hexagram, HexagramPosition};
pub use math::{CyclicSequence, checked_forward_distance, forward_distance};
pub use nayin::Nayin;
pub use stem::{FiveCombination, Stem};
pub use stem_derivation::{derive_hour_stem, derive_month_stem};
pub use ten_god::TenGod;
pub use trigram::{Trigram, TrigramPosition};
pub use xun::Xun;
pub use yin_yang::YinYang;
