use std::collections::BTreeMap;

use toml::Value as TomlValue;

use crate::core::{Document, SourceMeta, Value};
use crate::error::{DocpackError, DocpackResult};
use crate::input::{SourceSpec, child_path, infer_shape};

pub fn parse(spec: &SourceSpec) -> DocpackResult<Document> {
    let text = std::str::from_utf8(&spec.bytes).map_err(|error| DocpackError::Parse {
        format: spec.format,
        origin: spec.origin.clone(),
        detail: error.to_string(),
        path: None,
    })?;
    let value: TomlValue = toml::from_str(text).map_err(|error| DocpackError::Parse {
        format: spec.format,
        origin: spec.origin.clone(),
        detail: error.to_string(),
        path: None,
    })?;
    let root = convert_toml(value, "")?;
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

fn convert_toml(value: TomlValue, path: &str) -> DocpackResult<Value> {
    match value {
        TomlValue::String(value) => Ok(Value::String(value)),
        TomlValue::Integer(value) => Ok(Value::Integer(value)),
        TomlValue::Float(value) => Ok(Value::Float(value)),
        TomlValue::Boolean(value) => Ok(Value::Bool(value)),
        TomlValue::Datetime(value) => Ok(Value::String(value.to_string())),
        TomlValue::Array(values) => {
            let mut result = Vec::with_capacity(values.len());
            for (index, value) in values.into_iter().enumerate() {
                result.push(convert_toml(value, &child_path(path, index.to_string()))?);
            }
            Ok(Value::List(result))
        }
        TomlValue::Table(values) => Ok(Value::Object(
            values
                .into_iter()
                .map(|(key, value)| {
                    let path = child_path(path, &key);
                    Ok((key, convert_toml(value, &path)?))
                })
                .collect::<DocpackResult<BTreeMap<_, _>>>()?,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::core::{SourceFormat, Value};
    use crate::input::SourceSpec;

    #[test]
    fn normalizes_toml_datetime_as_string() {
        let spec = SourceSpec::from_stdin(
            "data",
            SourceFormat::Toml,
            b"created_at = 1979-05-27T07:32:00Z\n".to_vec(),
            false,
            None,
        );
        let doc = parse(&spec).unwrap();
        match doc.root {
            Value::Object(map) => assert_eq!(
                map.get("created_at"),
                Some(&Value::String("1979-05-27T07:32:00Z".to_string()))
            ),
            _ => panic!("expected object root"),
        }
    }
}
