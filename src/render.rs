use crate::parser::ArrayVec;
use clap::ValueEnum;
use indenter::indented;
use std::error::Error;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum OutputMode {
	Tuple,
	Dict,
	Map,
}

pub fn render_to_typst(
	val: Vec<ArrayVec>,
	mode: OutputMode,
	name: &str,
	out: &mut dyn IoWrite,
) -> Result<(), Box<dyn Error>> {
	if name.is_empty() {
		return Err("Variable name cannot be empty".into());
	}
	write!(out, "#let {name} = ")?;

	let (begin_char, end_char) = match mode {
		OutputMode::Tuple | OutputMode::Map => {
			("(", ")")
		}
		OutputMode::Dict => {
			("[", "]")
		}
	};

	let mut buffer = String::new();
	writeln!(buffer, "{begin_char}")?;
	{
		let mut indented_buffer = indented(&mut buffer).ind(1).with_str("\t");

		for row in val {
			indented_buffer.write_fmt(format_args!("{row}\n"))?;
		}
	}
	writeln!(buffer, "{end_char}")?;


	Ok(write!(out, "{buffer}")?)
}


#[cfg(test)]
mod tests {
	use super::*;
use crate::parser::ArrayVec;

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
	fn renders_tuple_mode_correctly() {
		let val = vec![ArrayVec::from([1, 2]), ArrayVec::from([3, 4])];
		let mut output = MockWriter::default();

		render_to_typst(val, OutputMode::Tuple, "my_var", &mut output).unwrap();

		assert_eq!(
			output.content,
			"#let my_var = (\n\t[1, 2]\n\t[3, 4]\n)\n"
		);
	}

	#[test]
	fn renders_dict_mode_correctly() {
		let val = vec![ArrayVec::from([1, 2]), ArrayVec::from([3, 4])];
		let mut output = MockWriter::default();

		render_to_typst(val, OutputMode::Dict, "my_var", &mut output).unwrap();

		assert_eq!(
			output.content,
			"#let my_var = [\n\t[1, 2]\n\t[3, 4]\n]\n"
		);
	}

	#[test]
	fn renders_empty_vec_correctly() {
		let val: Vec<ArrayVec> = vec![];
		let mut output = MockWriter::default();

		render_to_typst(val, OutputMode::Map, "empty_var", &mut output).unwrap();

		assert_eq!(output.content, "#let empty_var = (\n)\n");
	}

	#[test]
	fn handles_invalid_name_gracefully() {
		let val = vec![ArrayVec::from([1, 2])];
		let mut output = MockWriter::default();

		let result = render_to_typst(val, OutputMode::Tuple, "", &mut output);
		assert!(result.is_err());
	}
}