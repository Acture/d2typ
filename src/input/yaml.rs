use std::collections::BTreeMap;

use serde_yaml::Value as YamlValue;

use crate::core::{Document, SourceMeta, Value};
use crate::error::{DocpackError, DocpackResult};
use crate::input::{SourceSpec, child_path, infer_shape};

pub fn parse(spec: &SourceSpec) -> DocpackResult<Document> {
    let value: YamlValue =
        serde_yaml::from_slice(&spec.bytes).map_err(|error| DocpackError::Parse {
            format: spec.format,
            origin: spec.origin.clone(),
            detail: error.to_string(),
            path: None,
        })?;
    let root = convert_yaml(value, spec, "")?;
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

fn convert_yaml(value: YamlValue, spec: &SourceSpec, path: &str) -> DocpackResult<Value> {
    match value {
        YamlValue::Null => Ok(Value::Null),
        YamlValue::Bool(value) => Ok(Value::Bool(value)),
        YamlValue::Number(value) => {
            if let Some(integer) = value.as_i64() {
                Ok(Value::Integer(integer))
            } else if let Some(unsigned) = value.as_u64() {
                match i64::try_from(unsigned) {
                    Ok(integer) => Ok(Value::Integer(integer)),
                    Err(_) => Ok(Value::Float(unsigned as f64)),
                }
            } else {
                Ok(Value::Float(value.as_f64().unwrap_or_default()))
            }
        }
        YamlValue::String(value) => Ok(Value::String(value)),
        YamlValue::Sequence(values) => {
            let mut result = Vec::with_capacity(values.len());
            for (index, value) in values.into_iter().enumerate() {
                result.push(convert_yaml(
                    value,
                    spec,
                    &child_path(path, index.to_string()),
                )?);
            }
            Ok(Value::List(result))
        }
        YamlValue::Mapping(values) => {
            let mut result = BTreeMap::new();
            for (key, value) in values {
                let Some(key) = key.as_str() else {
                    return Err(DocpackError::UnsupportedKey {
                        format: spec.format,
                        origin: spec.origin.clone(),
                        path: if path.is_empty() {
                            "/".to_string()
                        } else {
                            path.to_string()
                        },
                        key_repr: render_yaml_key(&key),
                    });
                };
                result.insert(
                    key.to_string(),
                    convert_yaml(value, spec, &child_path(path, key))?,
                );
            }
            Ok(Value::Object(result))
        }
        YamlValue::Tagged(tagged) => convert_yaml(tagged.value, spec, path),
    }
}

fn render_yaml_key(key: &YamlValue) -> String {
    match key {
        YamlValue::Null => "null".to_string(),
        YamlValue::Bool(value) => value.to_string(),
        YamlValue::Number(value) => value.to_string(),
        YamlValue::String(value) => value.clone(),
        YamlValue::Sequence(_) => "<sequence>".to_string(),
        YamlValue::Mapping(_) => "<mapping>".to_string(),
        YamlValue::Tagged(_) => "<tagged>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::core::SourceFormat;
    use crate::input::SourceSpec;

    #[test]
    fn rejects_non_string_yaml_keys() {
        let spec = SourceSpec::from_stdin(
            "data",
            SourceFormat::Yaml,
            b"? [1, 2]\n: value\n".to_vec(),
            false,
            None,
        );
        assert!(parse(&spec).is_err());
    }
}
