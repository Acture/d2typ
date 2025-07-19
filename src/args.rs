use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use crate::render::OutputMode;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Args {
	/// Input file (omit for stdin)
	pub input: Option<PathBuf>,

	/// Output file (omit for stdout)
	#[clap(short, long)]
	pub output: Option<PathBuf>,

	/// Force input format
	#[clap(short, long, default_value = "auto")]
	pub format: FormatOpt,

	/// Output mode: tuple, dict, map
	#[clap(short = 'a', long = "as", default_value = "OutputMode::Tuple", value_enum)]
	pub mode: OutputMode,

	/// For CSV input: treat as no header
	#[clap(long)]
	pub no_header: bool,

	/// For XLSX input: select sheet
	#[clap(long)]
	pub sheet: Option<String>,
}

#[derive(Copy, Clone, ValueEnum, PartialEq, Eq)]
pub enum FormatOpt {
	Auto,
	Csv,
	Json,
	Yaml,
	Toml,
	Xlsx,
}