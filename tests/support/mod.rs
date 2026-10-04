//! Serde 模型观察器：独立捕获 unit variant 名称、数字索引和代码。
use serde::{Serialize, Serializer};

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
