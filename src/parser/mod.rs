pub mod value;
pub mod parsed_data;

use std::error::Error;
use std::io::Read;
use std::path::PathBuf;

use crate::cliargs::CliArgs;
use calamine::{open_workbook_auto, Reader};
use parsed_data::ParsedData;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use toml::Value as TomlValue;
use value::TypstValue;

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


pub fn parse_input(
	reader: &mut dyn Read,
	format: InputFormat,
	args: &CliArgs,
) -> Result<ParsedData, Box<dyn Error>> {
	Ok(match format {
		InputFormat::Csv => {
			let mut rdr = csv::ReaderBuilder::new().has_headers(!args.no_header).from_reader(reader);
			let header = match args.no_header {
				true => None,
				false => Some(rdr.headers()?.into_iter().map(|s| s.to_string()).collect::<Vec<_>>()),
			};
			let rows = rdr.records().collect::<Result<Vec<_>, _>>()? // CSV 解析失败 => Err
				.into_iter().map(|record| {
				record.iter().map(TypstValue::from) // 可能失败
					.collect::<Vec<_>>()   // 立即传播
			}).collect::<Vec<Vec<TypstValue>>>(); // 全部收集，传播失败
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
				let rows = range.rows().map(|row| row.iter().map(|cell| cell.into()).collect::<Vec<TypstValue>>()).collect();
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
            arr.into_iter().map(TypstValue::from).collect::<Vec<_>>(),
		),
		JsonValue::Object(map) => ParsedData::Map(
            map.into_iter().map(|(k, v)| Ok::<(String, TypstValue), Box<dyn Error>>((k, TypstValue::from(v)))).collect::<Result<Vec<_>, _>>()?,
		),
		other => ParsedData::List(vec![TypstValue::from(other)]),
	})
}

fn yaml_to_parsed_data(v: YamlValue) -> Result<ParsedData, Box<dyn Error>> {
	Ok(match v {
		YamlValue::Sequence(seq) => ParsedData::List(seq.into_iter().map(TypstValue::from).collect::<Vec<_>>()),
		YamlValue::Mapping(map) => ParsedData::Map(
			map.into_iter().map(|(k, v)| Ok::<(String, TypstValue), Box<dyn Error>>((k.as_str().ok_or("Failed As Str")?.to_string(), TypstValue::from(v)))).collect::<Result::<Vec<(String, TypstValue)>, _>>()?
		),
		other => ParsedData::List(vec![TypstValue::from(other)]),
	})
}

fn toml_to_parsed_data(v: TomlValue) -> Result<ParsedData, Box<dyn Error>> {
	Ok(match v {
		TomlValue::Array(arr) => ParsedData::List(
			arr.into_iter().map(TypstValue::from).collect::<Vec<_>>()
		),
		TomlValue::Table(tbl) => ParsedData::Map(
            tbl.into_iter().map(|(k, v)| Ok::<(String, TypstValue), Box<dyn Error>>((k, TypstValue::from(v)))).collect::<Result::<Vec<(String, TypstValue)>, _>>()?,
		),
		other => ParsedData::List(vec![TypstValue::from(other)]),
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
