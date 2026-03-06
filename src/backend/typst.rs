use std::fmt::Write;

use crate::backend::{ArtifactKind, Backend, BackendKind, RenderRequest, RenderedArtifact};
use crate::core::{Document, Value};
use crate::error::{DocpackError, DocpackResult};

pub struct TypstBackend;

impl Backend for TypstBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::Typst
    }

    fn render(&self, doc: &Document, req: &RenderRequest) -> DocpackResult<RenderedArtifact> {
        let body = match req.artifact {
            ArtifactKind::DataModule => render_data_module(doc, req),
            ArtifactKind::TableFragment => render_table_fragment(doc, req),
        }?;
        Ok(RenderedArtifact { body })
    }
}

fn render_data_module(doc: &Document, req: &RenderRequest) -> DocpackResult<String> {
    Ok(format!(
        "#let {} = {}\n",
        req.root_name,
        render_value(&doc.root)
    ))
}

fn render_table_fragment(doc: &Document, req: &RenderRequest) -> DocpackResult<String> {
    let width = doc.table_width().unwrap_or(0);
    let rows = collect_table_rows(doc, req)?;
    let mut output = String::new();
    writeln!(output, "#table(").map_err(into_render_error(req))?;
    writeln!(output, "  columns: {width},").map_err(into_render_error(req))?;
    if let Some(columns) = &doc.meta.tabular_columns {
        write!(output, "  table.header").map_err(into_render_error(req))?;
        for column in columns {
            write!(output, "[{}]", escape_table_cell(column)).map_err(into_render_error(req))?;
        }
        writeln!(output, ",").map_err(into_render_error(req))?;
    }
    for row in rows {
        write!(output, "  ").map_err(into_render_error(req))?;
        for (index, cell) in row.iter().enumerate() {
            if index > 0 {
                write!(output, ", ").map_err(into_render_error(req))?;
            }
            write!(output, "[{}]", escape_table_cell(cell)).map_err(into_render_error(req))?;
        }
        writeln!(output, ",").map_err(into_render_error(req))?;
    }
    writeln!(output, ")").map_err(into_render_error(req))?;
    Ok(output)
}

fn collect_table_rows(doc: &Document, req: &RenderRequest) -> DocpackResult<Vec<Vec<String>>> {
    match &doc.root {
        Value::List(rows) => {
            rows.iter()
                .map(|row| match row {
                    Value::Object(values) => {
                        let columns = doc.meta.tabular_columns.as_ref().ok_or_else(|| {
                            DocpackError::Render {
                                backend: req.backend,
                                artifact: req.artifact,
                                detail: "record-shaped table data requires tabular columns"
                                    .to_string(),
                            }
                        })?;
                        columns
                            .iter()
                            .map(|column| {
                                render_scalar_cell(values.get(column).unwrap_or(&Value::Null), req)
                            })
                            .collect()
                    }
                    Value::List(values) => values
                        .iter()
                        .map(|value| render_scalar_cell(value, req))
                        .collect(),
                    _ => Err(DocpackError::Render {
                        backend: req.backend,
                        artifact: req.artifact,
                        detail: "table-fragment expects rows as objects or lists".to_string(),
                    }),
                })
                .collect()
        }
        _ => Err(DocpackError::Render {
            backend: req.backend,
            artifact: req.artifact,
            detail: "table-fragment expects a list root".to_string(),
        }),
    }
}

fn render_scalar_cell(value: &Value, req: &RenderRequest) -> DocpackResult<String> {
    value.scalar_text().ok_or_else(|| DocpackError::Render {
        backend: req.backend,
        artifact: req.artifact,
        detail: "table-fragment only accepts scalar cell values".to_string(),
    })
}

fn render_value(value: &Value) -> String {
    match value {
        Value::Null => "none".to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Integer(value) => value.to_string(),
        Value::Float(value) => value.to_string(),
        Value::String(value) => format!("\"{}\"", escape_string(value)),
        Value::List(values) => format!(
            "({})",
            values
                .iter()
                .map(render_value)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Object(values) => {
            if values.is_empty() {
                "(:)".to_string()
            } else {
                format!(
                    "({})",
                    values
                        .iter()
                        .map(|(key, value)| format!(
                            "\"{}\": {}",
                            escape_string(key),
                            render_value(value)
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}

fn escape_string(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn escape_table_cell(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(']', "\\]")
        .replace('\n', "\\n")
}

fn into_render_error(req: &RenderRequest) -> impl FnOnce(std::fmt::Error) -> DocpackError + '_ {
    move |_| DocpackError::Render {
        backend: req.backend,
        artifact: req.artifact,
        detail: "failed to format Typst output".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::TypstBackend;
    use crate::backend::{ArtifactKind, Backend, BackendKind, RenderRequest};
    use crate::core::{Document, Origin, SourceFormat, SourceMeta, TopLevelShape, Value};

    #[test]
    fn renders_typst_data_module() {
        let mut map = BTreeMap::new();
        map.insert("active".to_string(), Value::Bool(true));
        map.insert("age".to_string(), Value::Integer(30));
        map.insert("name".to_string(), Value::String("Alice".to_string()));
        let doc = Document {
            source_id: "data".to_string(),
            root: Value::Object(map),
            meta: SourceMeta {
                format: SourceFormat::Json,
                origin: Origin::Stdin,
                top_level_shape: TopLevelShape::Object,
                tabular_columns: None,
                header_present: None,
            },
        };
        let req = RenderRequest {
            backend: BackendKind::Typst,
            artifact: ArtifactKind::DataModule,
            style: "typst-official".to_string(),
            root_name: "data".to_string(),
        };
        let rendered = TypstBackend.render(&doc, &req).unwrap();
        assert_eq!(
            rendered.body,
            "#let data = (\"active\": true, \"age\": 30, \"name\": \"Alice\")\n"
        );
    }
}
