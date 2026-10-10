//! 地支值、藏干记录、固定配对与完整三支关系组。

use crate::element::Element;
use crate::error::InvalidIndex;
use crate::math::CyclicSequence;
use crate::stem::Stem;
use crate::yin_yang::YinYang;
use core::{
    fmt,
    ops::{Add, Sub},
    str::FromStr,
};

/// 地支藏干容器：本气及两个按采用表排列的附加槽位。
///
/// 这是可自定义记录，不携带地支身份、规则来源或权重。
/// `secondary` 和 `tertiary` 独立可选；直接构造和反序列化都允许重复成员，
/// 也允许第二槽为空而第三槽存在，不会自动校验记录是否符合某个地支的藏干表。
/// 两个附加槽位不按权重排序，也不直接代表古籍按季节来源划分的中气、余气。
///
/// 需要本库采用表中的结果时，使用 [`Branch::hidden_stems`]；
/// 修改其返回记录后，仍只是自定义记录，不再保证与原查询结果相符。
///
/// ```
/// use matharts_core::{Branch, HiddenStems, Stem};
///
/// let selected = Branch::Zi.hidden_stems();
/// assert_eq!(selected.primary, Stem::Gui);
/// assert_eq!(selected.secondary, None);
/// assert_eq!(selected.tertiary, None);
///
/// // 容器允许这个自定义记录；它不是某个地支藏干的校验证明。
/// let custom = HiddenStems {
///     primary: Stem::Jia,
///     secondary: None,
///     tertiary: Some(Stem::Jia),
/// };
/// assert_eq!(custom.tertiary, Some(custom.primary));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct HiddenStems {
    /// 本气；此容器允许调用方自定义成员，不代表已校验的地支属性。
    pub primary: Stem,
    /// 第二槽；采用表中的首个附加藏干，不表示统一的中气类别或强弱等级。
    pub secondary: Option<Stem>,
    /// 第三槽；采用表中的另一个附加藏干，不等同于季节承接意义的余气。
    pub tertiary: Option<Stem>,
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for HiddenStems {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::{self, MapAccess, Visitor};
        // 字段必须出现；可选槽位用显式 null 表示为空。
        fn optional_stem<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<Stem>, D::Error> {
            serde::Deserialize::deserialize(d)
        }
        #[derive(serde::Deserialize)]
        #[serde(rename = "HiddenStems", deny_unknown_fields)]
        struct Record {
            primary: Stem,
            #[serde(deserialize_with = "optional_stem")]
            secondary: Option<Stem>,
            #[serde(deserialize_with = "optional_stem")]
            tertiary: Option<Stem>,
        }
        struct RecordVisitor;
        impl<'de> Visitor<'de> for RecordVisitor {
            type Value = Record;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("HiddenStems object with primary, secondary and tertiary")
            }
            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                serde::Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
            }
        }
        let raw: Record = if d.is_human_readable() {
            d.deserialize_map(RecordVisitor)?
        } else {
            serde::Deserialize::deserialize(d)?
        };
        Ok(Self {
            primary: raw.primary,
            secondary: raw.secondary,
            tertiary: raw.tertiary,
        })
    }
}

/// 十二地支 (0..=11)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[must_use]
pub enum Branch {
    /// 子
    Zi = 0,
    /// 丑
    Chou = 1,
    /// 寅
    Yin = 2,
    /// 卯
    Mao = 3,
    /// 辰
    Chen = 4,
    /// 巳
    Si = 5,
    /// 午
    Wu = 6,
    /// 未
    Wei = 7,
    /// 申
    Shen = 8,
    /// 酉
    You = 9,
    /// 戌
    Xu = 10,
    /// 亥
    Hai = 11,
}

impl CyclicSequence for Branch {
    const MODULUS: core::num::NonZeroU8 = core::num::NonZeroU8::new(12).unwrap();

    #[inline]
    fn from_index(idx: u8) -> Self {
        Self::ALL[(idx % Self::MODULUS.get()) as usize]
    }

    #[inline]
    fn index(self) -> u8 {
        self as u8
    }
}

impl Branch {
    /// 全部十二支，按索引顺序由子至亥排列。
    pub const ALL: [Self; 12] = [
        Self::Zi,
        Self::Chou,
        Self::Yin,
        Self::Mao,
        Self::Chen,
        Self::Si,
        Self::Wu,
        Self::Wei,
        Self::Shen,
        Self::You,
        Self::Xu,
        Self::Hai,
    ];

    /// 阴阳极性：传统一基序数奇阳偶阴，对应零基索引偶阳奇阴。
    #[inline]
    pub const fn yin_yang(self) -> YinYang {
        if (self as u8).is_multiple_of(2) {
            YinYang::Yang
        } else {
            YinYang::Yin
        }
    }

    /// 地支本气五行
    #[inline]
    pub const fn element(self) -> Element {
        match self {
            Self::Yin | Self::Mao => Element::Wood,
            Self::Si | Self::Wu => Element::Fire,
            Self::Chen | Self::Xu | Self::Chou | Self::Wei => Element::Earth,
            Self::Shen | Self::You => Element::Metal,
            Self::Hai | Self::Zi => Element::Water,
        }
    }

    /// 地支藏干：本气在首槽，其余成员按下表固定排列，不表示权重或旺衰。
    ///
    /// 本方法返回本库采用表中的记录，不接受外部记录进行校验。
    /// 槽位顺序是本库的固定表示，不把歌诀次序解释为气的等级，
    /// 也不计算月内司令天数。
    ///
    /// # 采用定义与证据状态
    /// 成员对照[《渊海子平》转录《又地支藏遁歌》](https://zh.wikisource.org/wiki/淵海子平#又地支藏遁歌)，
    /// 并以[《三命通会》四库全书本卷二《论地支》](https://zh.wikisource.org/wiki/三命通會_(四庫全書本)/卷02#論地支)
    /// 对照季节承接意义的余气。前一转录页未明确底本且标有来源未核实提示，
    /// 因而其刊本版本及影印校勘仍待补；不把网页转录当作已校勘的古籍版本。
    /// 消费场景是调用方已确定地支后，查询子平藏干成员；本方法不判断藏干透出、
    /// 权重、旺衰或月令用事。这些条件不能从槽位位置推定。
    ///
    /// | 支 | `primary` | `secondary` | `tertiary` |
    /// | --- | --- | --- | --- |
    /// | 子 | 癸 | — | — |
    /// | 丑 | 己 | 癸 | 辛 |
    /// | 寅 | 甲 | 丙 | 戊 |
    /// | 卯 | 乙 | — | — |
    /// | 辰 | 戊 | 乙 | 癸 |
    /// | 巳 | 丙 | 庚 | 戊 |
    /// | 午 | 丁 | 己 | — |
    /// | 未 | 己 | 丁 | 乙 |
    /// | 申 | 庚 | 壬 | 戊 |
    /// | 酉 | 辛 | — | — |
    /// | 戌 | 戊 | 辛 | 丁 |
    /// | 亥 | 壬 | 甲 | — |
    ///
    /// “—”表示 `None`。季节承接意义的余气丑癸、辰乙、未丁、戌辛位于
    /// `secondary`，不能直接从第三槽 `tertiary` 提取这类余气。
    /// 同输入对照：歌诀列丑的成员为癸、辛、己，本表返回己、癸、辛，成员相同而
    /// 排列不同；《论地支》称辰中的乙为余气，另以壬水墓后还魂说明癸，
    /// 不能把这些描述合并为统一的槽位等级。
    /// 12 支的成员及槽位由 `tests/rules.rs` 独立表验收。
    /// 不同体系的藏干成员、槽位含义及实际消费方同输入对照证据待补；
    /// 本库示例和表测试不构成其他体系已经接入的证明。
    ///
    /// # Examples
    /// ```
    /// use matharts_core::{Branch, HiddenStems, Stem};
    ///
    /// assert_eq!(Branch::Chou.hidden_stems(), HiddenStems {
    ///     primary: Stem::Ji,
    ///     secondary: Some(Stem::Gui),
    ///     tertiary: Some(Stem::Xin),
    /// });
    /// assert_eq!(Branch::Chen.hidden_stems().secondary, Some(Stem::Yi));
    /// assert_eq!(Branch::Chen.hidden_stems().tertiary, Some(Stem::Gui));
    /// ```
    #[inline]
    pub const fn hidden_stems(self) -> HiddenStems {
        match self {
            Branch::Zi => HiddenStems {
                primary: Stem::Gui,
                secondary: None,
                tertiary: None,
            },
            Branch::Chou => HiddenStems {
                primary: Stem::Ji,
                secondary: Some(Stem::Gui),
                tertiary: Some(Stem::Xin),
            },
            Branch::Yin => HiddenStems {
                primary: Stem::Jia,
                secondary: Some(Stem::Bing),
                tertiary: Some(Stem::Wu),
            },
            Branch::Mao => HiddenStems {
                primary: Stem::Yi,
                secondary: None,
                tertiary: None,
            },
            Branch::Chen => HiddenStems {
                primary: Stem::Wu,
                secondary: Some(Stem::Yi),
                tertiary: Some(Stem::Gui),
            },
            Branch::Si => HiddenStems {
                primary: Stem::Bing,
                secondary: Some(Stem::Geng),
                tertiary: Some(Stem::Wu),
            },
            Branch::Wu => HiddenStems {
                primary: Stem::Ding,
                secondary: Some(Stem::Ji),
                tertiary: None,
            },
            Branch::Wei => HiddenStems {
                primary: Stem::Ji,
                secondary: Some(Stem::Ding),
                tertiary: Some(Stem::Yi),
            },
            Branch::Shen => HiddenStems {
                primary: Stem::Geng,
                secondary: Some(Stem::Ren),
                tertiary: Some(Stem::Wu),
            },
            Branch::You => HiddenStems {
                primary: Stem::Xin,
                secondary: None,
                tertiary: None,
            },
            Branch::Xu => HiddenStems {
                primary: Stem::Wu,
                secondary: Some(Stem::Xin),
                tertiary: Some(Stem::Ding),
            },
            Branch::Hai => HiddenStems {
                primary: Stem::Ren,
                secondary: Some(Stem::Jia),
                tertiary: None,
            },
        }
    }

    /// 六害（相穿）：子未、丑午、寅巳、卯辰、申亥、酉戌
    #[inline]
    pub fn is_harming(self, target: Self) -> bool {
        crate::SixHarm::from_branches([self, target]).is_some()
    }

    /// 六破：子酉、卯午、辰丑、巳申、寅亥、未戌
    ///
    /// 采用《六壬大全》卷三《破》的六对双向关系，仅查询配对是否存在。
    /// 不等同于《古今图书集成》所录排除四孟的“破杀”，也不等同于
    /// 《五行大义》的“冲破”；返回值不判断上下文中的实际作用。
    /// 原文转录：[《六壬大全》卷三](https://libokang.com/zh-hant/guji/liuren/六壬大全/3/)。
    #[inline]
    pub fn is_breaking(self, target: Self) -> bool {
        crate::SixBreak::from_branches([self, target]).is_some()
    }

    /// 查询有向相刑配对；自刑仅表示同名两支属于表中组合，不判断事件是否发生。
    ///
    /// # 采用定义与证据状态
    /// 来源版本：[《三命通会》四库全书本卷二《论三刑》转录](https://zh.wikisource.org/wiki/三命通會_(四庫全書本)/卷02#論三刑)。
    /// 消费场景是命理调用方按此表查询有序两支；不是完整三支集合识别。
    /// 同输入对照：寅刑巳返回 `true`，巳刑寅返回 `false`；辰见辰属于自刑配对。
    /// 类别注释采用该篇一说：寅巳申为无恩，丑戌未为恃势。
    /// 同篇引《三车一览》对调这两组名称并认可其说，故此处并非唯一命名。
    /// 名称差异不改变本方法采用的有向配对；不查询刑的强弱或发生条件。
    /// 全部 144 个有序输入由 `tests/rules.rs` 独立表验收。
    /// 转录字形的影印本校勘、其他体系的方向与自刑条件、实际消费方对照证据待补。
    #[inline]
    pub fn is_punishing(self, target: Self) -> bool {
        match self {
            // 无礼之刑：子刑卯，卯刑子
            Branch::Zi => target == Branch::Mao,
            Branch::Mao => target == Branch::Zi,
            // 无恩之刑：寅刑巳，巳刑申，申刑寅
            Branch::Yin => target == Branch::Si,
            Branch::Si => target == Branch::Shen,
            Branch::Shen => target == Branch::Yin,
            // 恃势之刑：丑刑戌，戌刑未，未刑丑
            Branch::Chou => target == Branch::Xu,
            Branch::Xu => target == Branch::Wei,
            Branch::Wei => target == Branch::Chou,
            // 自刑：辰午酉亥
            Branch::Chen | Branch::Wu | Branch::You | Branch::Hai => self == target,
        }
    }

    /// 六冲地支（对宫，相距 6 步）
    #[inline]
    pub fn opposite(self) -> Self {
        self.offset(6)
    }

    /// 本支所属的完整六冲配对，不判断实际冲动或吉凶。
    pub const fn six_clash(self) -> crate::SixClash {
        crate::SixClash::from_branch(self)
    }

    /// 本支所属的完整六害配对，不判断受害程度或应用效果。
    pub const fn six_harm(self) -> crate::SixHarm {
        crate::SixHarm::from_branch(self)
    }

    /// 本支在《六壬大全》采用六破表中的配对，不代表所有体系的“破”。
    pub const fn six_break(self) -> crate::SixBreak {
        crate::SixBreak::from_branch(self)
    }

    /// 本支所属的完整六合组，不判断实际成合或合化。
    /// 成员及伙伴由 [`crate::SixCombination`] 查询。
    pub const fn six_combination(self) -> crate::SixCombination {
        crate::SixCombination::from_branch(self)
    }

    /// 六合配对中的另一支；沿用 [`Self::six_combination`] 的固定表，不判断成合。
    pub const fn six_combination_partner(self) -> Self {
        let [first, second] = self.six_combination().members();
        if self as u8 == first as u8 {
            second
        } else {
            first
        }
    }

    /// 六冲配对中的另一支；沿用 [`Self::six_clash`] 的固定表，不判断实际作用。
    pub const fn six_clash_partner(self) -> Self {
        let [first, second] = self.six_clash().members();
        if self as u8 == first as u8 {
            second
        } else {
            first
        }
    }

    /// 六害配对中的另一支；沿用 [`Self::six_harm`] 的固定表，不判断实际作用。
    pub const fn six_harm_partner(self) -> Self {
        let [first, second] = self.six_harm().members();
        if self as u8 == first as u8 {
            second
        } else {
            first
        }
    }

    /// 采用六破表中的另一支；来源及体系限制与 [`Self::six_break`] 相同。
    pub const fn six_break_partner(self) -> Self {
        let [first, second] = self.six_break().members();
        if self as u8 == first as u8 {
            second
        } else {
            first
        }
    }

    /// 本支所属的完整三合组；成员缺失时也能查询身份，不判断实际成局。
    pub const fn three_combination(self) -> crate::ThreeCombination {
        crate::ThreeCombination::from_branch(self)
    }

    /// 本支所属的完整三会（方合）组，不判断时令、旺衰或成化。
    pub const fn three_meeting(self) -> crate::ThreeMeeting {
        crate::ThreeMeeting::from_branch(self)
    }

    /// 判定两支是否构成六冲
    #[inline]
    pub fn is_clashing_with(self, target: Self) -> bool {
        crate::SixClash::from_branches([self, target]).is_some()
    }
}

/// 将整数位移的向前步进结果写回自身。
impl core::ops::AddAssign<i32> for Branch {
    fn add_assign(&mut self, rhs: i32) {
        *self = *self + rhs;
    }
}

/// 将整数位移的向后步进结果写回自身。
impl core::ops::SubAssign<i32> for Branch {
    fn sub_assign(&mut self, rhs: i32) {
        *self = *self - rhs;
    }
}

impl Add<i32> for Branch {
    type Output = Self;
    #[inline]
    fn add(self, rhs: i32) -> Self::Output {
        self.offset(rhs)
    }
}

/// 向后步进整数位移；支持全部 `i32`，包括 `i32::MIN`。
impl Sub<i32> for Branch {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: i32) -> Self::Output {
        Self::from_index(crate::math::sequence::wrap(
            i64::from(self.index()) - i64::from(rhs),
            Self::MODULUS,
        ))
    }
}

impl TryFrom<u8> for Branch {
    type Error = crate::InvalidIndex;
    fn try_from(index: u8) -> Result<Self, Self::Error> {
        Self::try_from_index(index)
    }
}

/// 输出精确中文地支名，与 [`FromStr`] 的输入一致；不改变 Serde 编码。
impl fmt::Display for Branch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Zi => "子",
            Self::Chou => "丑",
            Self::Yin => "寅",
            Self::Mao => "卯",
            Self::Chen => "辰",
            Self::Si => "巳",
            Self::Wu => "午",
            Self::Wei => "未",
            Self::Shen => "申",
            Self::You => "酉",
            Self::Xu => "戌",
            Self::Hai => "亥",
        })
    }
}

/// 只接受精确中文地支名，不裁剪空白，不接受拼音或其他别名。
///
/// ```
/// use matharts_core::{Branch, ParseError};
/// assert_eq!("子".parse::<Branch>(), Ok(Branch::Zi));
/// assert_eq!(Branch::Zi.to_string(), "子");
/// assert_eq!("Zi".parse::<Branch>(), Err(ParseError::InvalidBranch));
/// ```
impl FromStr for Branch {
    type Err = crate::ParseError;

    /// # Errors
    /// 输入不是精确中文地支名时返回 [`crate::ParseError::InvalidBranch`]。
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "子" => Ok(Self::Zi),
            "丑" => Ok(Self::Chou),
            "寅" => Ok(Self::Yin),
            "卯" => Ok(Self::Mao),
            "辰" => Ok(Self::Chen),
            "巳" => Ok(Self::Si),
            "午" => Ok(Self::Wu),
            "未" => Ok(Self::Wei),
            "申" => Ok(Self::Shen),
            "酉" => Ok(Self::You),
            "戌" => Ok(Self::Xu),
            "亥" => Ok(Self::Hai),
            _ => Err(Self::Err::InvalidBranch),
        }
    }
}

// 可读格式只接受代码字符串；紧凑格式使用原生枚举模型。

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Branch {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{self, EnumAccess, VariantAccess};
        const CODES: &[&str; Branch::ALL.len()] = &[
            "Zi", "Chou", "Yin", "Mao", "Chen", "Si", "Wu", "Wei", "Shen", "You", "Xu", "Hai",
        ];
        struct Identifier;
        impl<'de> de::DeserializeSeed<'de> for Identifier {
            type Value = Branch;
            fn deserialize<D: serde::Deserializer<'de>>(
                self,
                d: D,
            ) -> Result<Self::Value, D::Error> {
                d.deserialize_identifier(self)
            }
        }
        impl de::Visitor<'_> for Identifier {
            type Value = Branch;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("variant identifier")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                CODES
                    .iter()
                    .position(|code| *code == value)
                    .map(|index| Branch::ALL[index])
                    .ok_or_else(|| E::unknown_variant(value, CODES))
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                usize::try_from(value)
                    .ok()
                    .and_then(|index| Branch::ALL.get(index))
                    .copied()
                    .ok_or_else(|| {
                        E::invalid_value(
                            de::Unexpected::Unsigned(value),
                            &"variant index 0 <= i < 12",
                        )
                    })
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
            type Value = Branch;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("Branch code string")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                de::Visitor::visit_str(Identifier, value)
            }
        }
        struct EnumVisitor;
        impl<'de> de::Visitor<'de> for EnumVisitor {
            type Value = Branch;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("enum Branch")
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
            deserializer.deserialize_enum("Branch", CODES, EnumVisitor)
        }
    }
}

// 六合配对及其固定成员。
/// 六种地支六合组，只提供固定配对，不携带合化五行或成立条件。
///
/// 成员按地支索引升序；关系可以与刑、破等同时存在。
///
/// # 采用定义与证据状态
/// 来源版本：[南秉吉《选择纪要》上编（1867）《地支六合》](https://zh.wikisource.org/wiki/選擇紀要/上編)。
/// 消费场景是择日或命理中的固定两支配对识别。
/// 例如子丑返回 `ZiChou`，子子不构成完整配对；古籍的合化五行及其应用条件
/// 不属于本类型的输出。成员、完整识别和编码由 `tests/pair_groups.rs` 独立验收。
/// 转录的影印本校勘及跨体系实际消费方对照证据待补；保持当前成员定义。
///
/// ```
/// use matharts_core::{Branch, SixCombination};
/// let group = Branch::Hai.six_combination();
/// assert_eq!(group, SixCombination::YinHai);
/// assert_eq!(group.members(), [Branch::Yin, Branch::Hai]);
/// assert_eq!(group.partner_of(Branch::Hai), Some(Branch::Yin));
/// assert_eq!(group.partner_of(Branch::Zi), None);
/// assert_eq!(SixCombination::from_branches([Branch::Hai, Branch::Yin]), Some(group));
/// assert_eq!(SixCombination::from_branches([Branch::Zi, Branch::Yin]), None);
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[must_use]
pub enum SixCombination {
    /// 子丑组。
    ZiChou = 0,
    /// 寅亥组。
    YinHai = 1,
    /// 卯戌组。
    MaoXu = 2,
    /// 辰酉组。
    ChenYou = 3,
    /// 巳申组。
    SiShen = 4,
    /// 午未组。
    WuWei = 5,
}

impl SixCombination {
    /// 全部六组，按身份索引排列；不表示强弱或应用顺序。
    pub const ALL: [Self; 6] = [
        Self::ZiChou,
        Self::YinHai,
        Self::MaoXu,
        Self::ChenYou,
        Self::SiShen,
        Self::WuWei,
    ];

    /// 固定零基身份索引，范围为 `0..6`；不提供周期回绕。
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// 完整两支，按地支索引升序返回，不保留输入次序。
    pub const fn members(self) -> [Branch; 2] {
        match self {
            Self::ZiChou => [Branch::Zi, Branch::Chou],
            Self::YinHai => [Branch::Yin, Branch::Hai],
            Self::MaoXu => [Branch::Mao, Branch::Xu],
            Self::ChenYou => [Branch::Chen, Branch::You],
            Self::SiShen => [Branch::Si, Branch::Shen],
            Self::WuWei => [Branch::Wu, Branch::Wei],
        }
    }

    /// 本支是否属于此固定配对。
    pub const fn contains(self, branch: Branch) -> bool {
        Self::from_branch(branch).index() == self.index()
    }

    /// 查询给定成员的另一支；非本组成员返回 `None`。
    pub const fn partner_of(self, branch: Branch) -> Option<Branch> {
        let [a, b] = self.members();
        if branch as u8 == a as u8 {
            Some(b)
        } else if branch as u8 == b as u8 {
            Some(a)
        } else {
            None
        }
    }

    /// 单支的唯一所属组，不表示完整配对已经出现。
    pub const fn from_branch(branch: Branch) -> Self {
        match branch {
            Branch::Zi | Branch::Chou => Self::ZiChou,
            Branch::Yin | Branch::Hai => Self::YinHai,
            Branch::Mao | Branch::Xu => Self::MaoXu,
            Branch::Chen | Branch::You => Self::ChenYou,
            Branch::Si | Branch::Shen => Self::SiShen,
            Branch::Wu | Branch::Wei => Self::WuWei,
        }
    }

    /// 识别完整无序配对，拒绝重复成员与混组；不判断成合或合化。
    pub const fn from_branches([a, b]: [Branch; 2]) -> Option<Self> {
        let group = Self::from_branch(a);
        if a as u8 != b as u8 && group.contains(b) {
            Some(group)
        } else {
            None
        }
    }
}

impl TryFrom<u8> for SixCombination {
    type Error = InvalidIndex;

    /// # Errors
    /// 索引不在 `0..6` 时返回错误，不做周期约减。
    fn try_from(index: u8) -> Result<Self, Self::Error> {
        Self::ALL
            .get(usize::from(index))
            .copied()
            .ok_or(InvalidIndex {
                index,
                upper_bound: 6,
            })
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for SixCombination {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::{self, EnumAccess, VariantAccess};
        const CODES: &[&str; SixCombination::ALL.len()] =
            &["ZiChou", "YinHai", "MaoXu", "ChenYou", "SiShen", "WuWei"];
        struct Identifier;
        impl<'de> de::DeserializeSeed<'de> for Identifier {
            type Value = SixCombination;
            fn deserialize<I: serde::Deserializer<'de>>(
                self,
                d: I,
            ) -> Result<SixCombination, I::Error> {
                d.deserialize_identifier(self)
            }
        }
        impl de::Visitor<'_> for Identifier {
            type Value = SixCombination;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("variant identifier")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<SixCombination, E> {
                CODES
                    .iter()
                    .position(|code| *code == value)
                    .map(|index| SixCombination::ALL[index])
                    .ok_or_else(|| E::unknown_variant(value, CODES))
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<SixCombination, E> {
                u8::try_from(value)
                    .ok()
                    .and_then(|index| SixCombination::try_from(index).ok())
                    .ok_or_else(|| {
                        E::invalid_value(
                            de::Unexpected::Unsigned(value),
                            &"variant index 0 <= i < 6",
                        )
                    })
            }
            fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<SixCombination, E> {
                match core::str::from_utf8(value) {
                    Ok(code) => self.visit_str(code),
                    Err(_) => Err(E::invalid_value(de::Unexpected::Bytes(value), &self)),
                }
            }
        }
        struct CodeVisitor;
        impl de::Visitor<'_> for CodeVisitor {
            type Value = SixCombination;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("SixCombination code string")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<SixCombination, E> {
                de::Visitor::visit_str(Identifier, value)
            }
        }
        struct EnumVisitor;
        impl<'de> de::Visitor<'de> for EnumVisitor {
            type Value = SixCombination;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("enum SixCombination")
            }
            fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<SixCombination, A::Error> {
                let (value, variant) = data.variant_seed(Identifier)?;
                variant.unit_variant()?;
                Ok(value)
            }
        }
        if d.is_human_readable() {
            d.deserialize_str(CodeVisitor)
        } else {
            d.deserialize_enum("SixCombination", CODES, EnumVisitor)
        }
    }
}

// 六冲配对及其固定成员。
/// 六种地支六冲配对，即十二支的对宫；不判断实际冲动或吉凶。
///
/// # 采用定义与证据状态
/// 来源版本：[南秉吉《选择纪要》上编（1867）《地支相冲》](https://zh.wikisource.org/wiki/選擇紀要/上編#地支相沖)。
/// 采用子午、丑未、寅申、卯酉、辰戌、巳亥六对；消费场景是择日或命理的对宫成员识别。
/// 同输入对照：子午识别为 `ZiWu`，子丑不属于六冲；交换输入次序仍为同一配对。
/// 该节也用“冲破”指这些对宫，不等同于 [`SixBreak`] 的六破采用表。
/// 转录的影印本校勘及跨体系实际消费方对照证据待补；成员一致不证明作用条件一致。
///
/// ```
/// use matharts_core::{Branch, SixClash};
/// let pair = Branch::Zi.six_clash();
/// assert_eq!(pair, SixClash::ZiWu);
/// assert_eq!(pair.members(), [Branch::Zi, Branch::Wu]);
/// assert_eq!(pair.partner_of(Branch::Zi), Some(Branch::Wu));
/// assert_eq!(pair.partner_of(Branch::Chou), None);
/// assert_eq!(SixClash::from_branches([Branch::Wu, Branch::Zi]), Some(pair));
/// assert_eq!(SixClash::from_branches([Branch::Zi; 2]), None);
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[must_use]
pub enum SixClash {
    /// 子午配对。
    ZiWu = 0,
    /// 丑未配对。
    ChouWei = 1,
    /// 寅申配对。
    YinShen = 2,
    /// 卯酉配对。
    MaoYou = 3,
    /// 辰戌配对。
    ChenXu = 4,
    /// 巳亥配对。
    SiHai = 5,
}

impl SixClash {
    /// 全部六个身份，按零基索引排列，不表示强弱或应用顺序。
    pub const ALL: [Self; 6] = [
        Self::ZiWu,
        Self::ChouWei,
        Self::YinShen,
        Self::MaoYou,
        Self::ChenXu,
        Self::SiHai,
    ];

    /// 固定零基身份索引，范围为 `0..6`；不提供周期回绕。
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// 完整两支，按地支索引升序返回，不保留输入次序。
    pub const fn members(self) -> [Branch; 2] {
        match self {
            Self::ZiWu => [Branch::Zi, Branch::Wu],
            Self::ChouWei => [Branch::Chou, Branch::Wei],
            Self::YinShen => [Branch::Yin, Branch::Shen],
            Self::MaoYou => [Branch::Mao, Branch::You],
            Self::ChenXu => [Branch::Chen, Branch::Xu],
            Self::SiHai => [Branch::Si, Branch::Hai],
        }
    }

    /// 本支是否属于此固定配对。
    pub const fn contains(self, branch: Branch) -> bool {
        Self::from_branch(branch).index() == self.index()
    }

    /// 查询给定成员的另一支；非本组成员返回 `None`。
    pub const fn partner_of(self, branch: Branch) -> Option<Branch> {
        let [a, b] = self.members();
        if branch as u8 == a as u8 {
            Some(b)
        } else if branch as u8 == b as u8 {
            Some(a)
        } else {
            None
        }
    }

    /// 单支的唯一所属配对；不表示另一支已出现或实际作用成立。
    pub const fn from_branch(branch: Branch) -> Self {
        match branch {
            Branch::Zi | Branch::Wu => Self::ZiWu,
            Branch::Chou | Branch::Wei => Self::ChouWei,
            Branch::Yin | Branch::Shen => Self::YinShen,
            Branch::Mao | Branch::You => Self::MaoYou,
            Branch::Chen | Branch::Xu => Self::ChenXu,
            Branch::Si | Branch::Hai => Self::SiHai,
        }
    }

    /// 识别完整无序配对；拒绝重复与混组，不判断上下文中的作用。
    pub const fn from_branches([a, b]: [Branch; 2]) -> Option<Self> {
        let group = Self::from_branch(a);
        if a as u8 != b as u8 && group.contains(b) {
            Some(group)
        } else {
            None
        }
    }
}

impl TryFrom<u8> for SixClash {
    type Error = InvalidIndex;

    /// # Errors
    /// 索引不在 `0..6` 时返回错误，不做周期约减。
    fn try_from(index: u8) -> Result<Self, Self::Error> {
        Self::ALL
            .get(usize::from(index))
            .copied()
            .ok_or(InvalidIndex {
                index,
                upper_bound: 6,
            })
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for SixClash {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::{self, EnumAccess, VariantAccess};
        const CODES: &[&str; SixClash::ALL.len()] =
            &["ZiWu", "ChouWei", "YinShen", "MaoYou", "ChenXu", "SiHai"];
        struct Identifier;
        impl<'de> de::DeserializeSeed<'de> for Identifier {
            type Value = SixClash;
            fn deserialize<I: serde::Deserializer<'de>>(self, d: I) -> Result<SixClash, I::Error> {
                d.deserialize_identifier(self)
            }
        }
        impl de::Visitor<'_> for Identifier {
            type Value = SixClash;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("variant identifier")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<SixClash, E> {
                match value {
                    "ZiWu" => Ok(SixClash::ZiWu),
                    "ChouWei" => Ok(SixClash::ChouWei),
                    "YinShen" => Ok(SixClash::YinShen),
                    "MaoYou" => Ok(SixClash::MaoYou),
                    "ChenXu" => Ok(SixClash::ChenXu),
                    "SiHai" => Ok(SixClash::SiHai),
                    _ => Err(E::unknown_variant(value, CODES)),
                }
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<SixClash, E> {
                u8::try_from(value)
                    .ok()
                    .and_then(|index| SixClash::try_from(index).ok())
                    .ok_or_else(|| {
                        E::invalid_value(
                            de::Unexpected::Unsigned(value),
                            &"variant index 0 <= i < 6",
                        )
                    })
            }
            fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<SixClash, E> {
                match core::str::from_utf8(value) {
                    Ok(code) => self.visit_str(code),
                    Err(_) => Err(E::invalid_value(de::Unexpected::Bytes(value), &self)),
                }
            }
        }
        struct CodeVisitor;
        impl de::Visitor<'_> for CodeVisitor {
            type Value = SixClash;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("SixClash code string")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<SixClash, E> {
                de::Visitor::visit_str(Identifier, value)
            }
        }
        struct EnumVisitor;
        impl<'de> de::Visitor<'de> for EnumVisitor {
            type Value = SixClash;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("enum SixClash")
            }
            fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<SixClash, A::Error> {
                let (value, variant) = data.variant_seed(Identifier)?;
                variant.unit_variant()?;
                Ok(value)
            }
        }
        if d.is_human_readable() {
            d.deserialize_str(CodeVisitor)
        } else {
            d.deserialize_enum("SixClash", CODES, EnumVisitor)
        }
    }
}

// 六害配对及其固定成员。
/// 六种地支六害配对；集合无序不表示应用效果对称。
///
/// # 采用定义与证据状态
/// 来源版本：[《三命通会》四库全书本卷二《论六害》转录](https://zh.wikisource.org/wiki/三命通會_(四庫全書本)/卷02#論六害)及
/// [南秉吉《选择纪要》上编（1867）《地支六害》](https://zh.wikisource.org/wiki/選擇紀要/上編#地支六害一名穿心六害)。
/// 两处均列子未、丑午、寅巳、卯辰、申亥、酉戌；消费场景是命理或择日的固定成员识别。
/// 同输入对照：子未识别为 `ZiWei`，子午不属于六害；交换输入次序仍为同一配对。
/// 《论六害》对酉见戌与戌见酉的作用有不同论述；本类型的无序身份不携带这种方向性作用。
/// 只识别固定配对，不判断相穿、受害程度或实际吉凶。
/// 转录的影印本校勘及跨体系实际消费方对照证据待补；不将成员对称解释为效果对称。
///
/// ```
/// use matharts_core::{Branch, SixHarm};
/// let pair = Branch::Zi.six_harm();
/// assert_eq!(pair, SixHarm::ZiWei);
/// assert_eq!(pair.members(), [Branch::Zi, Branch::Wei]);
/// assert_eq!(SixHarm::from_branches([Branch::Wei, Branch::Zi]), Some(pair));
/// assert_eq!(SixHarm::from_branches([Branch::Zi, Branch::Wu]), None);
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[must_use]
pub enum SixHarm {
    /// 子未配对。
    ZiWei = 0,
    /// 丑午配对。
    ChouWu = 1,
    /// 寅巳配对。
    YinSi = 2,
    /// 卯辰配对。
    MaoChen = 3,
    /// 申亥配对。
    ShenHai = 4,
    /// 酉戌配对。
    YouXu = 5,
}

impl SixHarm {
    /// 全部六个身份，按零基索引排列，不表示强弱或应用顺序。
    pub const ALL: [Self; 6] = [
        Self::ZiWei,
        Self::ChouWu,
        Self::YinSi,
        Self::MaoChen,
        Self::ShenHai,
        Self::YouXu,
    ];

    /// 固定零基身份索引，范围为 `0..6`；不提供周期回绕。
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// 完整两支，按地支索引升序返回，不保留输入次序。
    pub const fn members(self) -> [Branch; 2] {
        match self {
            Self::ZiWei => [Branch::Zi, Branch::Wei],
            Self::ChouWu => [Branch::Chou, Branch::Wu],
            Self::YinSi => [Branch::Yin, Branch::Si],
            Self::MaoChen => [Branch::Mao, Branch::Chen],
            Self::ShenHai => [Branch::Shen, Branch::Hai],
            Self::YouXu => [Branch::You, Branch::Xu],
        }
    }

    /// 本支是否属于此固定配对。
    pub const fn contains(self, branch: Branch) -> bool {
        Self::from_branch(branch).index() == self.index()
    }

    /// 查询给定成员的另一支；非本组成员返回 `None`。
    pub const fn partner_of(self, branch: Branch) -> Option<Branch> {
        let [a, b] = self.members();
        if branch as u8 == a as u8 {
            Some(b)
        } else if branch as u8 == b as u8 {
            Some(a)
        } else {
            None
        }
    }

    /// 单支的唯一所属配对；不表示另一支已出现或实际作用成立。
    pub const fn from_branch(branch: Branch) -> Self {
        match branch {
            Branch::Zi | Branch::Wei => Self::ZiWei,
            Branch::Chou | Branch::Wu => Self::ChouWu,
            Branch::Yin | Branch::Si => Self::YinSi,
            Branch::Mao | Branch::Chen => Self::MaoChen,
            Branch::Shen | Branch::Hai => Self::ShenHai,
            Branch::You | Branch::Xu => Self::YouXu,
        }
    }

    /// 识别完整无序配对；拒绝重复与混组，不判断上下文中的作用。
    pub const fn from_branches([a, b]: [Branch; 2]) -> Option<Self> {
        let group = Self::from_branch(a);
        if a as u8 != b as u8 && group.contains(b) {
            Some(group)
        } else {
            None
        }
    }
}

impl TryFrom<u8> for SixHarm {
    type Error = InvalidIndex;

    /// # Errors
    /// 索引不在 `0..6` 时返回错误，不做周期约减。
    fn try_from(index: u8) -> Result<Self, Self::Error> {
        Self::ALL
            .get(usize::from(index))
            .copied()
            .ok_or(InvalidIndex {
                index,
                upper_bound: 6,
            })
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for SixHarm {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::{self, EnumAccess, VariantAccess};
        const CODES: &[&str; SixHarm::ALL.len()] =
            &["ZiWei", "ChouWu", "YinSi", "MaoChen", "ShenHai", "YouXu"];
        struct Identifier;
        impl<'de> de::DeserializeSeed<'de> for Identifier {
            type Value = SixHarm;
            fn deserialize<I: serde::Deserializer<'de>>(self, d: I) -> Result<SixHarm, I::Error> {
                d.deserialize_identifier(self)
            }
        }
        impl de::Visitor<'_> for Identifier {
            type Value = SixHarm;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("variant identifier")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<SixHarm, E> {
                CODES
                    .iter()
                    .position(|code| *code == value)
                    .map(|index| SixHarm::ALL[index])
                    .ok_or_else(|| E::unknown_variant(value, CODES))
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<SixHarm, E> {
                u8::try_from(value)
                    .ok()
                    .and_then(|index| SixHarm::try_from(index).ok())
                    .ok_or_else(|| {
                        E::invalid_value(
                            de::Unexpected::Unsigned(value),
                            &"variant index 0 <= i < 6",
                        )
                    })
            }
            fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<SixHarm, E> {
                match core::str::from_utf8(value) {
                    Ok(code) => self.visit_str(code),
                    Err(_) => Err(E::invalid_value(de::Unexpected::Bytes(value), &self)),
                }
            }
        }
        struct CodeVisitor;
        impl de::Visitor<'_> for CodeVisitor {
            type Value = SixHarm;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("SixHarm code string")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<SixHarm, E> {
                de::Visitor::visit_str(Identifier, value)
            }
        }
        struct EnumVisitor;
        impl<'de> de::Visitor<'de> for EnumVisitor {
            type Value = SixHarm;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("enum SixHarm")
            }
            fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<SixHarm, A::Error> {
                let (value, variant) = data.variant_seed(Identifier)?;
                variant.unit_variant()?;
                Ok(value)
            }
        }
        if d.is_human_readable() {
            d.deserialize_str(CodeVisitor)
        } else {
            d.deserialize_enum("SixHarm", CODES, EnumVisitor)
        }
    }
}

// 采用《六壬大全》表的六破配对及其固定成员。
/// 《六壬大全》卷三《破》采用的六种地支配对；不判断实际作用。
///
/// # 采用定义与证据状态
/// 来源定位：[《六壬大全》卷三《破》网页原文转录](https://libokang.com/zh-hant/guji/liuren/六壬大全/3/#破)。
/// 网页未明确底本版本，刊本定位及影印本校勘待补；不将网页译文当作原文证据。
/// 沿用 [`Branch::is_breaking`] 的子酉、丑辰、寅亥、卯午、巳申、未戌表；
/// 消费语义限定为六壬调用方在该表中查询成员，实际临日、入传和应用条件由调用方判断。
/// 同输入对照：原文列申破巳并说明六对可反向，本库将巳申识别为 `SiShen`；
/// 同时可属于六合，配对身份不互相排斥，也不包含原文的作用断语。
/// 不等同于排除四孟的破杀或表示对宫的冲破；不代表所有体系的“破”。
/// 其他体系是否采用同表、同输入语义及实际消费方对照证据待补。
///
/// ```
/// use matharts_core::{Branch, SixBreak, SixCombination};
/// let pair = Branch::Si.six_break();
/// assert_eq!(pair, SixBreak::SiShen);
/// assert_eq!(pair.members(), [Branch::Si, Branch::Shen]);
/// assert_eq!(SixBreak::from_branches([Branch::Shen, Branch::Si]), Some(pair));
/// assert_eq!(SixCombination::from_branches(pair.members()), Some(SixCombination::SiShen));
/// assert!(Branch::Si.is_punishing(Branch::Shen));
/// assert!(!Branch::Shen.is_punishing(Branch::Si));
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[must_use]
pub enum SixBreak {
    /// 子酉配对。
    ZiYou = 0,
    /// 丑辰配对。
    ChouChen = 1,
    /// 寅亥配对。
    YinHai = 2,
    /// 卯午配对。
    MaoWu = 3,
    /// 巳申配对。
    SiShen = 4,
    /// 未戌配对。
    WeiXu = 5,
}

impl SixBreak {
    /// 全部六个身份，按零基索引排列，不表示强弱或应用顺序。
    pub const ALL: [Self; 6] = [
        Self::ZiYou,
        Self::ChouChen,
        Self::YinHai,
        Self::MaoWu,
        Self::SiShen,
        Self::WeiXu,
    ];

    /// 固定零基身份索引，范围为 `0..6`；不提供周期回绕。
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// 完整两支，按地支索引升序返回，不保留输入次序。
    pub const fn members(self) -> [Branch; 2] {
        match self {
            Self::ZiYou => [Branch::Zi, Branch::You],
            Self::ChouChen => [Branch::Chou, Branch::Chen],
            Self::YinHai => [Branch::Yin, Branch::Hai],
            Self::MaoWu => [Branch::Mao, Branch::Wu],
            Self::SiShen => [Branch::Si, Branch::Shen],
            Self::WeiXu => [Branch::Wei, Branch::Xu],
        }
    }

    /// 本支是否属于此固定配对。
    pub const fn contains(self, branch: Branch) -> bool {
        Self::from_branch(branch).index() == self.index()
    }

    /// 查询给定成员的另一支；非本组成员返回 `None`。
    pub const fn partner_of(self, branch: Branch) -> Option<Branch> {
        let [a, b] = self.members();
        if branch as u8 == a as u8 {
            Some(b)
        } else if branch as u8 == b as u8 {
            Some(a)
        } else {
            None
        }
    }

    /// 单支的唯一所属配对；不表示另一支已出现或实际作用成立。
    pub const fn from_branch(branch: Branch) -> Self {
        match branch {
            Branch::Zi | Branch::You => Self::ZiYou,
            Branch::Chou | Branch::Chen => Self::ChouChen,
            Branch::Yin | Branch::Hai => Self::YinHai,
            Branch::Mao | Branch::Wu => Self::MaoWu,
            Branch::Si | Branch::Shen => Self::SiShen,
            Branch::Wei | Branch::Xu => Self::WeiXu,
        }
    }

    /// 识别完整无序配对；拒绝重复与混组，不判断上下文中的作用。
    pub const fn from_branches([a, b]: [Branch; 2]) -> Option<Self> {
        let group = Self::from_branch(a);
        if a as u8 != b as u8 && group.contains(b) {
            Some(group)
        } else {
            None
        }
    }
}

impl TryFrom<u8> for SixBreak {
    type Error = InvalidIndex;

    /// # Errors
    /// 索引不在 `0..6` 时返回错误，不做周期约减。
    fn try_from(index: u8) -> Result<Self, Self::Error> {
        Self::ALL
            .get(usize::from(index))
            .copied()
            .ok_or(InvalidIndex {
                index,
                upper_bound: 6,
            })
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for SixBreak {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::{self, EnumAccess, VariantAccess};
        const CODES: &[&str; SixBreak::ALL.len()] =
            &["ZiYou", "ChouChen", "YinHai", "MaoWu", "SiShen", "WeiXu"];
        struct Identifier;
        impl<'de> de::DeserializeSeed<'de> for Identifier {
            type Value = SixBreak;
            fn deserialize<I: serde::Deserializer<'de>>(self, d: I) -> Result<SixBreak, I::Error> {
                d.deserialize_identifier(self)
            }
        }
        impl de::Visitor<'_> for Identifier {
            type Value = SixBreak;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("variant identifier")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<SixBreak, E> {
                CODES
                    .iter()
                    .position(|code| *code == value)
                    .map(|index| SixBreak::ALL[index])
                    .ok_or_else(|| E::unknown_variant(value, CODES))
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<SixBreak, E> {
                u8::try_from(value)
                    .ok()
                    .and_then(|index| SixBreak::try_from(index).ok())
                    .ok_or_else(|| {
                        E::invalid_value(
                            de::Unexpected::Unsigned(value),
                            &"variant index 0 <= i < 6",
                        )
                    })
            }
            fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<SixBreak, E> {
                match core::str::from_utf8(value) {
                    Ok(code) => self.visit_str(code),
                    Err(_) => Err(E::invalid_value(de::Unexpected::Bytes(value), &self)),
                }
            }
        }
        struct CodeVisitor;
        impl de::Visitor<'_> for CodeVisitor {
            type Value = SixBreak;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("SixBreak code string")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<SixBreak, E> {
                de::Visitor::visit_str(Identifier, value)
            }
        }
        struct EnumVisitor;
        impl<'de> de::Visitor<'de> for EnumVisitor {
            type Value = SixBreak;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("enum SixBreak")
            }
            fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<SixBreak, A::Error> {
                let (value, variant) = data.variant_seed(Identifier)?;
                variant.unit_variant()?;
                Ok(value)
            }
        }
        if d.is_human_readable() {
            d.deserialize_str(CodeVisitor)
        } else {
            d.deserialize_enum("SixBreak", CODES, EnumVisitor)
        }
    }
}

// 三合完整组及其固定成员。
/// 四种完整三合组，不判断实际成局、成化或吉凶。
///
/// 成员输出按地支零基索引升序；身份代码的文字次序不是数组次序。
///
/// # 采用定义与证据状态
/// 来源版本：[《三命通会》四库全书本卷二《论支元三合》](https://zh.wikisource.org/wiki/三命通會_(四庫全書本)/卷02)及
/// [南秉吉《选择纪要》上编（1867）《地支三合》](https://zh.wikisource.org/wiki/選擇紀要/上編)。
/// 采用申子辰水、巳酉丑金、
/// 寅午戌火、亥卯未木；消费场景是命理或择日的完整三支成员识别。
/// 同输入对照：申子辰在两处均对应水，本库返回 `ShenZiChen` 及固定水行；
/// 申子子返回 `None`，不以重复成员补足第三支。
/// 原文的实际化局、神煞及吉凶解释均未移入本类型；其三合成员与本库数组的
/// 索引升序表示也不是同一种应用次序。
/// 全部 1,728 个有序输入及四组五行由 `tests/branch_groups.rs` 独立验收。
/// 转录存在字形误录，尚未逐字校勘影印本；跨体系实际消费方和应用结果
/// 对照证据待补。当前测试证明采用表的实现，不证明实际成局条件通用。
///
/// ```
/// use matharts_core::{Branch, Element, ThreeCombination};
/// let group = ThreeCombination::ShenZiChen;
/// assert_eq!(group.members(), [Branch::Zi, Branch::Chen, Branch::Shen]);
/// assert_eq!(group.element(), Element::Water);
/// assert_eq!(ThreeCombination::from_branches([Branch::Shen, Branch::Chen, Branch::Zi]), Some(group));
/// assert_eq!(ThreeCombination::from_branches([Branch::Zi; 3]), None);
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[must_use]
pub enum ThreeCombination {
    /// 申子辰组，对应水；不表示成员本气已经改变。
    ShenZiChen = 0,
    /// 巳酉丑组，对应金；不表示成员本气已经改变。
    SiYouChou = 1,
    /// 寅午戌组，对应火；不表示成员本气已经改变。
    YinWuXu = 2,
    /// 亥卯未组，对应木；不表示成员本气已经改变。
    HaiMaoWei = 3,
}

impl ThreeCombination {
    /// 全部四组，按稳定身份索引排列，不表示强弱或应用顺序。
    pub const ALL: [Self; 4] = [
        Self::ShenZiChen,
        Self::SiYouChou,
        Self::YinWuXu,
        Self::HaiMaoWei,
    ];

    /// 身份的固定零基索引，范围为 `0..4`。
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// 完整三支，按地支索引升序返回，不保留输入序列或原文次序。
    pub const fn members(self) -> [Branch; 3] {
        match self {
            Self::ShenZiChen => [Branch::Zi, Branch::Chen, Branch::Shen],
            Self::SiYouChou => [Branch::Chou, Branch::Si, Branch::You],
            Self::YinWuXu => [Branch::Yin, Branch::Wu, Branch::Xu],
            Self::HaiMaoWei => [Branch::Mao, Branch::Wei, Branch::Hai],
        }
    }

    /// 该固定组对应的五行，不判断时令、强弱或合化条件。
    pub const fn element(self) -> Element {
        match self {
            Self::ShenZiChen => Element::Water,
            Self::SiYouChou => Element::Metal,
            Self::YinWuXu => Element::Fire,
            Self::HaiMaoWei => Element::Wood,
        }
    }

    /// 该支是否属于此固定集合。
    pub fn contains(self, branch: Branch) -> bool {
        self.members().contains(&branch)
    }

    /// 单支在本类关系中的唯一所属组；不表示其他成员已出现在输入中。
    pub const fn from_branch(branch: Branch) -> Self {
        match branch {
            Branch::Zi | Branch::Chen | Branch::Shen => Self::ShenZiChen,
            Branch::Chou | Branch::Si | Branch::You => Self::SiYouChou,
            Branch::Yin | Branch::Wu | Branch::Xu => Self::YinWuXu,
            Branch::Mao | Branch::Wei | Branch::Hai => Self::HaiMaoWei,
        }
    }

    /// 识别完整的无序三支集合，拒绝重复成员或混组。
    ///
    /// 此结果只表示集合相等，应用层仍须保留并解释原始输入次序及上下文。
    pub fn from_branches([a, b, c]: [Branch; 3]) -> Option<Self> {
        if a == b || a == c || b == c {
            return None;
        }
        let group = Self::from_branch(a);
        if group.contains(b) && group.contains(c) {
            Some(group)
        } else {
            None
        }
    }
}

impl TryFrom<u8> for ThreeCombination {
    type Error = InvalidIndex;

    /// # Errors
    /// 当身份索引不在 `0..4` 时返回错误，不做周期回绕。
    fn try_from(index: u8) -> Result<Self, Self::Error> {
        match index {
            0 => Ok(Self::ShenZiChen),
            1 => Ok(Self::SiYouChou),
            2 => Ok(Self::YinWuXu),
            3 => Ok(Self::HaiMaoWei),
            _ => Err(InvalidIndex {
                index,
                upper_bound: 4,
            }),
        }
    }
}

// 可读格式只接受代码字符串；紧凑格式使用原生枚举模型。

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ThreeCombination {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{self, EnumAccess, VariantAccess};
        const CODES: &[&str; ThreeCombination::ALL.len()] =
            &["ShenZiChen", "SiYouChou", "YinWuXu", "HaiMaoWei"];
        struct Identifier;
        impl<'de> de::DeserializeSeed<'de> for Identifier {
            type Value = ThreeCombination;
            fn deserialize<D: serde::Deserializer<'de>>(
                self,
                d: D,
            ) -> Result<Self::Value, D::Error> {
                d.deserialize_identifier(self)
            }
        }
        impl de::Visitor<'_> for Identifier {
            type Value = ThreeCombination;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("variant identifier")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                CODES
                    .iter()
                    .position(|code| *code == value)
                    .map(|index| ThreeCombination::ALL[index])
                    .ok_or_else(|| E::unknown_variant(value, CODES))
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                usize::try_from(value)
                    .ok()
                    .and_then(|index| ThreeCombination::ALL.get(index))
                    .copied()
                    .ok_or_else(|| {
                        E::invalid_value(
                            de::Unexpected::Unsigned(value),
                            &"variant index 0 <= i < 4",
                        )
                    })
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
            type Value = ThreeCombination;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("ThreeCombination code string")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                de::Visitor::visit_str(Identifier, value)
            }
        }
        struct EnumVisitor;
        impl<'de> de::Visitor<'de> for EnumVisitor {
            type Value = ThreeCombination;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("enum ThreeCombination")
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
            deserializer.deserialize_enum("ThreeCombination", CODES, EnumVisitor)
        }
    }
}

// 三会（方合）完整组及其固定成员。
/// 四种完整三会（方合）组，不判断实际成局、成化或吉凶。
///
/// 成员输出按地支零基索引升序；身份代码的文字次序不是数组次序。
///
/// # 采用定义与证据状态
/// 采用的是同方三支定义；来源版本为[《古今图书集成》影印文件 Volume 471 第 38 页《属象》转录](https://zh.wikisource.org/wiki/Page:Gujin_Tushu_Jicheng,_Volume_471_(1700-1725).djvu/38)。
/// 原文列寅卯辰木、巳午未火、申酉戌金、亥子丑水；消费场景是命理方合的成员查询。
/// 同输入对照：寅卯辰按该同方定义返回 `YinMaoChen` 和木；
/// 寅午戌属于三合火组，本类型的完整组识别返回 `None`。
/// 名称须与[《三命通会》四库全书本卷二《论支元三合》](https://zh.wikisource.org/wiki/三命通會_(四庫全書本)/卷02)中另引的“三会”用语区分，
/// 不能按同名词自动套用这里的四组定义。
/// 全部 1,728 个有序输入由 `tests/branch_groups.rs` 独立验收。
/// 该页转录标记为未校对；影印本校勘、其他体系的同方语义和实际消费方对照证据待补；
/// 本库只提供已有成员及固定五行，不以同方身份推定实际成化。
///
/// ```
/// use matharts_core::{Branch, Element, ThreeMeeting};
/// let group = ThreeMeeting::YinMaoChen;
/// assert_eq!(group.members(), [Branch::Yin, Branch::Mao, Branch::Chen]);
/// assert_eq!(group.element(), Element::Wood);
/// assert_eq!(ThreeMeeting::from_branches([Branch::Chen, Branch::Mao, Branch::Yin]), Some(group));
/// assert_eq!(ThreeMeeting::from_branches([Branch::Zi; 3]), None);
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[must_use]
pub enum ThreeMeeting {
    /// 寅卯辰组，对应木；不表示成员本气已经改变。
    YinMaoChen = 0,
    /// 巳午未组，对应火；不表示成员本气已经改变。
    SiWuWei = 1,
    /// 申酉戌组，对应金；不表示成员本气已经改变。
    ShenYouXu = 2,
    /// 亥子丑组，对应水；不表示成员本气已经改变。
    HaiZiChou = 3,
}

impl ThreeMeeting {
    /// 全部四组，按稳定身份索引排列，不表示强弱或应用顺序。
    pub const ALL: [Self; 4] = [
        Self::YinMaoChen,
        Self::SiWuWei,
        Self::ShenYouXu,
        Self::HaiZiChou,
    ];

    /// 身份的固定零基索引，范围为 `0..4`。
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// 完整三支，按地支索引升序返回，不保留输入序列或原文次序。
    pub const fn members(self) -> [Branch; 3] {
        match self {
            Self::YinMaoChen => [Branch::Yin, Branch::Mao, Branch::Chen],
            Self::SiWuWei => [Branch::Si, Branch::Wu, Branch::Wei],
            Self::ShenYouXu => [Branch::Shen, Branch::You, Branch::Xu],
            Self::HaiZiChou => [Branch::Zi, Branch::Chou, Branch::Hai],
        }
    }

    /// 该固定组对应的五行，不判断时令、强弱或合化条件。
    pub const fn element(self) -> Element {
        match self {
            Self::YinMaoChen => Element::Wood,
            Self::SiWuWei => Element::Fire,
            Self::ShenYouXu => Element::Metal,
            Self::HaiZiChou => Element::Water,
        }
    }

    /// 该支是否属于此固定集合。
    pub fn contains(self, branch: Branch) -> bool {
        self.members().contains(&branch)
    }

    /// 单支在本类关系中的唯一所属组；不表示其他成员已出现在输入中。
    pub const fn from_branch(branch: Branch) -> Self {
        match branch {
            Branch::Yin | Branch::Mao | Branch::Chen => Self::YinMaoChen,
            Branch::Si | Branch::Wu | Branch::Wei => Self::SiWuWei,
            Branch::Shen | Branch::You | Branch::Xu => Self::ShenYouXu,
            Branch::Zi | Branch::Chou | Branch::Hai => Self::HaiZiChou,
        }
    }

    /// 识别完整的无序三支集合，拒绝重复成员或混组。
    ///
    /// 此结果只表示集合相等，应用层仍须保留并解释原始输入次序及上下文。
    pub fn from_branches([a, b, c]: [Branch; 3]) -> Option<Self> {
        if a == b || a == c || b == c {
            return None;
        }
        let group = Self::from_branch(a);
        if group.contains(b) && group.contains(c) {
            Some(group)
        } else {
            None
        }
    }
}

impl TryFrom<u8> for ThreeMeeting {
    type Error = InvalidIndex;

    /// # Errors
    /// 当身份索引不在 `0..4` 时返回错误，不做周期回绕。
    fn try_from(index: u8) -> Result<Self, Self::Error> {
        match index {
            0 => Ok(Self::YinMaoChen),
            1 => Ok(Self::SiWuWei),
            2 => Ok(Self::ShenYouXu),
            3 => Ok(Self::HaiZiChou),
            _ => Err(InvalidIndex {
                index,
                upper_bound: 4,
            }),
        }
    }
}

// 可读格式只接受代码字符串；紧凑格式使用原生枚举模型。

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ThreeMeeting {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{self, EnumAccess, VariantAccess};
        const CODES: &[&str; ThreeMeeting::ALL.len()] =
            &["YinMaoChen", "SiWuWei", "ShenYouXu", "HaiZiChou"];
        struct Identifier;
        impl<'de> de::DeserializeSeed<'de> for Identifier {
            type Value = ThreeMeeting;
            fn deserialize<D: serde::Deserializer<'de>>(
                self,
                d: D,
            ) -> Result<Self::Value, D::Error> {
                d.deserialize_identifier(self)
            }
        }
        impl de::Visitor<'_> for Identifier {
            type Value = ThreeMeeting;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("variant identifier")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                CODES
                    .iter()
                    .position(|code| *code == value)
                    .map(|index| ThreeMeeting::ALL[index])
                    .ok_or_else(|| E::unknown_variant(value, CODES))
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                usize::try_from(value)
                    .ok()
                    .and_then(|index| ThreeMeeting::ALL.get(index))
                    .copied()
                    .ok_or_else(|| {
                        E::invalid_value(
                            de::Unexpected::Unsigned(value),
                            &"variant index 0 <= i < 4",
                        )
                    })
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
            type Value = ThreeMeeting;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("ThreeMeeting code string")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                de::Visitor::visit_str(Identifier, value)
            }
        }
        struct EnumVisitor;
        impl<'de> de::Visitor<'de> for EnumVisitor {
            type Value = ThreeMeeting;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("enum ThreeMeeting")
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
            deserializer.deserialize_enum("ThreeMeeting", CODES, EnumVisitor)
        }
    }
}
