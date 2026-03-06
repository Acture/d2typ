use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use toml::Value as TomlValue;

use crate::core::SourceFormat;
use crate::error::{DocpackError, DocpackResult};
use crate::manifest::{Manifest, SourceEntry};

#[derive(Debug, Clone)]
pub struct LoadedManifest {
    pub path: PathBuf,
    pub dir: PathBuf,
    pub manifest: Manifest,
}

impl LoadedManifest {
    pub fn resolve_source_path(&self, source: &SourceEntry) -> PathBuf {
        if source.path.is_absolute() {
            source.path.clone()
        } else {
            self.dir.join(&source.path)
        }
    }

    pub fn source_by_id(&self, id: &str) -> Option<&SourceEntry> {
        self.manifest.sources.iter().find(|source| source.id == id)
    }
}

pub fn load_manifest(path: &Path) -> DocpackResult<LoadedManifest> {
    let contents = fs::read_to_string(path).map_err(|source| DocpackError::ManifestLoad {
        path: path.to_path_buf(),
        detail: source.to_string(),
    })?;
    let manifest: Manifest =
        toml::from_str(&contents).map_err(|error| DocpackError::ManifestLoad {
            path: path.to_path_buf(),
            detail: error.to_string(),
        })?;
    validate_manifest(path, &manifest)?;

    Ok(LoadedManifest {
        path: path.to_path_buf(),
        dir: path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
        manifest,
    })
}

pub fn detect_inspect_manifest(path: &Path) -> bool {
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "docpack.toml")
    {
        return true;
    }
    let Ok(contents) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(value) = toml::from_str::<TomlValue>(&contents) else {
        return false;
    };
    match value {
        TomlValue::Table(table) => table.contains_key("sources") || table.contains_key("outputs"),
        _ => false,
    }
}

fn validate_manifest(path: &Path, manifest: &Manifest) -> DocpackResult<()> {
    let mut problems = Vec::new();
    let mut source_ids = HashSet::new();
    for source in &manifest.sources {
        if source.id.trim().is_empty() {
            problems.push("sources.id must not be empty".to_string());
        }
        if !source_ids.insert(source.id.clone()) {
            problems.push(format!("duplicate source id '{}'", source.id));
        }
        validate_source_options(source, &mut problems);
    }

    let mut output_ids = HashSet::new();
    for output in &manifest.outputs {
        if output.id.trim().is_empty() {
            problems.push("outputs.id must not be empty".to_string());
        }
        if !output_ids.insert(output.id.clone()) {
            problems.push(format!("duplicate output id '{}'", output.id));
        }
        if !source_ids.contains(&output.source) {
            problems.push(format!(
                "output '{}' references missing source '{}'",
                output.id, output.source
            ));
        }
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(DocpackError::ManifestInvalid {
            path: path.to_path_buf(),
            problems,
        })
    }
}

fn validate_source_options(source: &SourceEntry, problems: &mut Vec<String>) {
    let format = source
        .format
        .or_else(|| SourceFormat::from_extension(&source.path));

    if source.no_header.unwrap_or(false) {
        match format {
            Some(SourceFormat::Csv | SourceFormat::Xlsx) => {}
            Some(other) => problems.push(format!(
                "source '{}' sets no_header, but that option is only valid for csv or xlsx sources (got {})",
                source.id, other
            )),
            None => problems.push(format!(
                "source '{}' sets no_header, but its format cannot be inferred; set format = \"csv\" or format = \"xlsx\" explicitly",
                source.id
            )),
        }
    }

    if source.sheet.is_some() {
        match format {
            Some(SourceFormat::Xlsx) => {}
            Some(other) => problems.push(format!(
                "source '{}' sets sheet, but that option is only valid for xlsx sources (got {})",
                source.id, other
            )),
            None => problems.push(format!(
                "source '{}' sets sheet, but its format cannot be inferred; set format = \"xlsx\" explicitly",
                source.id
            )),
        }
    }
}
