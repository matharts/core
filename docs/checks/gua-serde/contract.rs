//! Isolated contract experiment. These are not matharts-core production types.

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_test::{
    Compact, Configure, Readable, Token, assert_de_tokens, assert_de_tokens_error,
    assert_ser_tokens,
};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename = "Trigram")]
enum Trigram {
    Kun,
    Zhen,
    Kan,
    Dui,
    Gen,
    Li,
    Xun,
    Qian,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename = "TrigramPosition")]
enum TrigramPosition {
    First,
    Second,
    Third,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename = "HexagramPosition")]
enum HexagramPosition {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
}

// Text inputs must be strings: derived externally tagged enums also accept
// objects such as {"Qian": null}, which are outside the text contract.
macro_rules! code_deserialize {
    ($ty:ident, $name:literal, $($variant:ident),+) => {
        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                if d.is_human_readable() {
                    struct CodeVisitor;
                    impl<'de> Visitor<'de> for CodeVisitor {
                        type Value = $ty;
                        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                            f.write_str(concat!($name, " code string"))
                        }
                        fn visit_str<E: de::Error>(self, v: &str) -> Result<$ty, E> {
                            match v {
                                $(stringify!($variant) => Ok($ty::$variant),)+
                                _ => Err(E::unknown_variant(v, &[$(stringify!($variant)),+])),
                            }
                        }
                    }
                    d.deserialize_str(CodeVisitor)
                } else {
                    #[derive(Deserialize)]
                    #[serde(rename = $name)]
                    enum Wire { $($variant),+ }
                    Ok(match Wire::deserialize(d)? { $(Wire::$variant => Self::$variant),+ })
                }
            }
        }
    };
}
code_deserialize!(Trigram, "Trigram", Kun, Zhen, Kan, Dui, Gen, Li, Xun, Qian);
code_deserialize!(TrigramPosition, "TrigramPosition", First, Second, Third);
code_deserialize!(
    HexagramPosition,
    "HexagramPosition",
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth
);

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename = "Hexagram")]
struct Hexagram {
    lower: Trigram,
    upper: Trigram,
}

impl<'de> Deserialize<'de> for Hexagram {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        const FIELDS: &[&str] = &["lower", "upper"];
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Lower,
            Upper,
        }
        struct ShapeVisitor {
            human_readable: bool,
        }
        impl<'de> Visitor<'de> for ShapeVisitor {
            type Value = Hexagram;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("Hexagram fields lower and upper")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Hexagram, A::Error> {
                if self.human_readable {
                    return Err(de::Error::custom(
                        "human-readable Hexagram requires an object",
                    ));
                }
                let lower = a
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let upper = a
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                if a.next_element::<Trigram>()?.is_some() {
                    return Err(de::Error::invalid_length(3, &self));
                }
                Ok(Hexagram { lower, upper })
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Hexagram, A::Error> {
                let (mut lower, mut upper) = (None, None);
                while let Some(field) = a.next_key()? {
                    match field {
                        Field::Lower => {
                            if lower.is_some() {
                                return Err(de::Error::duplicate_field("lower"));
                            }
                            lower = Some(a.next_value()?);
                        }
                        Field::Upper => {
                            if upper.is_some() {
                                return Err(de::Error::duplicate_field("upper"));
                            }
                            upper = Some(a.next_value()?);
                        }
                    }
                }
                Ok(Hexagram {
                    lower: lower.ok_or_else(|| de::Error::missing_field("lower"))?,
                    upper: upper.ok_or_else(|| de::Error::missing_field("upper"))?,
                })
            }
        }
        let human_readable = d.is_human_readable();
        d.deserialize_struct("Hexagram", FIELDS, ShapeVisitor { human_readable })
    }
}

fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("../../fixtures/gua-design-v1.json")).unwrap()
}

#[test]
fn json_examples_and_rejections() {
    let f = fixture();
    let positive = f["hexagram_cases"].as_array().unwrap();
    let negative = f["invalid_hexagram_json"].as_array().unwrap();
    let duplicate = f["invalid_hexagram_json_text"].as_array().unwrap();
    assert_eq!(
        (positive.len(), negative.len(), duplicate.len()),
        (6, 16, 2)
    );
    for row in positive {
        let value = row["wire"].clone();
        let h: Hexagram = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(h).unwrap(), value);
        assert_eq!(
            serde_json::from_str::<Hexagram>(&value.to_string()).unwrap(),
            h
        );
    }
    for value in negative {
        assert!(
            serde_json::from_value::<Hexagram>(value.clone()).is_err(),
            "{value}"
        );
        assert!(
            serde_json::from_str::<Hexagram>(&value.to_string()).is_err(),
            "{value}"
        );
    }
    for text in duplicate {
        assert!(serde_json::from_str::<Hexagram>(text.as_str().unwrap()).is_err());
    }
    assert_eq!(
        serde_json::from_str::<Hexagram>(r#"{"upper":"Kun","lower":"Qian"}"#).unwrap(),
        tai()
    );
    // Regression control: the added two-element negative defeats the old derive strategy.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Derived {
        #[serde(rename = "lower")]
        _lower: Trigram,
        #[serde(rename = "upper")]
        _upper: Trigram,
    }
    assert!(serde_json::from_str::<Derived>(r#"["Qian","Kun"]"#).is_ok());
}

fn tai() -> Hexagram {
    Hexagram {
        lower: Trigram::Qian,
        upper: Trigram::Kun,
    }
}

fn struct_tokens() -> [Token; 6] {
    [
        Token::Struct {
            name: "Hexagram",
            len: 2,
        },
        Token::Str("lower"),
        Token::UnitVariant {
            name: "Trigram",
            variant: "Qian",
        },
        Token::Str("upper"),
        Token::UnitVariant {
            name: "Trigram",
            variant: "Kun",
        },
        Token::StructEnd,
    ]
}

#[test]
fn struct_model_and_input_profiles() {
    let tokens = struct_tokens();
    assert_ser_tokens(&tai().readable(), &tokens);
    assert_ser_tokens(&tai().compact(), &tokens);
    let mut readable_tokens = tokens;
    readable_tokens[2] = Token::Str("Qian");
    readable_tokens[4] = Token::Str("Kun");
    assert_de_tokens(&tai().readable(), &readable_tokens);
    assert_de_tokens(&tai().compact(), &tokens);
    assert_de_tokens(
        &tai().compact(),
        &[
            Token::Seq { len: Some(2) },
            Token::UnitVariant {
                name: "Trigram",
                variant: "Qian",
            },
            Token::UnitVariant {
                name: "Trigram",
                variant: "Kun",
            },
            Token::SeqEnd,
        ],
    );
    assert_de_tokens_error::<Readable<Hexagram>>(
        &[Token::Seq { len: Some(2) }],
        "human-readable Hexagram requires an object",
    );
    assert_de_tokens_error::<Compact<Hexagram>>(
        &[Token::Seq { len: Some(0) }, Token::SeqEnd],
        "invalid length 0, expected Hexagram fields lower and upper",
    );
    assert_de_tokens_error::<Compact<Hexagram>>(
        &[
            Token::Seq { len: Some(1) },
            Token::UnitVariant {
                name: "Trigram",
                variant: "Qian",
            },
            Token::SeqEnd,
        ],
        "invalid length 1, expected Hexagram fields lower and upper",
    );
    assert_de_tokens_error::<Compact<Hexagram>>(
        &[
            Token::Seq { len: Some(3) },
            Token::UnitVariant {
                name: "Trigram",
                variant: "Qian",
            },
            Token::UnitVariant {
                name: "Trigram",
                variant: "Kun",
            },
            Token::UnitVariant {
                name: "Trigram",
                variant: "Kan",
            },
            Token::SeqEnd,
        ],
        "invalid length 3, expected Hexagram fields lower and upper",
    );
    for readable in [true, false] {
        let missing = [Token::Map { len: Some(0) }, Token::MapEnd];
        let mut duplicate = [
            Token::Map { len: None },
            Token::Str("lower"),
            Token::Str("Qian"),
            Token::Str("lower"),
        ];
        let unknown = [Token::Map { len: None }, Token::Str("moving_line")];
        if readable {
            assert_de_tokens_error::<Readable<Hexagram>>(&missing, "missing field `lower`");
            assert_de_tokens_error::<Readable<Hexagram>>(&duplicate, "duplicate field `lower`");
            assert_de_tokens_error::<Readable<Hexagram>>(
                &unknown,
                "unknown field `moving_line`, expected `lower` or `upper`",
            );
        } else {
            duplicate[2] = Token::UnitVariant {
                name: "Trigram",
                variant: "Qian",
            };
            assert_de_tokens_error::<Compact<Hexagram>>(&missing, "missing field `lower`");
            assert_de_tokens_error::<Compact<Hexagram>>(&duplicate, "duplicate field `lower`");
            assert_de_tokens_error::<Compact<Hexagram>>(
                &unknown,
                "unknown field `moving_line`, expected `lower` or `upper`",
            );
        }
    }
}

// serde_test 1.0.177 deliberately ignores variant_index in serialize_unit_variant.
// This tiny serializer observes the exact name/index/code triple independently.
struct VariantModel;
type Model = (&'static str, u32, &'static str);
type Error = serde::de::value::Error;
macro_rules! reject_scalar {
    ($($method:ident($ty:ty)),* $(,)?) => {$(
        fn $method(self, _: $ty) -> Result<Model, Error> { Err(serde::ser::Error::custom("not unit_variant")) }
    )*};
}
impl Serializer for VariantModel {
    type Ok = Model;
    type Error = Error;
    type SerializeSeq = serde::ser::Impossible<Model, Error>;
    type SerializeTuple = serde::ser::Impossible<Model, Error>;
    type SerializeTupleStruct = serde::ser::Impossible<Model, Error>;
    type SerializeTupleVariant = serde::ser::Impossible<Model, Error>;
    type SerializeMap = serde::ser::Impossible<Model, Error>;
    type SerializeStruct = serde::ser::Impossible<Model, Error>;
    type SerializeStructVariant = serde::ser::Impossible<Model, Error>;
    reject_scalar!(
        serialize_bool(bool),
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_f32(f32),
        serialize_f64(f64),
        serialize_char(char),
        serialize_str(&str),
        serialize_bytes(&[u8])
    );
    fn serialize_unit_variant(
        self,
        n: &'static str,
        i: u32,
        v: &'static str,
    ) -> Result<Model, Error> {
        Ok((n, i, v))
    }
    fn serialize_none(self) -> Result<Model, Error> {
        self.serialize_unit()
    }
    fn serialize_unit(self) -> Result<Model, Error> {
        Err(serde::ser::Error::custom("not unit_variant"))
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<Model, Error> {
        self.serialize_unit()
    }
    fn serialize_some<T: ?Sized + Serialize>(self, _: &T) -> Result<Model, Error> {
        self.serialize_unit()
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: &T,
    ) -> Result<Model, Error> {
        self.serialize_unit()
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<Model, Error> {
        self.serialize_unit()
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        Err(serde::ser::Error::custom("not unit_variant"))
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Error> {
        self.serialize_seq(None)
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        self.serialize_seq(None)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        self.serialize_seq(None)
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Error> {
        self.serialize_seq(None)
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeStruct, Error> {
        self.serialize_seq(None)
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        self.serialize_seq(None)
    }
}

fn check_variants<T: Serialize + serde::de::DeserializeOwned + PartialEq + fmt::Debug + Copy>(
    values: &[T],
    name: &'static str,
) {
    let f = fixture();
    let model = &f["serde_model"][name];
    assert_eq!(model["kind"], "unit_variant");
    assert_eq!(model["name"], name);
    let codes = model["variants"].as_array().unwrap();
    assert_eq!(values.len(), codes.len());
    for (index, (&value, code)) in values.iter().zip(codes).enumerate() {
        let triple = value.serialize(VariantModel).unwrap();
        assert_eq!(triple, (name, index as u32, code.as_str().unwrap()));
        assert_eq!(serde_json::to_value(value).unwrap(), *code);
        assert_eq!(serde_json::from_value::<T>(code.clone()).unwrap(), value);
        let object = serde_json::json!({code.as_str().unwrap(): null});
        assert!(serde_json::from_value::<T>(object.clone()).is_err());
        assert!(serde_json::from_str::<T>(&object.to_string()).is_err());
        assert_de_tokens(
            &value.compact(),
            &[Token::Enum { name }, Token::U32(index as u32), Token::Unit],
        );
    }
    for value in [
        serde_json::json!(0),
        serde_json::json!("unknown"),
        serde_json::json!({"First":0}),
    ] {
        assert!(serde_json::from_value::<T>(value).is_err());
    }
}

#[test]
fn all_seventeen_variant_models() {
    check_variants(
        &[
            Trigram::Kun,
            Trigram::Zhen,
            Trigram::Kan,
            Trigram::Dui,
            Trigram::Gen,
            Trigram::Li,
            Trigram::Xun,
            Trigram::Qian,
        ],
        "Trigram",
    );
    check_variants(
        &[
            TrigramPosition::First,
            TrigramPosition::Second,
            TrigramPosition::Third,
        ],
        "TrigramPosition",
    );
    check_variants(
        &[
            HexagramPosition::First,
            HexagramPosition::Second,
            HexagramPosition::Third,
            HexagramPosition::Fourth,
            HexagramPosition::Fifth,
            HexagramPosition::Sixth,
        ],
        "HexagramPosition",
    );
    assert_de_tokens_error::<Compact<Trigram>>(
        &[Token::Enum { name: "Trigram" }, Token::U32(8)],
        "invalid value: integer `8`, expected variant index 0 <= i < 8",
    );
    assert_de_tokens_error::<Compact<Trigram>>(
        &[
            Token::Enum { name: "Trigram" },
            Token::Str("Qian"),
            Token::U8(1),
        ],
        "invalid type: integer `1`, expected unit",
    );
    let m = &fixture()["serde_model"]["Hexagram"];
    assert_eq!(
        m,
        &serde_json::json!({"kind":"struct","name":"Hexagram","length":2,"fields":["lower","upper"]})
    );
}

#[test]
fn model_checks_detect_json_invisible_mutations() {
    struct Mutation(u8);
    impl Serialize for Mutation {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            match self.0 {
                0 => s.serialize_str("Qian"),
                1 => s.serialize_unit_variant("Renamed", 7, "Qian"),
                _ => s.serialize_unit_variant("Trigram", 0, "Qian"),
            }
        }
    }
    for change in 0..3 {
        let m = Mutation(change);
        assert_eq!(
            serde_json::to_string(&m).unwrap(),
            serde_json::to_string(&Trigram::Qian).unwrap()
        );
        assert_ne!(m.serialize(VariantModel).ok(), Some(("Trigram", 7, "Qian")));
    }
    // Same model assertion as above, with deliberately altered expectations.
    for change in 0..3 {
        let mut tokens = struct_tokens();
        match change {
            0 => {
                tokens[0] = Token::Map { len: Some(2) };
                tokens[5] = Token::MapEnd;
            }
            1 => {
                tokens[0] = Token::Struct {
                    name: "Renamed",
                    len: 2,
                }
            }
            _ => {
                tokens.swap(1, 3);
                tokens.swap(2, 4);
            }
        }
        assert!(
            std::panic::catch_unwind(|| assert_ser_tokens(&tai().readable(), &tokens)).is_err()
        );
    }
}
