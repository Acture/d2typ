// src/parser/mod.rs
use std::error::Error;
use std::fmt::Display;
use std::io::Read;
use std::path::PathBuf;

use crate::cliargs::CliArgs;
use calamine::{open_workbook_auto, Data, Reader as _};
use csv::ReaderBuilder;
use derive_more::{Display, From , TryFrom};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize, Display, From, TryFrom, PartialEq)]
pub enum TypstValue {
	Null,
	Bool(bool),
	Int(i64),
	Float(f64),
	#[from(ignore)]
	Str(String),
	Array(ArrayVec),
	Dict(DictVec),
	Ident(String),
}

impl From<&str> for TypstValue {
	fn from(s: &str) -> Self {
		if s.is_empty() {
			TypstValue::Null
		} else if let Ok(i) = s.parse::<i64>() {
			TypstValue::Int(i)
		} else if let Ok(f) = s.parse::<f64>() {
			TypstValue::Float(f)
		} else if s.eq_ignore_ascii_case("true") {
			TypstValue::Bool(true)
		} else if s.eq_ignore_ascii_case("false") {
			TypstValue::Bool(false)
		} else {
			TypstValue::Str(s.to_string())
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, From, PartialEq)]
pub struct DictVec(Vec<(TypstValue, TypstValue)>);

impl Display for DictVec {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let entries: Vec<String> = self.0.iter()
			.map(|(k, v)| format!("{}: {}", k.to_string(), v.to_string()))
			.collect();
		write!(f, "{{{}}}", entries.join(", "))
	}
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArrayVec(Vec<TypstValue>);

impl Display for ArrayVec {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let entries: Vec<String> = self.0.iter().map(|v| v.to_string()).collect();
		write!(f, "[{}]", entries.join(", "))
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
			Data::DateTimeIso(d) => TypstValue::Str(d.clone()),
			Data::DurationIso(d) => TypstValue::Str(d.clone()),
		}
	}
}

impl From<Data> for TypstValue {
	fn from(data: Data) -> Self {
		<Self as From<&Data>>::from(&data)
	}
}

impl<T, const N: usize> From<[T; N]> for ArrayVec
where
	T: Into<TypstValue>,
{
	fn from(arr: [T; N]) -> Self {
		ArrayVec(arr.into_iter().map(Into::into).collect())
	}
}

pub fn parse_input(
	reader: &mut dyn Read,
	format: InputFormat,
	args: &CliArgs,
) -> Result<Vec<ArrayVec>, Box<dyn Error>> {
	Ok(match format {
		InputFormat::Csv => {
			let mut rdr = ReaderBuilder::new().has_headers(!args.no_header).from_reader(reader);
			let res = rdr
				.records()
				.filter_map(|r| r.ok())
				.map(|record| ArrayVec(record.iter().map(|c| TypstValue::from(c)).collect::<Vec<TypstValue>>()))
				.collect::<Vec<_>>();
			println!("Parsed CSV data: {:?}", res);
			res
		}
		InputFormat::Json => serde_json::from_reader(reader)?,
		InputFormat::Yaml => serde_yaml::from_reader(reader)?,
		InputFormat::Toml => {
			let mut buffer = String::new();
			reader.read_to_string(&mut buffer)?;
			toml::from_str::<toml::Value>(&buffer).map(|v| v.try_into().unwrap())?
		}
		InputFormat::Xlsx => {
			let path = args.input.as_ref().ok_or("XLSX format requires input file")?;
			let mut workbook = open_workbook_auto(path)?;
			if let Ok(range) = workbook.worksheet_range(args.sheet.as_deref().unwrap_or("Sheet1")) {
				range.rows()
					.into_iter()
					.map(|row|
						ArrayVec(row.into_iter().map(|c_d| c_d.into::<>()).collect::<Vec<_>>())
					)
					.collect::<Vec<ArrayVec>>()
			} else {
				Err("Failed to read XLSX sheet")?
			}
		}
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
		println!("{result:?}");
		assert_eq!(result.len(), 2);
	}
}