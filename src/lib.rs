pub mod backend;
pub mod core;
pub mod error;
pub mod input;
pub mod manifest;

pub use backend::{
    ArtifactKind, Backend, BackendKind, RenderRequest, RenderedArtifact, render_document,
    validate_request,
};
pub use core::{Document, Origin, SourceFormat, SourceMeta, TopLevelShape, Value};
pub use error::{DocpackError, DocpackResult};
pub use input::{SourceSpec, detect_format, parse_source};
