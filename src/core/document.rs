use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use clap::ValueEnum;
use serde::Deserialize;

use crate::core::Value;

/// A normalized source document plus source-specific metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub source_id: String,
    pub root: Value,
    pub meta: SourceMeta,
}

impl Document {
    /// Returns `true` when the document originated from a tabular source.
    pub fn is_tabular(&self) -> bool {
        self.meta.is_tabular()
    }

    /// Returns the tabular width when the document carries table metadata.
    pub fn table_width(&self) -> Option<usize> {
        if let Some(columns) = &self.meta.tabular_columns {
            return Some(columns.len());
        }

        match &self.root {
            Value::List(rows) => rows.first().and_then(|row| match row {
                Value::List(cells) => Some(cells.len()),
                _ => None,
            }),
            _ => None,
        }
    }
}

/// Supported source input formats.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum SourceFormat {
    Csv,
    Json,
    #[serde(alias = "yml")]
    #[value(alias = "yml")]
    Yaml,
    Toml,
    Xlsx,
}

impl SourceFormat {
    /// Infers a supported source format from a file extension.
    pub fn from_extension(path: &Path) -> Option<Self> {
        match path
            .extension()
            .and_then(|ext| ext.to_str())?
            .to_ascii_lowercase()
            .as_str()
        {
            "csv" => Some(Self::Csv),
            "json" => Some(Self::Json),
            "yaml" | "yml" => Some(Self::Yaml),
            "toml" => Some(Self::Toml),
            "xls" | "xlsx" => Some(Self::Xlsx),
            _ => None,
        }
    }
}

impl Display for SourceFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::Xlsx => "xlsx",
        };
        write!(f, "{value}")
    }
}

/// Where a source was loaded from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    File(PathBuf),
    Stdin,
}

impl Display for Origin {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File(path) => write!(f, "{}", path.display()),
            Self::Stdin => write!(f, "stdin"),
        }
    }
}

/// Source metadata retained alongside the normalized value tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMeta {
    pub format: SourceFormat,
    pub origin: Origin,
    pub top_level_shape: TopLevelShape,
    pub tabular_columns: Option<Vec<String>>,
    pub header_present: Option<bool>,
}

impl SourceMeta {
    /// Returns `true` when the source shape is tabular.
    pub fn is_tabular(&self) -> bool {
        matches!(
            self.top_level_shape,
            TopLevelShape::TabularRecords | TopLevelShape::TabularMatrix
        )
    }
}

/// Top-level source shape used for backend inference and table rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopLevelShape {
    Scalar,
    List,
    Object,
    TabularRecords,
    TabularMatrix,
}

impl Display for TopLevelShape {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Scalar => "scalar",
            Self::List => "list",
            Self::Object => "object",
            Self::TabularRecords => "tabular-records",
            Self::TabularMatrix => "tabular-matrix",
        };
        write!(f, "{value}")
    }
}
