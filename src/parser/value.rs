use serde_json::Value as JsonValue;
use std::collections::BTreeMap;
use std::fmt::Display;
use toml::Value as TomlValue;
use serde_yaml::Value as YamlValue;
use serde::{Deserialize, Serialize};
use derive_more::TryFrom;
use calamine::Data;

#[derive(Debug, Clone, Serialize, Deserialize, TryFrom, PartialEq)]
pub enum TypstValue {
	Null,
	Bool(bool),
	Int(i64),
	Float(f64),
	Str(String),
	Tuple(Vec<TypstValue>),
	Map(BTreeMap<String, TypstValue>),
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
					.collect::<BTreeMap<_, _>>();
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
					.collect::<BTreeMap<_, _>>();
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
					.collect::<BTreeMap<_, _>>();
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

impl Display for TypstValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			TypstValue::Null => write!(f, "null"),
			TypstValue::Bool(b) => write!(f, "{}", b),
			TypstValue::Int(i) => write!(f, "{}", i),
			TypstValue::Float(fl) => write!(f, "{}", fl),
			TypstValue::Str(s) => write!(f, "{}", s),
			TypstValue::Tuple(t) => write!(f, "({})", t.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")),
			TypstValue::Map(m) => write!(f, "{{{}}}", m.iter().map(|(k, v)| format!("{}: {}", k, v)).collect::<Vec<_>>().join(", ")),
			TypstValue::Array(a) => write!(f, "[{}]", a.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")),
		}
	}
}


mod tests {
	use super::*;

	#[test]
	fn test_typst_value_from_str() {
		assert_eq!(TypstValue::from("true"), TypstValue::Bool(true));
		assert_eq!(TypstValue::from("123"), TypstValue::Int(123));
		assert_eq!(TypstValue::from("45.67"), TypstValue::Float(45.67));
		assert_eq!(TypstValue::from("hello"), TypstValue::Str("hello".to_string()));
		assert_eq!(TypstValue::from(""), TypstValue::Null);
	}


	#[test]
	fn test_display_typst_value() {
		let value = TypstValue::Map(BTreeMap::from([
			("key1".to_string(), TypstValue::Int(42)),
			("key2".to_string(), TypstValue::Str("value".to_string())),
			("key3".to_string(), TypstValue::Array(vec![TypstValue::Bool(true), TypstValue::Float(3.11)])),
		]));
		assert_eq!(value.to_string(), "{key1: 42, key2: value, key3: [true, 3.11]}");
	}
}