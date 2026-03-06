use std::fmt::{Display, Formatter};
use std::io;
use std::path::PathBuf;

use crate::backend::{ArtifactKind, BackendKind};
use crate::core::{Origin, SourceFormat};

/// Result type used across the public `docpack` library APIs.
pub type DocpackResult<T> = Result<T, DocpackError>;

/// Structured error type for parsing, inference, rendering, and manifest work.
#[derive(Debug)]
pub enum DocpackError {
    Io {
        origin: Origin,
        source: io::Error,
    },
    DetectFormat {
        origin: Origin,
        detail: String,
    },
    Parse {
        format: SourceFormat,
        origin: Origin,
        detail: String,
        path: Option<String>,
    },
    UnsupportedKey {
        format: SourceFormat,
        origin: Origin,
        path: String,
        key_repr: String,
    },
    InvalidSheet {
        path: PathBuf,
        requested: String,
        available: Vec<String>,
    },
    InconsistentRowWidth {
        origin: Origin,
        expected: usize,
        actual: usize,
        row_index: usize,
    },
    InvalidRootName {
        supplied: String,
    },
    ManifestLoad {
        path: PathBuf,
        detail: String,
    },
    ManifestInvalid {
        path: PathBuf,
        problems: Vec<String>,
    },
    Inference {
        detail: String,
    },
    Render {
        backend: BackendKind,
        artifact: ArtifactKind,
        detail: String,
    },
}

impl Display for DocpackError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { origin, source } => write!(f, "I/O error at {origin}: {source}"),
            Self::DetectFormat { origin, detail } => {
                write!(f, "failed to detect format for {origin}: {detail}")
            }
            Self::Parse {
                format,
                origin,
                detail,
                path,
            } => {
                if let Some(path) = path {
                    write!(
                        f,
                        "failed to parse {format} from {origin} at {path}: {detail}"
                    )
                } else {
                    write!(f, "failed to parse {format} from {origin}: {detail}")
                }
            }
            Self::UnsupportedKey {
                format,
                origin,
                path,
                key_repr,
            } => write!(
                f,
                "unsupported {format} mapping key at {origin} {path}: {key_repr}"
            ),
            Self::InvalidSheet {
                path,
                requested,
                available,
            } => write!(
                f,
                "sheet '{requested}' was not found in {}. available sheets: {}",
                path.display(),
                available.join(", ")
            ),
            Self::InconsistentRowWidth {
                origin,
                expected,
                actual,
                row_index,
            } => write!(
                f,
                "inconsistent row width at {origin} row {row_index}: expected {expected}, got {actual}"
            ),
            Self::InvalidRootName { supplied } => {
                write!(f, "invalid root name after sanitization: {supplied}")
            }
            Self::ManifestLoad { path, detail } => {
                write!(f, "failed to load manifest {}: {detail}", path.display())
            }
            Self::ManifestInvalid { path, problems } => write!(
                f,
                "manifest {} is invalid:\n- {}",
                path.display(),
                problems.join("\n- ")
            ),
            Self::Inference { detail } => write!(f, "inference error: {detail}"),
            Self::Render {
                backend,
                artifact,
                detail,
            } => write!(f, "failed to render {backend} {artifact}: {detail}"),
        }
    }
}

impl std::error::Error for DocpackError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
