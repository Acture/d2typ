use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::io::Read;
use std::path::PathBuf;

use crate::cliargs::CliArgs;
use calamine::{open_workbook_auto, Data, Reader};
use derive_more::{Display, TryFrom};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use tabled::builder::Builder as TabledBuilder;
use tabled::settings::Style;
use toml::Value as TomlValue;

#[derive(Copy, Clone, Debug)]
#[derive(PartialEq)]
pub enum InputFormat {
	Csv,
	Json,
	Yaml,
	Toml,
	Xlsx,
}

pub fn detect_format(path: &Option<PathBuf>) -> Result<InputFormat, Box<dyn Error>> {
	if let Some(p) = path {
		if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
			return match ext.to_ascii_lowercase().as_str() {
				"csv" => Ok(InputFormat::Csv),
				"json" => Ok(InputFormat::Json),
				"yaml" | "yml" => Ok(InputFormat::Yaml),
				"toml" => Ok(InputFormat::Toml),
				"xls" | "xlsx" => Ok(InputFormat::Xlsx),
				_ => Err("Unknown file extension".into()),
			};
		}
	}
	Err("Cannot detect format from input".into())
}

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

impl TryFrom<&str> for TypstValue {
	type Error = Box<dyn Error>;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		if value.is_empty() {
			Ok(TypstValue::Null)
		} else if let Ok(b) = value.parse::<bool>() {
			Ok(TypstValue::Bool(b))
		} else if let Ok(i) = value.parse::<i64>() {
			Ok(TypstValue::Int(i))
		} else if let Ok(f) = value.parse::<f64>() {
			Ok(TypstValue::Float(f))
		} else {
			Ok(TypstValue::Str(value.to_string()))
		}
	}
}

impl TryFrom<JsonValue> for TypstValue {
	type Error = Box<dyn Error>;

	fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
		match value {
			JsonValue::Null => Ok(TypstValue::Null),
			JsonValue::Bool(b) => Ok(TypstValue::Bool(b)),
			JsonValue::Number(n) => {
				if let Some(i) = n.as_i64() {
					Ok(TypstValue::Int(i))
				} else if let Some(f) = n.as_f64() {
					Ok(TypstValue::Float(f))
				} else {
					Err("Invalid number type".into())
				}
			}
			JsonValue::String(s) => Ok(TypstValue::Str(s)),
			JsonValue::Array(arr) => {
				let values = arr.into_iter().map(TypstValue::try_from).collect::<Result<Vec<_>, _>>()?;
				Ok(TypstValue::Array(values))
			}
			JsonValue::Object(map) => {
				let values = map.into_iter()
					.map(|(k, v)| Ok::<(String, TypstValue), Self::Error>((k, TypstValue::try_from(v)?)))
					.collect::<Result<HashMap<_, _>, _>>()?;
				Ok(TypstValue::Map(values))
			}
		}
	}
}

impl TryFrom<YamlValue> for TypstValue {
	type Error = Box<dyn Error>;

	fn try_from(value: YamlValue) -> Result<Self, Self::Error> {
		match value {
			YamlValue::Null => Ok(TypstValue::Null),
			YamlValue::Bool(b) => Ok(TypstValue::Bool(b)),
			YamlValue::Number(n) => {
				if let Some(i) = n.as_i64() {
					Ok(TypstValue::Int(i))
				} else if let Some(f) = n.as_f64() {
					Ok(TypstValue::Float(f))
				} else {
					Err("Invalid number type".into())
				}
			}
			YamlValue::String(s) => Ok(TypstValue::Str(s)),
			YamlValue::Sequence(seq) => {
				let values = seq.into_iter().map(TypstValue::try_from).collect::<Result<Vec<_>, _>>()?;
				Ok(TypstValue::Array(values))
			}
			YamlValue::Mapping(map) => {
				let values = map.into_iter()
					.filter_map(|(k, v)| k.as_str().map(|ks| (ks.to_string(), TypstValue::try_from(v).unwrap())))
					.collect::<HashMap<_, _>>();
				Ok(TypstValue::Map(values))
			}
			_ => Err("Unsupported YAML value type".into()),
		}
	}
}

impl TryFrom<TomlValue> for TypstValue {
	type Error = Box<dyn Error>;

	fn try_from(value: TomlValue) -> Result<Self, Self::Error> {
		match value {
			TomlValue::String(s) => Ok(TypstValue::Str(s)),
			TomlValue::Integer(i) => Ok(TypstValue::Int(i)),
			TomlValue::Float(f) => Ok(TypstValue::Float(f)),
			TomlValue::Boolean(b) => Ok(TypstValue::Bool(b)),
			TomlValue::Array(arr) => {
				let values = arr.into_iter().map(TypstValue::try_from).collect::<Result<Vec<_>, _>>()?;
				Ok(TypstValue::Array(values))
			}
			TomlValue::Table(tbl) => {
				let values = tbl.into_iter()
					.map(|(k, v)| Ok::<(String, TypstValue), Self::Error>((k, TypstValue::try_from(v)?)))
					.collect::<Result<HashMap<_, _>, _>>()?;
				Ok(TypstValue::Map(values))
			}
			_ => Err("Unsupported TOML value type".into()),
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


#[derive(Debug, Clone, PartialEq)]
pub enum ParsedData {
	Table((Option<Vec<String>>, Vec<Vec<TypstValue>>)),
	Map(Vec<(String, TypstValue)>),
	List(Vec<TypstValue>),
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

pub fn parse_input(
	reader: &mut dyn Read,
	format: InputFormat,
	args: &CliArgs,
) -> Result<ParsedData, Box<dyn Error>> {
	Ok(match format {
		InputFormat::Csv => {
			let mut rdr = csv::ReaderBuilder::new()
				.has_headers(!args.no_header)
				.from_reader(reader);
			let header = match args.no_header {
				true => None,
				false => Some(rdr.headers()?.into_iter().map(|s| s.to_string()).collect::<Vec<_>>()),
			};
			let rows = rdr
				.records()
				.collect::<Result<Vec<_>, _>>()? // CSV 解析失败 => Err
				.into_iter()
				.map(|record| {
					record
						.iter()
						.map(|s| TypstValue::try_from(s)) // 可能失败
						.collect::<Result<Vec<_>, _>>()   // 立即传播
				})
				.collect::<Result<Vec<Vec<TypstValue>>, _>>()?; // 全部收集，传播失败
			ParsedData::Table((header, rows))
		}
		InputFormat::Json => {
			let v: JsonValue = serde_json::from_reader(reader)?;
			json_to_parsed_data(v)?
		}
		InputFormat::Yaml => {
			let v: YamlValue = serde_yaml::from_reader(reader)?;
			yaml_to_parsed_data(v)?
		}
		InputFormat::Toml => {
			let mut buffer = String::new();
			reader.read_to_string(&mut buffer)?;
			let v: TomlValue = toml::from_str(&buffer)?;
			toml_to_parsed_data(v)?
		}
		InputFormat::Xlsx => {
			let path = args.input.as_ref().ok_or("XLSX format requires input file")?;
			let mut workbook = open_workbook_auto(path)?;
			if let Ok(range) = workbook.worksheet_range(args.sheet.as_deref().unwrap_or("Sheet1")) {
				let header = match args.no_header {
					true => None,
					false => range.headers()
				};
				let rows = range
					.rows()
					.into_iter()
					.map(|row| row.iter().map(|cell| cell.into()).collect::<Vec<TypstValue>>())
					.collect();
				ParsedData::Table((header, rows))
			} else {
				return Err("Failed to read XLSX sheet".into());
			}
		}
	})
}

fn json_to_parsed_data(v: JsonValue) -> Result<ParsedData, Box<dyn Error>> {
	Ok(match v {
		JsonValue::Array(arr) => ParsedData::List(
            arr.into_iter().map(TypstValue::try_from).collect::<Result<Vec<_>, _>>()?,
		),
		JsonValue::Object(map) => ParsedData::Map(
            map.into_iter()
				.map(|(k, v)| Ok::<(String, TypstValue), Box<dyn Error>>((k, TypstValue::try_from(v)?)))
				.collect::<Result<Vec<_>, _>>()?,
		),
		other => ParsedData::List(vec![TypstValue::try_from(other)?]),
	})
}

fn yaml_to_parsed_data(v: YamlValue) -> Result<ParsedData, Box<dyn Error>> {
	Ok(match v {
		YamlValue::Sequence(seq) => ParsedData::List(seq.into_iter().map(TypstValue::try_from).collect::<Result<Vec<_>, _>>()?),
		YamlValue::Mapping(map) => ParsedData::Map(
			map.into_iter()
				.map(|(k, v)| Ok::<(String, TypstValue), Box<dyn Error>>((k.as_str().ok_or("Failed As Str")?.to_string(), TypstValue::try_from(v)?)))
				.collect::<Result::<Vec<(String, TypstValue)>, _>>()?
		),
		other => ParsedData::List(vec![TypstValue::try_from(other)?]),
	})
}

fn toml_to_parsed_data(v: TomlValue) -> Result<ParsedData, Box<dyn Error>> {
	Ok(match v {
		TomlValue::Array(arr) => ParsedData::List(
			arr.into_iter().map(TypstValue::try_from).collect::<Result<Vec<_>, _>>()?
		),
		TomlValue::Table(tbl) => ParsedData::Map(
            tbl.into_iter()
				.map(|(k, v)| Ok::<(String, TypstValue), Box<dyn Error>>((k, TypstValue::try_from(v)?)))
				.collect::<Result::<Vec<(String, TypstValue)>, _>>()?,
		),
		other => ParsedData::List(vec![TypstValue::try_from(other)?]),
	})
}


#[cfg(test)]
mod tests {
	use super::*;
use std::io::Cursor;

	#[test]
	fn test_detect_format() {
		assert_eq!(detect_format(&Some(PathBuf::from("data.csv"))).unwrap(), InputFormat::Csv);
		assert_eq!(detect_format(&Some(PathBuf::from("data.json"))).unwrap(), InputFormat::Json);
		assert_eq!(detect_format(&Some(PathBuf::from("data.yaml"))).unwrap(), InputFormat::Yaml);
		assert_eq!(detect_format(&Some(PathBuf::from("data.toml"))).unwrap(), InputFormat::Toml);
		assert_eq!(detect_format(&Some(PathBuf::from("data.xlsx"))).unwrap(), InputFormat::Xlsx);
		assert!(detect_format(&Some(PathBuf::from("data.txt"))).is_err());
	}

	#[test]
	fn test_parse_csv() {
		let data = "name,age\nAlice,30\nBob,25";
		let mut reader = Cursor::new(data);
		let result = parse_input(&mut reader, InputFormat::Csv, &CliArgs::default()).unwrap();
		println!("{result}");
	}
}