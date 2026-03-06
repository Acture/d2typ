use std::collections::BTreeMap;

use serde_json::Value as JsonValue;

use crate::core::{Document, SourceMeta, Value};
use crate::error::{DocpackError, DocpackResult};
use crate::input::{SourceSpec, infer_shape};

pub fn parse(spec: &SourceSpec) -> DocpackResult<Document> {
    let value: JsonValue =
        serde_json::from_slice(&spec.bytes).map_err(|error| DocpackError::Parse {
            format: spec.format,
            origin: spec.origin.clone(),
            detail: error.to_string(),
            path: None,
        })?;
    let root = convert_json(value);
    Ok(Document {
        source_id: spec.source_id.clone(),
        meta: SourceMeta {
            format: spec.format,
            origin: spec.origin.clone(),
            top_level_shape: infer_shape(&root),
            tabular_columns: None,
            header_present: None,
        },
        root,
    })
}

fn convert_json(value: JsonValue) -> Value {
    match value {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(value) => Value::Bool(value),
        JsonValue::Number(value) => {
            if let Some(integer) = value.as_i64() {
                Value::Integer(integer)
            } else if let Some(unsigned) = value.as_u64() {
                match i64::try_from(unsigned) {
                    Ok(integer) => Value::Integer(integer),
                    Err(_) => Value::Float(unsigned as f64),
                }
            } else {
                Value::Float(value.as_f64().unwrap_or_default())
            }
        }
        JsonValue::String(value) => Value::String(value),
        JsonValue::Array(values) => Value::List(values.into_iter().map(convert_json).collect()),
        JsonValue::Object(values) => Value::Object(
            values
                .into_iter()
                .map(|(key, value)| (key, convert_json(value)))
                .collect::<BTreeMap<_, _>>(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::core::{Origin, SourceFormat, TopLevelShape, Value};
    use crate::input::SourceSpec;

    #[test]
    fn parses_json_object_into_document() {
        let spec = SourceSpec::from_stdin(
            "data",
            SourceFormat::Json,
            br#"{"name":"Alice","age":30,"active":true}"#.to_vec(),
            false,
            None,
        );
        let doc = parse(&spec).unwrap();
        assert_eq!(doc.source_id, "data");
        assert_eq!(doc.meta.origin, Origin::Stdin);
        assert_eq!(doc.meta.top_level_shape, TopLevelShape::Object);
        match &doc.root {
            Value::Object(map) => {
                assert_eq!(map.get("active"), Some(&Value::Bool(true)));
                assert_eq!(map.get("age"), Some(&Value::Integer(30)));
            }
            _ => panic!("expected object root"),
        }
    }
}
