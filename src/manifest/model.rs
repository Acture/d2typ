use std::path::PathBuf;

use serde::Deserialize;

use crate::backend::{ArtifactKind, BackendKind};
use crate::core::SourceFormat;

/// Top-level manifest structure loaded from `docpack.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub project: Option<ProjectConfig>,
    #[serde(default)]
    pub sources: Vec<SourceEntry>,
    #[serde(default)]
    pub outputs: Vec<OutputEntry>,
}

/// Project-level manifest settings.
#[derive(Debug, Clone, Deserialize)]
pub struct ProjectConfig {
    pub name: Option<String>,
    pub output_dir: Option<PathBuf>,
}

/// Source entry declared in a manifest.
#[derive(Debug, Clone, Deserialize)]
pub struct SourceEntry {
    pub id: String,
    pub path: PathBuf,
    pub format: Option<SourceFormat>,
    pub no_header: Option<bool>,
    pub sheet: Option<String>,
}

/// Output entry declared in a manifest.
#[derive(Debug, Clone, Deserialize)]
pub struct OutputEntry {
    pub id: String,
    pub source: String,
    pub path: PathBuf,
    pub backend: Option<BackendKind>,
    pub artifact: Option<ArtifactKind>,
    pub style: Option<String>,
    pub root_name: Option<String>,
}
