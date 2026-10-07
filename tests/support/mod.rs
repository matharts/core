//! 共同的 Serde 枚举契约检查；领域预期与冻结样本由调用者维护。
use serde::{Serialize, Serializer};
use serde_json::{Value, json};
use serde_test::{Configure, Token, assert_de_tokens, assert_tokens};

/// 字符串与数字始终检查；字节标识符由领域类型显式选择。
#[derive(Clone, Copy)]
pub(crate) struct IdentifierProfile {
    pub(crate) bytes: bool,
}

/// 执行完整的共同协议检查；案例顺序独立定义预期 variant index。
pub(crate) fn assert_enum_contract<T, const N: usize>(
    name: &'static str,
    cases: &[(T, &'static str); N],
    identifiers: IdentifierProfile,
) where
    T: Serialize + serde::de::DeserializeOwned + Copy + PartialEq + core::fmt::Debug,
{
    assert!(N > 0, "{name}: enum contract must execute nonempty cases");
    for (index, &(value, code)) in cases.iter().enumerate() {
        let index = u32::try_from(index).unwrap();
        assert_eq!(value.serialize(VariantModel).unwrap(), (name, index, code));
        assert_eq!(serde_json::to_value(value).unwrap(), json!(code));
        assert_eq!(serde_json::from_value::<T>(json!(code)).unwrap(), value);
        assert_eq!(
            serde_json::from_str::<T>(&serde_json::to_string(code).unwrap()).unwrap(),
            value
        );
        let tokens = [Token::UnitVariant {
            name,
            variant: code,
        }];
        assert_tokens(&value.compact(), &tokens);
        assert_tokens(&value.readable(), &tokens);
        assert_de_tokens(&value.readable(), &[Token::Str(code)]);
        for identifier in [
            Token::U32(index),
            Token::U64(u64::from(index)),
            Token::Str(code),
            Token::BorrowedStr(code),
        ] {
            assert_de_tokens(
                &value.compact(),
                &[Token::Enum { name }, identifier, Token::Unit],
            );
        }
        if identifiers.bytes {
            for identifier in [
                Token::Bytes(code.as_bytes()),
                Token::BorrowedBytes(code.as_bytes()),
            ] {
                assert_de_tokens(
                    &value.compact(),
                    &[Token::Enum { name }, identifier, Token::Unit],
                );
            }
        }
        for invalid in [
            json!({code: null}),
            json!({code: 1}),
            json!([code]),
            json!(format!(" {code}")),
            json!(format!("{code} ")),
            json!(format!("{code}\n")),
        ] {
            assert_json_rejected::<T>(&invalid);
        }
        assert!(
            serde_json::from_str::<T>(&format!(r#"{{"{code}":null,"{code}":null}}"#)).is_err(),
            "{name}: duplicate-key object must be rejected"
        );
    }
    for invalid in [
        json!(null),
        json!(true),
        json!(false),
        json!(0),
        json!(255),
        json!(-1),
        json!(1.5),
        json!([]),
        json!({}),
        json!(""),
        json!("unknown"),
    ] {
        assert_json_rejected::<T>(&invalid);
    }
}

/// 每个领域负例同时经过 Value 和原始 JSON 文本入口。
pub(crate) fn assert_json_rejected<T: serde::de::DeserializeOwned>(value: &Value) {
    assert!(
        serde_json::from_value::<T>(value.clone()).is_err(),
        "{} accepted JSON value {value}",
        core::any::type_name::<T>()
    );
    assert!(
        serde_json::from_str::<T>(&value.to_string()).is_err(),
        "{} accepted JSON text {value}",
        core::any::type_name::<T>()
    );
}

/// 独立捕获名称、索引和代码；`serde_test` 的 unit-variant token 不核对数字索引。
pub(crate) struct VariantModel;
pub(crate) type Model = (&'static str, u32, &'static str);
pub(crate) type Error = serde::de::value::Error;
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
