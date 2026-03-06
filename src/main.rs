mod cliargs;
use clap::Parser;
use std::fmt::Write as _;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cliargs::{BuildArgs, CliArgs, Commands, EmitArgs, InitArgs, InspectArgs, InspectTarget};
use docpack::{
    DocpackError, Origin, SourceFormat, SourceSpec, detect_format, manifest, parse_source,
    render_document,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), DocpackError> {
    let args = CliArgs::parse();
    match args.command {
        Commands::Build(args) => run_build(args),
        Commands::Emit(args) => run_emit(args),
        Commands::Inspect(args) => run_inspect(args),
        Commands::Init(args) => run_init(args),
    }
}

fn run_build(args: BuildArgs) -> Result<(), DocpackError> {
    manifest::build(args.manifest_path.as_deref())?;
    Ok(())
}

fn run_emit(args: EmitArgs) -> Result<(), DocpackError> {
    let input_path = input_path(&args.input);
    let spec = load_source_spec(&args.input, args.format, args.no_header, args.sheet.clone())?;
    let doc = parse_source(&spec)?;
    let request = manifest::resolve_source_request(
        &doc,
        manifest::SourceRequestOptions {
            input_path,
            output_path: args.output.as_deref(),
            backend: args.backend,
            artifact: args.artifact,
            style: args.style.as_deref(),
            root_name: args.root_name.as_deref(),
            require_explicit_backend_without_output: true,
        },
    )?;
    let rendered = render_document(&doc, &request)?;

    if let Some(path) = args.output {
        write_file(&path, &rendered.body)?;
    } else {
        let mut stdout = io::stdout();
        stdout
            .write_all(rendered.body.as_bytes())
            .map_err(|source| DocpackError::Io {
                origin: Origin::Stdin,
                source,
            })?;
    }
    Ok(())
}

fn run_inspect(args: InspectArgs) -> Result<(), DocpackError> {
    let target = match args.as_target {
        Some(target) => target,
        None => infer_inspect_target(&args.input),
    };
    match target {
        InspectTarget::Manifest => {
            let report = manifest::inspect_manifest(Path::new(&args.input))?;
            print!("{report}");
        }
        InspectTarget::Source => {
            let input_path = input_path(&args.input);
            let spec =
                load_source_spec(&args.input, args.format, args.no_header, args.sheet.clone())?;
            let doc = parse_source(&spec)?;
            let request = manifest::resolve_source_request(
                &doc,
                manifest::SourceRequestOptions {
                    input_path,
                    output_path: args.output.as_deref(),
                    backend: args.backend,
                    artifact: args.artifact,
                    style: args.style.as_deref(),
                    root_name: args.root_name.as_deref(),
                    require_explicit_backend_without_output: false,
                },
            )?;
            print!("{}", format_source_inspect(&doc, &request));
        }
    }
    Ok(())
}

fn run_init(args: InitArgs) -> Result<(), DocpackError> {
    manifest::init_template(args.path.as_deref(), args.force)?;
    Ok(())
}

fn load_source_spec(
    input: &str,
    format: Option<SourceFormat>,
    no_header: bool,
    sheet: Option<String>,
) -> Result<SourceSpec, DocpackError> {
    if input == "-" {
        let format = match format {
            Some(format) => format,
            None => return Err(detect_format(None).unwrap_err()),
        };
        let mut bytes = Vec::new();
        io::stdin()
            .read_to_end(&mut bytes)
            .map_err(|source| DocpackError::Io {
                origin: Origin::Stdin,
                source,
            })?;
        Ok(SourceSpec::from_stdin(
            "data", format, bytes, no_header, sheet,
        ))
    } else {
        let path = PathBuf::from(input);
        SourceSpec::from_path(
            SourceSpec::input_stem(&path),
            path,
            format,
            no_header,
            sheet,
        )
    }
}

fn infer_inspect_target(input: &str) -> InspectTarget {
    if input == "-" {
        return InspectTarget::Source;
    }
    let path = Path::new(input);
    if manifest::detect_inspect_manifest(path) {
        InspectTarget::Manifest
    } else {
        InspectTarget::Source
    }
}

fn input_path(input: &str) -> Option<&Path> {
    if input == "-" {
        None
    } else {
        Some(Path::new(input))
    }
}

fn format_source_inspect(doc: &docpack::Document, request: &docpack::RenderRequest) -> String {
    let mut output = String::new();
    writeln!(output, "Source").unwrap();
    writeln!(output, "  id: {}", doc.source_id).unwrap();
    writeln!(output, "  origin: {}", doc.meta.origin).unwrap();
    writeln!(output, "  format: {}", doc.meta.format).unwrap();
    writeln!(output).unwrap();
    writeln!(output, "Normalized Shape").unwrap();
    writeln!(output, "  top_level_shape: {}", doc.meta.top_level_shape).unwrap();
    writeln!(output).unwrap();
    writeln!(output, "Metadata").unwrap();
    writeln!(
        output,
        "  header_present: {}",
        doc.meta
            .header_present
            .map(|value| value.to_string())
            .unwrap_or_else(|| "n/a".to_string())
    )
    .unwrap();
    writeln!(
        output,
        "  tabular_columns: {}",
        doc.meta
            .tabular_columns
            .as_ref()
            .map(|columns| columns.join(", "))
            .unwrap_or_else(|| "n/a".to_string())
    )
    .unwrap();
    writeln!(output).unwrap();
    writeln!(output, "Resolved Render Defaults").unwrap();
    writeln!(output, "  backend: {}", request.backend).unwrap();
    writeln!(output, "  artifact: {}", request.artifact).unwrap();
    writeln!(output, "  style: {}", request.style).unwrap();
    writeln!(output, "  root_name: {}", request.root_name).unwrap();
    output
}

fn write_file(path: &Path, body: &str) -> Result<(), DocpackError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| DocpackError::Io {
            origin: Origin::File(parent.to_path_buf()),
            source,
        })?;
    }
    std::fs::write(path, body).map_err(|source| DocpackError::Io {
        origin: Origin::File(path.to_path_buf()),
        source,
    })
}
