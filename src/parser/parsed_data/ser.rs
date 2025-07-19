use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant};
use serde::{Serialize, Serializer};
use std::fmt::Write;

pub fn to_string<T: Serialize + ?Sized>(value: &T) -> Result<String, std::fmt::Error> {
	let mut buffer = String::new();
	value.serialize(PDSerializer {
		buffer: &mut buffer,
	})?;

	Ok(buffer)
}

#[derive(Debug)]
pub struct PDSerializer<'a> {
	pub(crate) buffer: &'a mut String,
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
			write!(self.buffer, "{}", v)
		}
		)*
	};
}

impl<'a> Serializer for PDSerializer<'a> {
	type Ok = ();
	type Error = std::fmt::Error;
	type SerializeSeq = PDSeqSerializer<'a>;
	type SerializeTuple = PDTupleSerializer<'a>;
	type SerializeTupleStruct = PDTupleStructSerializer;
	type SerializeTupleVariant = PDTupleVariantSerializer;
	type SerializeMap = PDMapSerializer<'a>;
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
		write!(self.buffer, "{name}::{variant}")?;
		Ok(())
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
		value.serialize(
			self
		)
	}

	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
		write!(self.buffer, "[")?;
		Ok(PDSeqSerializer {
			buffer: self.buffer,
			is_first: true,
		})
	}

	fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
		write!(self.buffer, "(")?;
		Ok(PDTupleSerializer {
			buffer: self.buffer,
			is_first: true,
		})
	}

	fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
		todo!()
	}

	fn serialize_tuple_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeTupleVariant, Self::Error> {
		todo!()
	}

	fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
		write!(self.buffer, "{{ ")?;
		Ok(PDMapSerializer {
			buffer: self.buffer,
			is_first: true,
		})
	}

	fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct, Self::Error> {
		todo!()
	}

	fn serialize_struct_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeStructVariant, Self::Error> {
		todo!()
	}
}


#[derive(Debug)]
pub struct PDSeqSerializer<'a> {
	buffer: &'a mut String,
	is_first: bool,
}

impl<'a, > SerializeSeq for PDSeqSerializer<'a> {
	type Ok = ();
	type Error = std::fmt::Error;

	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		if !self.is_first {
			write!(self.buffer, ", ")?;
		} else {
			self.is_first = false;
		}
		write!(self.buffer, "{}", to_string(value)?)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		write!(self.buffer, "]")?;
		Ok(())
	}
}

#[derive(Debug)]
pub struct PDTupleSerializer<'a> {
	buffer: &'a mut String,
	is_first: bool,
}

impl SerializeTuple for PDTupleSerializer<'_> {
	type Ok = ();
	type Error = std::fmt::Error;

	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		if !self.is_first {
			write!(self.buffer, ", ")?;
		} else {
			self.is_first = false;
		}
		write!(self.buffer, "{}", to_string(value)?)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		write!(self.buffer, ")")
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

pub struct PDMapSerializer<'a> {
	buffer: &'a mut String,
	is_first: bool,
}

impl<'a> SerializeMap for PDMapSerializer<'_> {
	type Ok = ();
	type Error = std::fmt::Error;

	fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		if !self.is_first {
			write!(self.buffer, ", ")?;
		} else {
			self.is_first = false;
		}
		write!(self.buffer, "{}", to_string(key)?)
	}

	fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		write!(self.buffer, "{}", to_string(value)?)
	}

	fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
	where
		K: ?Sized + Serialize,
		V: ?Sized + Serialize,
	{
		self.serialize_key(key)?;
		write!(self.buffer, ": ")?;
		self.serialize_value(value)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		write!(self.buffer, " }}")?;
		Ok(())
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


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_serialize_simple_values() {
		let mut buffer = String::new();
		let serializer = PDSerializer { buffer: &mut buffer };
		assert!(serializer.serialize_i32(42).is_ok());
		assert_eq!(buffer, "42");
	}

	#[test]
	fn test_serialize_simple_str() {
		let mut buffer = String::new();
		let serializer = PDSerializer { buffer: &mut buffer };
		assert!(serializer.serialize_str("Hello").is_ok());
		assert_eq!(buffer, "Hello");
	}

	#[test]
	fn test_unit_variant() {
		let mut buffer = String::new();
		let serializer = PDSerializer { buffer: &mut buffer };
		assert!(serializer.serialize_unit_variant("Test", 0, "Variant").is_ok());
		assert_eq!(buffer, "Test::Variant");
	}
}
