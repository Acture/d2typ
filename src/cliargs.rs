use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

use docpack::{ArtifactKind, BackendKind, SourceFormat};

#[derive(Debug, Parser)]
#[clap(author, version, about, propagate_version = true)]
#[command(
    name = "docpack",
    about = "Package structured data into document-ready snapshot modules"
)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Build(BuildArgs),
    Emit(EmitArgs),
    Inspect(InspectArgs),
    Init(InitArgs),
}

#[derive(Debug, Parser)]
pub struct BuildArgs {
    pub manifest_path: Option<PathBuf>,
}

#[derive(Debug, Parser)]
pub struct EmitArgs {
    pub input: String,

    #[arg(short, long)]
    pub output: Option<PathBuf>,

    #[arg(short, long)]
    pub format: Option<SourceFormat>,

    #[arg(long)]
    pub backend: Option<BackendKind>,

    #[arg(long)]
    pub artifact: Option<ArtifactKind>,

    #[arg(long)]
    pub style: Option<String>,

    #[arg(long)]
    pub root_name: Option<String>,

    #[arg(long, default_value_t = false)]
    pub no_header: bool,

    #[arg(long)]
    pub sheet: Option<String>,
}

#[derive(Debug, Parser)]
pub struct InspectArgs {
    pub input: String,

    #[arg(long = "as")]
    pub as_target: Option<InspectTarget>,

    #[arg(short, long)]
    pub output: Option<PathBuf>,

    #[arg(short, long)]
    pub format: Option<SourceFormat>,

    #[arg(long)]
    pub backend: Option<BackendKind>,

    #[arg(long)]
    pub artifact: Option<ArtifactKind>,

    #[arg(long)]
    pub style: Option<String>,

    #[arg(long)]
    pub root_name: Option<String>,

    #[arg(long, default_value_t = false)]
    pub no_header: bool,

    #[arg(long)]
    pub sheet: Option<String>,
}

#[derive(Debug, Parser)]
pub struct InitArgs {
    pub path: Option<PathBuf>,

    #[arg(long, default_value_t = false)]
    pub force: bool,
}

#[derive(Copy, Clone, ValueEnum, PartialEq, Eq, Debug)]
pub enum InspectTarget {
    Source,
    Manifest,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_emit_command() {
        let cli_args = CliArgs::parse_from([
            "docpack",
            "emit",
            "data.csv",
            "--output",
            "output.typ",
            "--format",
            "csv",
        ]);
        match cli_args.command {
            Commands::Emit(args) => {
                assert_eq!(args.input, "data.csv");
                assert_eq!(args.output.unwrap(), PathBuf::from("output.typ"));
                assert_eq!(args.format, Some(SourceFormat::Csv));
            }
            _ => panic!("expected emit command"),
        }
    }
}
