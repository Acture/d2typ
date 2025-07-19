use crate::parser::parsed_data::ParsedData;
use clap::ValueEnum;
use std::error::Error;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum, Debug, Default)]
pub enum RenderMode {
	#[default]
	Tuple,
	Dict,
	Map,
}

pub fn render_to_typst(
	val: ParsedData,
	name: &str,
	out: &mut dyn IoWrite,
) -> Result<(), Box<dyn Error>> {
	if name.is_empty() {
		return Err("Variable name cannot be empty".into());
	}
	write!(out, "#let {name} = {val}")?;
	Ok(())
}


#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug, Default)]
	pub struct MockWriter {
		pub content: String,
	}
	impl IoWrite for MockWriter {
		fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
			self.content.push_str(std::str::from_utf8(buf).unwrap());
			Ok(buf.len())
		}

		fn flush(&mut self) -> std::io::Result<()> {
			Ok(())
		}
	}

	#[test]
	fn test_render_to_typst() {
		let mut writer = MockWriter::default();
		let header = Some(vec!["col1", "col2"]);
		let data = vec![vec!["row1col1", "row1col2"], vec!["row2col1", "row2col2"]];
		let pd = ParsedData::from((header, data));
		let result = render_to_typst(pd, "test_data", &mut writer);
		assert!(result.is_ok());
		assert_eq!(
			writer.content,
			"#let test_data = [{ col1: row1col1, col2: row1col2 }, { col1: row2col1, col2: row2col2 }]"
		);
	}
}