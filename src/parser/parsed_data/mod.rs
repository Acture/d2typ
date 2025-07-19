mod ser;

use std::fmt::Display;
use tabled::builder::Builder as TabledBuilder;
use tabled::settings::Style;
use serde::{Serialize, Serializer};
use serde::ser::SerializeMap;
use crate::parser::parsed_data::ser::PDSerializer;
use crate::parser::value::TypstValue;

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedData {
	Table((Option<Vec<String>>, Vec<Vec<TypstValue>>)),
	Map(Vec<(String, TypstValue)>),
	List(Vec<TypstValue>),
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
				write!(f, "Table:\n{t}")
			}
			ParsedData::Map(map) => {
				write!(f, "Map: ")?;
				for (k, v) in map {
					writeln!(f, "{k}: {v}")?;
				}
				Ok(())
			}
			ParsedData::List(list) => {
				write!(f, "List: ")?;
				let s_list = list.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ");
				writeln!(f, "[{s_list}]")
			}
		}
	}
}



