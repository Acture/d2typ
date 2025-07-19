// src/main.rs
use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use clap::{Parser};
use cliargs::{CliArgs, FormatOpt};

mod parser;
mod render;
mod cliargs;

use parser::{detect_format, parse_input, InputFormat};
use render::render_to_typst;

fn main() -> Result<(), Box<dyn Error>> {
	let args = CliArgs::parse();
	println!("Parsed arguments: {:?}", args);

	// Input reader
	let (mut reader, name): (Box<dyn Read>, &str) = match &args.input {
		Some(path) => {
			let reader = BufReader::new(File::open(path)?);
			let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("data");
			(Box::new(reader), file_name)
		}
		None => { (Box::new(io::stdin()), "data") }
	};

	// Detect format
	let format = match args.format {
		FormatOpt::Auto => detect_format(&args.input)?,
		FormatOpt::Csv => InputFormat::Csv,
		FormatOpt::Json => InputFormat::Json,
		FormatOpt::Yaml => InputFormat::Yaml,
		FormatOpt::Toml => InputFormat::Toml,
		FormatOpt::Xlsx => InputFormat::Xlsx,
	};

	let data = parse_input(&mut reader, format, &args)?;

	// Output writer
	let mut writer: Box<dyn Write> = match &args.output {
		Some(path) => Box::new(BufWriter::new(File::create(path)?)),
		None => Box::new(io::stdout()),
	};

	render_to_typst(data, args.mode, name, &mut writer)?;
	Ok(())
}
