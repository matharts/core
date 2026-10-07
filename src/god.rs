//! 明确主客体的十神分类；不推断命盘中的吉凶。

use crate::element::ElementRelation;
use crate::stem::Stem;

/// 十神
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum God {
    /// 比肩 (同我，同极性)
    BiJian = 0,
    /// 劫财 (同我，异极性)
    JieCai = 1,
    /// 食神 (我生，同极性)
    ShiShen = 2,
    /// 伤官 (我生，异极性)
    ShangGuan = 3,
    /// 偏财 (我克，同极性)
    PianCai = 4,
    /// 正财 (我克，异极性)
    ZhengCai = 5,
    /// 七杀 (克我，同极性)
    QiSha = 6,
    /// 正官 (克我，异极性)
    ZhengGuan = 7,
    /// 偏印 (生我，同极性)
    ///
    /// “枭神”可作别称；此值只表示偏印关系，不表示枭夺食已经成立。
    /// 《三命通会·论倒食》区分无食时只论偏印，具体作用需结合上下文判断。
    /// 原文转录：[《论倒食》](https://www.shidianguji.com/zh/book/SK1610/chapter/1mecpymm3r8rv)。
    PianYin = 8,
    /// 正印 (生我，异极性)
    ZhengYin = 9,
}

impl Stem {
    /// 以自身为参照，查询目标天干的十神身份。
    ///
    /// 自身通常是日主，但也可以是调用方指定的其他参照天干；交换主客体会改变结果。
    ///
    /// # 采用定义与证据状态
    /// 来源版本：[《三命通会》四库全书本卷五《论古人立印食官财名义》](https://zh.wikisource.org/wiki/三命通會_(四庫全書本)/卷05)。
    /// 采用正五行生克与阴阳同异划分十神；消费场景是子平命理以日干为参照
    /// 给另一干分类，也允许调用方显式指定其他参照干。
    /// 同输入对照：甲见丙为食神、甲见辛为正官，与该篇举例相符；
    /// 丙见甲按本表为偏印，交换方向不保持原十神身份。
    /// 原文中的格局、喜忌及偏印的实际作用不由这个分类函数判断。
    /// 全部 100 个有序输入由 `tests/rules.rs` 独立表验收。
    /// 转录的影印本校勘、其他体系是否采用相同参照语义及实际消费方对照
    /// 证据待补；这些缺口不由固定表测试或单个偏印条目的来源替代。
    ///
    /// ```
    /// use matharts_core::{God, Stem};
    ///
    /// // 丙是甲的食神，甲是丙的偏印。
    /// assert_eq!(Stem::Jia.ten_god_of(Stem::Bing), God::ShiShen);
    /// assert_eq!(Stem::Bing.ten_god_of(Stem::Jia), God::PianYin);
    /// ```
    #[inline]
    pub fn ten_god_of(self, target: Stem) -> God {
        let same_primitive = self.primitive() == target.primitive();
        let relation = self.element().relation_to(target.element());

        match (relation, same_primitive) {
            (ElementRelation::Same, true) => God::BiJian,
            (ElementRelation::Same, false) => God::JieCai,
            (ElementRelation::Generates, true) => God::ShiShen,
            (ElementRelation::Generates, false) => God::ShangGuan,
            (ElementRelation::Overcomes, true) => God::PianCai,
            (ElementRelation::Overcomes, false) => God::ZhengCai,
            (ElementRelation::OvercomeBy, true) => God::QiSha,
            (ElementRelation::OvercomeBy, false) => God::ZhengGuan,
            (ElementRelation::GeneratedBy, true) => God::PianYin,
            (ElementRelation::GeneratedBy, false) => God::ZhengYin,
        }
    }
}

// 可读格式只接受代码字符串；紧凑格式使用原生枚举模型。

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for God {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{self, EnumAccess, VariantAccess};
        const CODES: &[&str] = &[
            "BiJian",
            "JieCai",
            "ShiShen",
            "ShangGuan",
            "PianCai",
            "ZhengCai",
            "QiSha",
            "ZhengGuan",
            "PianYin",
            "ZhengYin",
        ];
        struct Identifier;
        impl<'de> de::DeserializeSeed<'de> for Identifier {
            type Value = God;
            fn deserialize<D: serde::Deserializer<'de>>(
                self,
                d: D,
            ) -> Result<Self::Value, D::Error> {
                d.deserialize_identifier(self)
            }
        }
        impl de::Visitor<'_> for Identifier {
            type Value = God;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("variant identifier")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                match value {
                    "BiJian" => Ok(God::BiJian),
                    "JieCai" => Ok(God::JieCai),
                    "ShiShen" => Ok(God::ShiShen),
                    "ShangGuan" => Ok(God::ShangGuan),
                    "PianCai" => Ok(God::PianCai),
                    "ZhengCai" => Ok(God::ZhengCai),
                    "QiSha" => Ok(God::QiSha),
                    "ZhengGuan" => Ok(God::ZhengGuan),
                    "PianYin" => Ok(God::PianYin),
                    "ZhengYin" => Ok(God::ZhengYin),
                    _ => Err(E::unknown_variant(value, CODES)),
                }
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                match value {
                    0 => Ok(God::BiJian),
                    1 => Ok(God::JieCai),
                    2 => Ok(God::ShiShen),
                    3 => Ok(God::ShangGuan),
                    4 => Ok(God::PianCai),
                    5 => Ok(God::ZhengCai),
                    6 => Ok(God::QiSha),
                    7 => Ok(God::ZhengGuan),
                    8 => Ok(God::PianYin),
                    9 => Ok(God::ZhengYin),
                    _ => Err(E::invalid_value(
                        de::Unexpected::Unsigned(value),
                        &"variant index 0 <= i < 10",
                    )),
                }
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
            type Value = God;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("God code string")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                de::Visitor::visit_str(Identifier, value)
            }
        }
        struct EnumVisitor;
        impl<'de> de::Visitor<'de> for EnumVisitor {
            type Value = God;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("enum God")
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
            deserializer.deserialize_enum("God", CODES, EnumVisitor)
        }
    }
}
