//! 故障注入证明共同 Serde 枚举契约检查能拒绝模型变化和放宽输入。
#![cfg(feature = "serde")]

use core::fmt;
use matharts_core::{CyclicSequence, Stem};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

mod support;

const CASES: [(Stem, &str); 10] = [
    (Stem::Jia, "Jia"),
    (Stem::Yi, "Yi"),
    (Stem::Bing, "Bing"),
    (Stem::Ding, "Ding"),
    (Stem::Wu, "Wu"),
    (Stem::Ji, "Ji"),
    (Stem::Geng, "Geng"),
    (Stem::Xin, "Xin"),
    (Stem::Ren, "Ren"),
    (Stem::Gui, "Gui"),
];

#[derive(Clone, Copy, Debug, PartialEq)]
struct Probe<const CHANGE: u8>(Stem);

impl<const CHANGE: u8> Serialize for Probe<CHANGE> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let index = u32::from(self.0.index());
        let code = CASES[usize::from(self.0.index())].1;
        match CHANGE {
            0 => serializer.serialize_str(code),
            1 => serializer.serialize_unit_variant("Renamed", index, code),
            2 => serializer.serialize_unit_variant("Stem", index + 1, code),
            _ => self.0.serialize(serializer),
        }
    }
}

struct ReadableProbe<const CHANGE: u8>;

impl<'de, const CHANGE: u8> de::Visitor<'de> for ReadableProbe<CHANGE> {
    type Value = Probe<CHANGE>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a stem code")
    }

    fn visit_str<E: de::Error>(self, code: &str) -> Result<Self::Value, E> {
        Stem::deserialize(de::value::StrDeserializer::<E>::new(code)).map(Probe)
    }

    fn visit_string<E: de::Error>(self, code: String) -> Result<Self::Value, E> {
        if CHANGE == 3 && code == "unknown" {
            // Only the owned Value input is permissive; raw JSON text still rejects it.
            Ok(Probe(Stem::Jia))
        } else {
            self.visit_str(&code)
        }
    }

    fn visit_borrowed_str<E: de::Error>(self, code: &'de str) -> Result<Self::Value, E> {
        if CHANGE == 4 && code == "unknown" {
            // Only borrowed raw JSON text is permissive; Value input still rejects it.
            Ok(Probe(Stem::Jia))
        } else {
            self.visit_str(code)
        }
    }

    fn visit_enum<A: de::EnumAccess<'de>>(self, data: A) -> Result<Self::Value, A::Error> {
        use de::VariantAccess;
        let (stem, variant) = data.variant::<Stem>()?;
        variant.unit_variant()?;
        Ok(Probe(stem))
    }
}

impl<'de, const CHANGE: u8> Deserialize<'de> for Probe<CHANGE> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if CHANGE >= 3 && deserializer.is_human_readable() {
            deserializer.deserialize_any(ReadableProbe::<CHANGE>)
        } else {
            Stem::deserialize(deserializer).map(Self)
        }
    }
}

fn check<const CHANGE: u8>() {
    support::assert_enum_contract(
        "Stem",
        &CASES.map(|(stem, code)| (Probe::<CHANGE>(stem), code)),
        support::IdentifierProfile { bytes: true },
    );
}

#[test]
fn shared_contract_detects_json_invisible_model_mutations() {
    for stem in CASES.map(|case| case.0) {
        let expected = serde_json::to_value(stem).unwrap();
        assert_eq!(serde_json::to_value(Probe::<0>(stem)).unwrap(), expected);
        assert_eq!(serde_json::to_value(Probe::<1>(stem)).unwrap(), expected);
        assert_eq!(serde_json::to_value(Probe::<2>(stem)).unwrap(), expected);
    }
    for run in [check::<0>, check::<1>, check::<2>] {
        assert!(std::panic::catch_unwind(run).is_err());
    }
}

#[test]
fn shared_contract_detects_each_permissive_json_entry_independently() {
    for (stem, code) in CASES {
        assert_eq!(
            serde_json::from_value::<Probe<3>>(serde_json::json!(code)).unwrap(),
            Probe(stem)
        );
        assert_eq!(
            serde_json::from_value::<Probe<4>>(serde_json::json!(code)).unwrap(),
            Probe(stem)
        );
        let text = serde_json::to_string(code).unwrap();
        assert_eq!(
            serde_json::from_str::<Probe<3>>(&text).unwrap(),
            Probe(stem)
        );
        assert_eq!(
            serde_json::from_str::<Probe<4>>(&text).unwrap(),
            Probe(stem)
        );
    }
    assert_eq!(
        serde_json::from_value::<Probe<3>>(serde_json::json!("unknown")).unwrap(),
        Probe(Stem::Jia)
    );
    assert!(serde_json::from_str::<Probe<3>>("\"unknown\"").is_err());
    assert!(serde_json::from_value::<Probe<4>>(serde_json::json!("unknown")).is_err());
    assert_eq!(
        serde_json::from_str::<Probe<4>>("\"unknown\"").unwrap(),
        Probe(Stem::Jia)
    );
    for (run, expected) in [
        (check::<3> as fn(), "accepted JSON value \"unknown\""),
        (check::<4> as fn(), "accepted JSON text \"unknown\""),
    ] {
        let panic = std::panic::catch_unwind(run).unwrap_err();
        let message = panic.downcast_ref::<String>().unwrap();
        assert!(message.contains(expected), "{message}");
    }
}

#[test]
fn shared_contract_rejects_zero_samples() {
    assert!(
        std::panic::catch_unwind(|| {
            support::assert_enum_contract::<Stem, 0>(
                "Stem",
                &[],
                support::IdentifierProfile { bytes: true },
            );
        })
        .is_err()
    );
}
