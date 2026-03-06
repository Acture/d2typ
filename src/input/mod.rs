mod csv;
mod detect;
mod json;
mod source;
mod toml;
mod xlsx;
mod yaml;

pub use detect::detect_format;
pub use source::SourceSpec;

use crate::core::{Document, TopLevelShape, Value};
use crate::error::DocpackResult;

pub fn parse_source(spec: &SourceSpec) -> DocpackResult<Document> {
    match spec.format {
        crate::core::SourceFormat::Csv => csv::parse(spec),
        crate::core::SourceFormat::Json => json::parse(spec),
        crate::core::SourceFormat::Yaml => yaml::parse(spec),
        crate::core::SourceFormat::Toml => toml::parse(spec),
        crate::core::SourceFormat::Xlsx => xlsx::parse(spec),
    }
}

pub(crate) fn infer_shape(value: &Value) -> TopLevelShape {
    match value {
        Value::Null | Value::Bool(_) | Value::Integer(_) | Value::Float(_) | Value::String(_) => {
            TopLevelShape::Scalar
        }
        Value::List(_) => TopLevelShape::List,
        Value::Object(_) => TopLevelShape::Object,
    }
}

pub(crate) fn child_path(base: &str, segment: impl AsRef<str>) -> String {
    if base.is_empty() {
        format!("/{}", segment.as_ref())
    } else {
        format!("{base}/{}", segment.as_ref())
    }
}

pub(crate) fn coerce_text_cell(value: &str) -> Value {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
        Value::Null
    } else if let Ok(parsed) = trimmed.parse::<bool>() {
        Value::Bool(parsed)
    } else if let Ok(parsed) = trimmed.parse::<i64>() {
        Value::Integer(parsed)
    } else if let Ok(parsed) = trimmed.parse::<f64>() {
        Value::Float(parsed)
    } else {
        Value::String(value.to_string())
    }
}
