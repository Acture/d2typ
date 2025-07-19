use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
pub(crate) mod ser;

use crate::parser::parsed_data;
use crate::parser::value::TypstValue;
use serde::ser::{SerializeMap, SerializeTuple};
use serde::{Serialize, Serializer};
use std::fmt;
use std::fmt::Display;
use tabled::builder::Builder as TabledBuilder;
use tabled::settings::Style;

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedData {
	Table((Option<Vec<String>>, Vec<Vec<TypstValue>>)),
	Map(Vec<(String, TypstValue)>),
	List(Vec<TypstValue>),
	Tuple(Vec<TypstValue>),
}

impl<T: ToString, V: Into<TypstValue>> From<(Option<Vec<T>>, Vec<Vec<V>>)> for ParsedData {
	fn from(value: (Option<Vec<T>>, Vec<Vec<V>>)) -> Self {
		let header: Option<Vec<String>> = match value.0 {
			Some(h) => Some(h.into_iter().map(|s| s.to_string()).collect()),
			None => None,
		};
		let data: Vec<Vec<TypstValue>> = value.1.into_iter()
			.map(|row| row.into_iter().map(|v| { v.into() }).collect())
			.collect();
		ParsedData::Table((header, data))
	}
}

impl<T: ToString, V: Into<TypstValue>> From<Vec<(T, V)>> for ParsedData {
	fn from(value: Vec<(T, V)>) -> Self {
		let value = value.into_iter()
			.map(|(k, v)| (k.to_string(), v.into()))
			.collect::<Vec<_>>();
		ParsedData::Map(value)
	}
}

impl From<Vec<TypstValue>> for ParsedData {
	fn from(value: Vec<TypstValue>) -> Self {
		ParsedData::List(value)
	}
}

impl Display for ParsedData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", parsed_data::ser::to_string(self)?)
	}
}


impl Serialize for ParsedData {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		Ok(match self {
			ParsedData::Table((header, rows)) => {
				match header {
					Some(h) => {
						if h.is_empty() {
							return Err(serde::ser::Error::custom("Header cannot be empty"));
						}
						let row_map = rows.iter()
							.map(|(row)| row.iter().zip(h.clone().iter()).map(|(cell, col)| (col.clone(), cell.clone())).collect::<BTreeMap<_, _>>())
							.collect::<Vec<_>>();
						serializer.collect_seq(row_map)?
					}
					_ => {
						serializer.collect_seq(rows)?
					}
				}
			}
			ParsedData::Map(map_data) => {
				serializer.collect_map(map_data.clone())?
			}
			ParsedData::List(list_data) => {
				serializer.collect_seq(list_data)?
			}
			ParsedData::Tuple(tuple_data) => {
				let mut seq = serializer.serialize_tuple(tuple_data.len())?;
				for item in tuple_data {
					seq.serialize_element(item)?;
				}
				seq.end()?
			}
		})
	}
}

impl ParsedData {
	pub fn to_table(&self) -> Result<String, fmt::Error> {
		let mut f = String::new();
		match self {
			ParsedData::Table((header, rows)) => {
				let mut builder = TabledBuilder::default();
				if let Some(h) = header {
					builder.push_record(h.clone())
				}

				for row in rows {
					let r: Vec<String> = row.iter().map(|v| v.to_string()).collect();
					builder.push_record(r);
				}

				let t = builder.build()
					.with(Style::rounded())
					.to_string();
				write!(f, "Table:\n{t}")?;
			}
			ParsedData::Map(map) => {
				write!(f, "Map: ")?;
				for (k, v) in map {
					writeln!(f, "{k}: {v}")?;
				}
			}
			ParsedData::List(list) => {
				write!(f, "List: ")?;
				let s_list = list.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ");
				writeln!(f, "[{s_list}]")?;
			}
			ParsedData::Tuple(tuple) => {
				write!(f, "Tuple: ")?;
				let s_tuple = tuple.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ");
				writeln!(f, "({s_tuple})")?;
			}
		}
		Ok(f)
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parsed_data_table_serialization() {
		let data = ParsedData::Table((Some(vec!["col1".to_string(), "col2".to_string(), "col3".to_string()]), vec![
			vec![TypstValue::Str("v1".to_string()), TypstValue::Str("v2".to_string()), TypstValue::Str("v3".to_string())],
			vec![TypstValue::Str("v4".to_string()), TypstValue::Str("v5".to_string()), TypstValue::Str("v6".to_string())],
		]));

		let serialized = ser::to_string(&data).unwrap();
		let expected = r#"[{ col1: v1, col2: v2, col3: v3 }, { col1: v4, col2: v5, col3: v6 }]"#;
		assert_eq!(serialized, expected);
		println!("Serialized ParsedData: {}", serialized);
	}

	#[test]
	fn test_parsed_data_map_serialization() {
		let data = ParsedData::Map(vec![
			("key1".to_string(), TypstValue::Str("value1".to_string())),
			("key2".to_string(), TypstValue::Str("value2".to_string())),
		]);
		let serialized = ser::to_string(&data).unwrap();
		let expected = r#"{ key1: value1, key2: value2 }"#;
		assert_eq!(serialized, expected);
		println!("Serialized ParsedData: {}", serialized);
	}

	#[test]
	fn test_parsed_data_list_serialization() {
		let data = ParsedData::List(vec![
			TypstValue::Str("item1".to_string()),
			TypstValue::Str("item2".to_string()),
		]);
		let serialized = ser::to_string(&data).unwrap();
		let expected = r#"[item1, item2]"#;
		assert_eq!(serialized, expected);
		println!("Serialized ParsedData: {}", serialized);
	}

	#[test]
	fn test_parsed_data_tuple_serialization() {
		let data = ParsedData::Tuple(vec![
			TypstValue::Str("item1".to_string()),
			TypstValue::Str("item2".to_string()),
		]);
		let serialized = ser::to_string(&data).unwrap();
		let expected = r#"(item1, item2)"#;
		assert_eq!(serialized, expected);
		println!("Serialized ParsedData: {}", serialized);
	}
}
