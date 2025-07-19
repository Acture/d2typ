use crate::render::RenderMode;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser, Default)]
#[clap(author, version, about)]
#[command(name = "d2typ", about = "Convert structured data to Typst format")]
pub struct CliArgs {
	/// Input file (omit for stdin)
	pub input: Option<PathBuf>,

	/// Output file (omit for stdout)
	#[clap(short, long)]
	pub output: Option<PathBuf>,

	/// Force input format
	#[clap(short, long, default_value = "auto")]
	pub format: FormatOpt,

	/// For CSV input: treat as no header
	#[clap(long, default_value = "false")]
	pub no_header: bool,

	/// For XLSX input: select sheet
	#[clap(long)]
	pub sheet: Option<String>,
}

#[derive(Copy, Clone, ValueEnum, PartialEq, Eq, Debug, Default)]
pub enum FormatOpt {
	#[default]
	Auto,
	Csv,
	Json,
	Yaml,
	Toml,
	Xlsx,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_cli_args() {
		let args = vec![
			"--input", "data.csv", "--output", "output.typst",
			"--format", "csv", "--as", "dict"
		];
		let cli_args = CliArgs::parse_from(args);
		assert_eq!(cli_args.input.unwrap(), PathBuf::from("data.csv"));
		assert_eq!(cli_args.output.unwrap(), PathBuf::from("output.typst"));
		assert_eq!(cli_args.format, FormatOpt::Csv);
	}
}