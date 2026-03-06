mod infer;
mod load;
mod model;

pub use infer::{
    ResolvedOutput, SourceRequestOptions, resolve_manifest_output, resolve_source_request,
    sanitize_root_name,
};
pub use load::{LoadedManifest, detect_inspect_manifest, load_manifest};
pub use model::{Manifest, OutputEntry, ProjectConfig, SourceEntry};

use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::backend::render_document;
use crate::core::{Document, Origin};
use crate::error::{DocpackError, DocpackResult};
use crate::input::{SourceSpec, parse_source};

/// Builds every output defined in a manifest file.
pub fn build(manifest_path: Option<&Path>) -> DocpackResult<Vec<PathBuf>> {
    let manifest_path = manifest_path.unwrap_or_else(|| Path::new("docpack.toml"));
    let loaded = load_manifest(manifest_path)?;
    let mut cached_docs: HashMap<String, Document> = HashMap::new();
    let mut written = Vec::new();

    for output in &loaded.manifest.outputs {
        let source = loaded
            .source_by_id(&output.source)
            .expect("validated manifest source");
        let doc = if let Some(doc) = cached_docs.get(&source.id) {
            doc.clone()
        } else {
            let spec = SourceSpec::from_path(
                source.id.clone(),
                loaded.resolve_source_path(source),
                source.format,
                source.no_header.unwrap_or(false),
                source.sheet.clone(),
            )?;
            let doc = parse_source(&spec)?;
            cached_docs.insert(source.id.clone(), doc.clone());
            doc
        };
        let resolved = resolve_manifest_output(&loaded, source, output, &doc)?;
        write_output_file(
            &resolved.output_path,
            &render_document(&doc, &resolved.request)?.body,
        )?;
        written.push(resolved.output_path);
    }

    Ok(written)
}

/// Produces a human-readable manifest inspection report.
pub fn inspect_manifest(path: &Path) -> DocpackResult<String> {
    let loaded = load_manifest(path)?;
    let mut cached_docs: HashMap<String, Document> = HashMap::new();
    let mut output = String::new();

    writeln!(output, "Project").unwrap();
    writeln!(output, "  path: {}", loaded.path.display()).unwrap();
    writeln!(
        output,
        "  name: {}",
        loaded
            .manifest
            .project
            .as_ref()
            .and_then(|project| project.name.as_deref())
            .unwrap_or("(unset)")
    )
    .unwrap();
    writeln!(
        output,
        "  output_dir: {}",
        loaded
            .manifest
            .project
            .as_ref()
            .and_then(|project| project.output_dir.as_ref())
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "(manifest dir)".to_string())
    )
    .unwrap();
    writeln!(output).unwrap();

    writeln!(output, "Sources").unwrap();
    for source in &loaded.manifest.sources {
        let spec = SourceSpec::from_path(
            source.id.clone(),
            loaded.resolve_source_path(source),
            source.format,
            source.no_header.unwrap_or(false),
            source.sheet.clone(),
        )?;
        let doc = parse_source(&spec)?;
        writeln!(
            output,
            "  - {}: {} (format: {}, shape: {})",
            source.id,
            loaded.resolve_source_path(source).display(),
            doc.meta.format,
            doc.meta.top_level_shape
        )
        .unwrap();
        if let Some(columns) = &doc.meta.tabular_columns {
            writeln!(output, "    columns: {}", columns.join(", ")).unwrap();
        }
        cached_docs.insert(source.id.clone(), doc);
    }
    writeln!(output).unwrap();

    writeln!(output, "Outputs").unwrap();
    for entry in &loaded.manifest.outputs {
        let resolved = {
            let source = loaded
                .source_by_id(&entry.source)
                .expect("validated manifest source");
            let doc = cached_docs
                .get(&source.id)
                .expect("manifest inspect parses every source");
            resolve_manifest_output(&loaded, source, entry, doc)?
        };
        writeln!(
            output,
            "  - {}: {} -> {}",
            resolved.output_id,
            resolved.source_id,
            resolved.output_path.display()
        )
        .unwrap();
    }
    writeln!(output).unwrap();

    writeln!(output, "Resolved Build Plan").unwrap();
    for entry in &loaded.manifest.outputs {
        let source = loaded
            .source_by_id(&entry.source)
            .expect("validated manifest source");
        let doc = cached_docs
            .get(&source.id)
            .expect("manifest inspect parses every source");
        let resolved = resolve_manifest_output(&loaded, source, entry, doc)?;
        writeln!(
            output,
            "  - {}: backend={}, artifact={}, style={}, root_name={}",
            resolved.output_id,
            resolved.request.backend,
            resolved.request.artifact,
            resolved.request.style,
            resolved.request.root_name
        )
        .unwrap();
    }

    Ok(output)
}

/// Creates a minimal manifest template on disk.
pub fn init_template(path: Option<&Path>, force: bool) -> DocpackResult<PathBuf> {
    let target = resolve_init_target(path);
    if target.exists() && !force {
        return Err(DocpackError::Io {
            origin: Origin::File(target.clone()),
            source: std::io::Error::new(
                ErrorKind::AlreadyExists,
                "refusing to overwrite existing manifest without --force",
            ),
        });
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|source| DocpackError::Io {
            origin: Origin::File(parent.to_path_buf()),
            source,
        })?;
    }
    fs::write(&target, manifest_template()).map_err(|source| DocpackError::Io {
        origin: Origin::File(target.clone()),
        source,
    })?;
    Ok(target)
}

fn resolve_init_target(path: Option<&Path>) -> PathBuf {
    match path {
        None => PathBuf::from("docpack.toml"),
        Some(path) if path.extension().and_then(|ext| ext.to_str()) == Some("toml") => {
            path.to_path_buf()
        }
        Some(path) => path.join("docpack.toml"),
    }
}

fn manifest_template() -> &'static str {
    r#"[project]
name = "example-project"
output_dir = "generated"

#[[sources]]
#id = "sales"
#path = "data/sales.csv"
#format = "csv"
#no_header = false

#[[outputs]]
#id = "sales_typst"
#source = "sales"
#path = "sales.typ"
#backend = "typst"
#artifact = "data-module"
#style = "typst-official"
#root_name = "sales"
"#
}

fn write_output_file(path: &Path, body: &str) -> DocpackResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| DocpackError::Io {
            origin: Origin::File(parent.to_path_buf()),
            source,
        })?;
    }
    fs::write(path, body).map_err(|source| DocpackError::Io {
        origin: Origin::File(path.to_path_buf()),
        source,
    })
}
