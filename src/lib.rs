//! `MathArts` 通用有限领域值：阴阳、五行、干支及显式命名的固定关系。
//!
//! 严格索引构造使用 `TryFrom<u8>`，循环操作使用 [`CyclicRing`]。
//! 固定关系的返回值不判断实际成局、成化或吉凶。
#![no_std]
#![forbid(unsafe_code)]

// 1. 纯数学基础设施层
pub mod math;

// 2. 领域实体与原语层
pub mod branch;
pub mod element;
pub mod error;
pub mod god;
pub mod growth;
pub mod hexagram;
pub mod nayin;
pub mod primitive;
pub mod sexagenary;
pub mod stem;
pub mod stem_derivation;
pub mod trigram;
pub mod xun;

// 顶级平铺导出：对外暴露完整零心智负担的 API
pub use branch::{
    Branch, HiddenStems, SixBreak, SixClash, SixCombination, SixHarm, ThreeCombination,
    ThreeMeeting,
};
pub use element::{Element, ElementRelation};
pub use error::{InvalidGanzhi, InvalidIndex, ParseError};
pub use god::God;
pub use growth::Growth;
pub use hexagram::{Hexagram, HexagramPosition};
pub use math::{CyclicRing, checked_forward_distance, forward_distance};
pub use nayin::Nayin;
pub use primitive::Primitive;
pub use sexagenary::SexagenaryCycle;
pub use stem::{FiveCombination, Stem};
pub use stem_derivation::{derive_hour_stem, derive_month_stem};
pub use trigram::{Trigram, TrigramPosition};
pub use xun::Xun;
