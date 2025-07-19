use serde_json::Value as JsonValue;
use std::collections::HashMap;
use toml::Value as TomlValue;
use serde_yaml::Value as YamlValue;
use serde::{Deserialize, Serialize};
use derive_more::{Display, TryFrom};
use calamine::Data;

#[derive(Debug, Clone, Serialize, Deserialize, TryFrom, PartialEq, Display)]
pub enum TypstValue {
	Null,
	Bool(bool),
	Int(i64),
	Float(f64),
	Str(String),
	#[display("Tuple: {:?}", _0)]
	Tuple(Vec<TypstValue>),
	#[display("HashMao: {:?}", _0)]
	Map(HashMap<String, TypstValue>),
	#[display("Array: {:?}", _0)]
	Array(Vec<TypstValue>),
}

impl From<&str> for TypstValue {
	fn from(value: &str) -> Self {
		if value.is_empty() {
			TypstValue::Null
		} else if let Ok(b) = value.parse::<bool>() {
			TypstValue::Bool(b)
		} else if let Ok(i) = value.parse::<i64>() {
			TypstValue::Int(i)
		} else if let Ok(f) = value.parse::<f64>() {
			TypstValue::Float(f)
		} else {
			TypstValue::Str(value.to_string())
		}
	}
}

impl From<JsonValue> for TypstValue {
	fn from(value: JsonValue) -> Self {
		match value {
			JsonValue::Null => TypstValue::Null,
			JsonValue::Bool(b) => TypstValue::Bool(b),
			JsonValue::Number(n) => {
				if let Some(i) = n.as_i64() {
					TypstValue::Int(i)
				} else if let Some(f) = n.as_f64() {
					TypstValue::Float(f)
				} else {
					unimplemented!("Unsupported number type in JSON: {:?}", n);
				}
			}
			JsonValue::String(s) => TypstValue::Str(s),
			JsonValue::Array(arr) => {
				let values = arr.into_iter().map(TypstValue::from).collect::<Vec<_>>();
				TypstValue::Array(values)
			}
			JsonValue::Object(map) => {
				let values = map.into_iter()
					.map(|(k, v)| (k, TypstValue::from(v)))
					.collect::<HashMap<_, _>>();
				TypstValue::Map(values)
			}
		}
	}
}

impl From<YamlValue> for TypstValue {
	fn from(value: YamlValue) -> Self {
		match value {
			YamlValue::Null => TypstValue::Null,
			YamlValue::Bool(b) => TypstValue::Bool(b),
			YamlValue::Number(n) => {
				if let Some(i) = n.as_i64() {
					TypstValue::Int(i)
				} else if let Some(f) = n.as_f64() {
					TypstValue::Float(f)
				} else {
					unimplemented!("Unsupported number type in YAML: {:?}", n);
				}
			}
			YamlValue::String(s) => TypstValue::Str(s),
			YamlValue::Sequence(seq) => {
				let values = seq.into_iter().map(TypstValue::from).collect::<Vec<_>>();
				TypstValue::Array(values)
			}
			YamlValue::Mapping(map) => {
				let values = map.into_iter()
					.filter_map(|(k, v)| k.as_str().map(|ks| (ks.to_string(), TypstValue::from(v))))
					.collect::<HashMap<_, _>>();
				TypstValue::Map(values)
			}
			_ => unimplemented!("Unsupported YAML value type: {:?}", value),
		}
	}
}

impl From<TomlValue> for TypstValue {
	fn from(value: TomlValue) -> Self {
		match value {
			TomlValue::String(s) => TypstValue::Str(s),
			TomlValue::Integer(i) => TypstValue::Int(i),
			TomlValue::Float(f) => TypstValue::Float(f),
			TomlValue::Boolean(b) => TypstValue::Bool(b),
			TomlValue::Array(arr) => {
				let values = arr.into_iter().map(TypstValue::from).collect::<Vec<_>>();
				TypstValue::Array(values)
			}
			TomlValue::Table(tbl) => {
				let values = tbl.into_iter()
					.map(|(k, v)| (k, TypstValue::from(v)))
					.collect::<HashMap<_, _>>();
				TypstValue::Map(values)
			}
			_ => {
				unimplemented!("Unsupported TOML value type: {:?}", value);
			}
		}
	}
}

impl From<&Data> for TypstValue {
	fn from(data: &Data) -> Self {
		match data {
			Data::Empty => TypstValue::Null,
			Data::Bool(b) => TypstValue::Bool(*b),
			Data::Int(i) => TypstValue::Int(*i),
			Data::Float(f) => TypstValue::Float(*f),
			Data::String(s) => TypstValue::Str(s.clone()),
			Data::DateTime(dt) => TypstValue::Str(dt.to_string()),
			Data::Error(e) => TypstValue::Str(e.to_string()),
			Data::DateTimeIso(dt) => TypstValue::Str(dt.to_string()),
			Data::DurationIso(c) => TypstValue::Str(c.to_string()),
		}
	}
}