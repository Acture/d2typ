use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant};
use serde::{Serialize, Serializer};
use std::fmt::Write;



#[derive(Debug, Default)]
pub struct PDSerializer {
	buffer: String,
}

macro_rules! serialize_simple_values {
	($($fn_name:ident, $v_type:ty)*) => {
		$(
		fn $fn_name(self, v: $v_type) -> Result<Self::Ok, Self::Error> {
			write!(self.buffer, "{}", v)
		}
		)*
	};
}

macro_rules! serialize_simple_str {
	($($fn_name:ident, $v_type:ty)*) => {
		$(
		fn $fn_name(self, v: $v_type) -> Result<Self::Ok, Self::Error> {
			write!(self.buffer, "{}", v.escape_default())
		}
		)*
	};
}

impl<'a> Serializer for &'a mut PDSerializer {
	type Ok = ();
	type Error = std::fmt::Error;
	type SerializeSeq = PDSeqSerializer;
	type SerializeTuple = PDTupleSerializer;
	type SerializeTupleStruct = PDTupleStructSerializer;
	type SerializeTupleVariant = PDTupleVariantSerializer;
	type SerializeMap = PDMapSerializer;
	type SerializeStruct = PDStructSerializer;
	type SerializeStructVariant = PDStructVariantSerializer;

	serialize_simple_values! {
		serialize_bool, bool
		serialize_i8, i8
		serialize_i16, i16
		serialize_i32, i32
		serialize_i64, i64
		serialize_i128, i128
		serialize_u8, u8
		serialize_u16, u16
		serialize_u32, u32
		serialize_u64, u64
		serialize_u128, u128
		serialize_f32, f32
		serialize_f64, f64
	}

	serialize_simple_str! {
		serialize_char, char
		serialize_str, &str
	}
	fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
		todo!()
	}

	fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
		todo!()
	}

	fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		todo!()
	}

	fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
		todo!()
	}

	fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
		todo!()
	}

	fn serialize_unit_variant(self, name: &'static str, variant_index: u32, variant: &'static str) -> Result<Self::Ok, Self::Error> {
		todo!()
	}

	fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		todo!()
	}

	fn serialize_newtype_variant<T>(self, name: &'static str, variant_index: u32, variant: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		todo!()
	}

	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
		todo!()
	}

	fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
		todo!()
	}

	fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
		todo!()
	}

	fn serialize_tuple_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeTupleVariant, Self::Error> {
		todo!()
	}

	fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
		todo!()
	}

	fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct, Self::Error> {
		todo!()
	}

	fn serialize_struct_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeStructVariant, Self::Error> {
		todo!()
	}
}


pub struct PDSeqSerializer;

impl SerializeSeq for PDSeqSerializer {
	type Ok = ();
	type Error = std::fmt::Error;

	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		todo!()
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
}

pub struct PDTupleSerializer;

impl SerializeTuple for PDTupleSerializer {
	type Ok = ();
	type Error = std::fmt::Error;

	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		todo!()
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
}

pub struct PDTupleStructSerializer;

impl SerializeTupleStruct for PDTupleStructSerializer {
	type Ok = ();
	type Error = std::fmt::Error;

	fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		todo!()
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
}

pub struct PDTupleVariantSerializer;

impl SerializeTupleVariant for PDTupleVariantSerializer {
	type Ok = ();
	type Error = std::fmt::Error;

	fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		todo!()
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
}

pub struct PDMapSerializer;

impl SerializeMap for PDMapSerializer {
	type Ok = ();
	type Error = std::fmt::Error;

	fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		todo!()
	}

	fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		todo!()
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
}

pub struct PDStructSerializer;

impl SerializeStruct for PDStructSerializer {
	type Ok = ();
	type Error = std::fmt::Error;

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		todo!()
	}

	fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
		todo!()
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
}

pub struct PDStructVariantSerializer;

impl SerializeStructVariant for PDStructVariantSerializer {
	type Ok = ();
	type Error = std::fmt::Error;

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		todo!()
	}

	fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
		todo!()
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
}
