//! 十二长生阶段值及阳顺阴逆、戊己随丙丁的天干查询。

use crate::branch::Branch;
use crate::math::CyclicSequence;
use crate::stem::Stem;
use crate::yin_yang::YinYang;

/// 十二长生寄生状态
///
/// 枚举只表示阶段及其固定序列。阶段本身不携带天干、地支、起点或顺逆规则；
/// [`CyclicSequence::offset`] 只移动阶段序号，不决定地支应当顺行还是逆行。
/// 查询天干在地支上的阶段，见 [`Stem::growth_phase_at`] 的采用定义。
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum GrowthPhase {
    /// 长生
    ChangSheng = 0,
    /// 沐浴
    MuYu = 1,
    /// 冠带
    GuanDai = 2,
    /// 临官
    LinGuan = 3,
    /// 帝旺
    DiWang = 4,
    /// 衰
    Shuai = 5,
    /// 病
    Bing = 6,
    /// 死
    Si = 7,
    /// 墓
    Mu = 8,
    /// 绝
    Jue = 9,
    /// 胎
    Tai = 10,
    /// 养
    Yang = 11,
}

impl GrowthPhase {
    /// 全部十二长生阶段，按索引顺序由长生至养排列，不表示强弱排序。
    pub const ALL: [Self; 12] = [
        Self::ChangSheng,
        Self::MuYu,
        Self::GuanDai,
        Self::LinGuan,
        Self::DiWang,
        Self::Shuai,
        Self::Bing,
        Self::Si,
        Self::Mu,
        Self::Jue,
        Self::Tai,
        Self::Yang,
    ];
}

/// 向前步进整数位移；与 [`CyclicSequence::offset`] 相同。
impl core::ops::Add<i32> for GrowthPhase {
    type Output = Self;
    fn add(self, rhs: i32) -> Self::Output {
        self.offset(rhs)
    }
}

/// 向后步进整数位移；支持全部 `i32`，包括 `i32::MIN`。
impl core::ops::Sub<i32> for GrowthPhase {
    type Output = Self;
    fn sub(self, rhs: i32) -> Self::Output {
        Self::from_index(crate::math::sequence::wrap(
            i64::from(self.index()) - i64::from(rhs),
            Self::MODULUS,
        ))
    }
}

/// 将整数位移的向前步进结果写回自身。
impl core::ops::AddAssign<i32> for GrowthPhase {
    fn add_assign(&mut self, rhs: i32) {
        *self = *self + rhs;
    }
}

/// 将整数位移的向后步进结果写回自身。
impl core::ops::SubAssign<i32> for GrowthPhase {
    fn sub_assign(&mut self, rhs: i32) {
        *self = *self - rhs;
    }
}

impl CyclicSequence for GrowthPhase {
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

impl Stem {
    /// 计算天干寄生在地支上的十二长生状态（阳顺阴逆，戊己随丙丁起点）。
    /// 此处采用固定起点表，不表示所有体系的长生规则。
    /// 返回值只表示该表中的阶段，不直接判定命盘旺衰或吉凶。
    ///
    /// # 采用定义与证据状态
    /// 来源版本：[《三命通会》四库全书本卷二《论天干阴阳生死》及《论地支》](https://zh.wikisource.org/wiki/三命通會_(四庫全書本)/卷02)。
    /// 十干起点采用甲亥、乙午、丙戊寅、丁己酉、庚巳、辛子、壬申、癸卯，
    /// 阳干顺行、阴干逆行；消费场景是子平命理按已确定的干、支查询阶段。
    ///
    /// 同输入对照：戊寅在本表为长生，与该本《论天干阴阳生死》的戊土起点相符；
    /// 戊申在本表为病，而《论地支》另有申为水土长生之地的表述。
    /// 后者不能直接当作本十干表的起点；本接口没有五行长生表或流派选择参数。
    /// 120 组固定输入由 `tests/rules.rs` 独立表锁定。
    /// 转录尚未逐字对照影印本，跨体系的同输入阶段对照及消费方接入证据待补；
    /// 现有行为保留，不据此推定五行长生或其他体系也采用本表。
    #[inline]
    pub fn growth_phase_at(self, branch: Branch) -> GrowthPhase {
        // 各干长生起点地支
        let start_branch = match self {
            Stem::Jia => Branch::Hai,             // 阳木生亥
            Stem::Yi => Branch::Wu,               // 阴木生午
            Stem::Bing | Stem::Wu => Branch::Yin, // 阳火土生寅
            Stem::Ding | Stem::Ji => Branch::You, // 阴火土生酉
            Stem::Geng => Branch::Si,             // 阳金生巳
            Stem::Xin => Branch::Zi,              // 阴金生子
            Stem::Ren => Branch::Shen,            // 阳水生申
            Stem::Gui => Branch::Mao,             // 阴水生卯
        };

        if self.yin_yang() == YinYang::Yang {
            // 阳干顺推
            let distance = start_branch.distance_to(branch);
            GrowthPhase::from_index(distance)
        } else {
            // 阴干逆推
            let distance = branch.distance_to(start_branch);
            GrowthPhase::from_index(distance)
        }
    }
}

impl TryFrom<u8> for GrowthPhase {
    type Error = crate::InvalidIndex;
    fn try_from(index: u8) -> Result<Self, Self::Error> {
        Self::try_from_index(index)
    }
}

// 可读格式只接受代码字符串；紧凑格式使用原生枚举模型。

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for GrowthPhase {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{self, EnumAccess, VariantAccess};
        const CODES: &[&str; GrowthPhase::ALL.len()] = &[
            "ChangSheng",
            "MuYu",
            "GuanDai",
            "LinGuan",
            "DiWang",
            "Shuai",
            "Bing",
            "Si",
            "Mu",
            "Jue",
            "Tai",
            "Yang",
        ];
        struct Identifier;
        impl<'de> de::DeserializeSeed<'de> for Identifier {
            type Value = GrowthPhase;
            fn deserialize<D: serde::Deserializer<'de>>(
                self,
                d: D,
            ) -> Result<Self::Value, D::Error> {
                d.deserialize_identifier(self)
            }
        }
        impl de::Visitor<'_> for Identifier {
            type Value = GrowthPhase;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("variant identifier")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                match value {
                    "ChangSheng" => Ok(GrowthPhase::ChangSheng),
                    "MuYu" => Ok(GrowthPhase::MuYu),
                    "GuanDai" => Ok(GrowthPhase::GuanDai),
                    "LinGuan" => Ok(GrowthPhase::LinGuan),
                    "DiWang" => Ok(GrowthPhase::DiWang),
                    "Shuai" => Ok(GrowthPhase::Shuai),
                    "Bing" => Ok(GrowthPhase::Bing),
                    "Si" => Ok(GrowthPhase::Si),
                    "Mu" => Ok(GrowthPhase::Mu),
                    "Jue" => Ok(GrowthPhase::Jue),
                    "Tai" => Ok(GrowthPhase::Tai),
                    "Yang" => Ok(GrowthPhase::Yang),
                    _ => Err(E::unknown_variant(value, CODES)),
                }
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                u8::try_from(value)
                    .ok()
                    .and_then(|index| GrowthPhase::try_from(index).ok())
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
            type Value = GrowthPhase;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("GrowthPhase code string")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                de::Visitor::visit_str(Identifier, value)
            }
        }
        struct EnumVisitor;
        impl<'de> de::Visitor<'de> for EnumVisitor {
            type Value = GrowthPhase;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("enum GrowthPhase")
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
            deserializer.deserialize_enum("GrowthPhase", CODES, EnumVisitor)
        }
    }
}
