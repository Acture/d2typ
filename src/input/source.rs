use std::fs;
use std::path::{Path, PathBuf};

use crate::core::{Origin, SourceFormat};
use crate::error::{DocpackError, DocpackResult};
use crate::input::detect_format;

/// Raw source description consumed by the input normalization layer.
#[derive(Debug, Clone)]
pub struct SourceSpec {
    pub source_id: String,
    pub origin: Origin,
    pub format: SourceFormat,
    pub bytes: Vec<u8>,
    pub no_header: bool,
    pub sheet: Option<String>,
}

impl SourceSpec {
    /// Loads a source from disk, reading its full contents into memory.
    pub fn from_path(
        source_id: impl Into<String>,
        path: PathBuf,
        format: Option<SourceFormat>,
        no_header: bool,
        sheet: Option<String>,
    ) -> DocpackResult<Self> {
        let detected = match format {
            Some(format) => format,
            None => detect_format(Some(&path))?,
        };
        let bytes = fs::read(&path).map_err(|source| DocpackError::Io {
            origin: Origin::File(path.clone()),
            source,
        })?;
        Ok(Self {
            source_id: source_id.into(),
            origin: Origin::File(path),
            format: detected,
            bytes,
            no_header,
            sheet,
        })
    }

    /// Builds a source specification from stdin bytes.
    pub fn from_stdin(
        source_id: impl Into<String>,
        format: SourceFormat,
        bytes: Vec<u8>,
        no_header: bool,
        sheet: Option<String>,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            origin: Origin::Stdin,
            format,
            bytes,
            no_header,
            sheet,
        }
    }

    /// Derives a fallback source identifier from an input path.
    pub fn input_stem(path: &Path) -> String {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .filter(|stem| !stem.is_empty())
            .unwrap_or("data")
            .to_string()
    }
}
