use std::fmt::Write;

use crate::backend::{ArtifactKind, Backend, BackendKind, RenderRequest, RenderedArtifact};
use crate::core::{Document, Value};
use crate::error::{DocpackError, DocpackResult};

pub struct LatexBackend;

impl Backend for LatexBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::Latex
    }

    fn render(&self, doc: &Document, req: &RenderRequest) -> DocpackResult<RenderedArtifact> {
        let body = match (req.artifact, req.style.as_str()) {
            (ArtifactKind::DataModule, "latex-expl3") => render_expl3_data_module(doc, req),
            (ArtifactKind::DataModule, "latex-classic-macro") => {
                render_classic_data_module(doc, req)
            }
            (ArtifactKind::TableFragment, "latex-booktabs-longtable") => {
                render_booktabs_longtable(doc, req)
            }
            (ArtifactKind::TableFragment, "latex-plain-tabular") => render_plain_tabular(doc, req),
            _ => Err(DocpackError::Render {
                backend: req.backend,
                artifact: req.artifact,
                detail: format!("unsupported LaTeX style '{}'", req.style),
            }),
        }?;
        Ok(RenderedArtifact { body })
    }
}

fn render_expl3_data_module(doc: &Document, req: &RenderRequest) -> DocpackResult<String> {
    let leaves = flatten_document(doc);
    let mut output = String::new();
    writeln!(output, "\\ExplSyntaxOn").map_err(into_render_error(req))?;
    writeln!(output, "\\prop_new:N \\g_docpack_{}_prop", req.root_name)
        .map_err(into_render_error(req))?;
    for (path, value) in leaves {
        writeln!(
            output,
            "\\prop_gput:Nnn \\g_docpack_{}_prop {{{}}} {{{}}}",
            req.root_name,
            path.join("/"),
            latex_escape(&value)
        )
        .map_err(into_render_error(req))?;
    }
    writeln!(output, "\\ExplSyntaxOff").map_err(into_render_error(req))?;
    Ok(output)
}

fn render_classic_data_module(doc: &Document, req: &RenderRequest) -> DocpackResult<String> {
    let leaves = flatten_document(doc);
    let mut output = String::new();
    for (path, value) in leaves {
        let suffix = path
            .iter()
            .map(|segment| sanitize_macro_segment(segment))
            .collect::<Vec<_>>()
            .join("__");
        writeln!(
            output,
            "\\expandafter\\def\\csname docpack@{}__{}\\endcsname{{{}}}",
            req.root_name,
            suffix,
            latex_escape(&value)
        )
        .map_err(into_render_error(req))?;
    }
    Ok(output)
}

fn render_booktabs_longtable(doc: &Document, req: &RenderRequest) -> DocpackResult<String> {
    let rows = collect_table_rows(doc, req)?;
    let width = doc.table_width().unwrap_or(0);
    let mut output = String::new();
    writeln!(output, "\\begin{{longtable}}{{{}}}", "l".repeat(width))
        .map_err(into_render_error(req))?;
    if let Some(columns) = &doc.meta.tabular_columns {
        writeln!(output, "\\toprule").map_err(into_render_error(req))?;
        writeln!(output, "{} \\\\", join_latex_row(columns)).map_err(into_render_error(req))?;
        writeln!(output, "\\midrule").map_err(into_render_error(req))?;
        writeln!(output, "\\endfirsthead").map_err(into_render_error(req))?;
        writeln!(output, "\\toprule").map_err(into_render_error(req))?;
        writeln!(output, "{} \\\\", join_latex_row(columns)).map_err(into_render_error(req))?;
        writeln!(output, "\\midrule").map_err(into_render_error(req))?;
        writeln!(output, "\\endhead").map_err(into_render_error(req))?;
    }
    for row in rows {
        writeln!(output, "{} \\\\", join_latex_row(&row)).map_err(into_render_error(req))?;
    }
    writeln!(output, "\\bottomrule").map_err(into_render_error(req))?;
    writeln!(output, "\\end{{longtable}}").map_err(into_render_error(req))?;
    Ok(output)
}

fn render_plain_tabular(doc: &Document, req: &RenderRequest) -> DocpackResult<String> {
    let rows = collect_table_rows(doc, req)?;
    let width = doc.table_width().unwrap_or(0);
    let mut output = String::new();
    writeln!(output, "\\begin{{tabular}}{{{}}}", "l".repeat(width))
        .map_err(into_render_error(req))?;
    if let Some(columns) = &doc.meta.tabular_columns {
        writeln!(output, "{} \\\\", join_latex_row(columns)).map_err(into_render_error(req))?;
        writeln!(output, "\\hline").map_err(into_render_error(req))?;
    }
    for row in rows {
        writeln!(output, "{} \\\\", join_latex_row(&row)).map_err(into_render_error(req))?;
    }
    writeln!(output, "\\end{{tabular}}").map_err(into_render_error(req))?;
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

fn flatten_value(
    value: &Value,
    path: &mut Vec<String>,
    out: &mut Vec<(Vec<String>, String)>,
    parent_is_list: bool,
) {
    match value {
        Value::Null => out.push((leaf_path(path), "none".to_string())),
        Value::Bool(value) => out.push((leaf_path(path), value.to_string())),
        Value::Integer(value) => out.push((leaf_path(path), value.to_string())),
        Value::Float(value) => out.push((leaf_path(path), value.to_string())),
        Value::String(value) => out.push((leaf_path(path), value.clone())),
        Value::List(values) => {
            for (index, value) in values.iter().enumerate() {
                path.push((index + 1).to_string());
                flatten_value(value, path, out, true);
                path.pop();
            }
            if !parent_is_list {
                let mut len_path = path.clone();
                len_path.push("__len__".to_string());
                out.push((len_path, values.len().to_string()));
            }
        }
        Value::Object(values) => {
            for (key, value) in values {
                path.push(key.clone());
                flatten_value(value, path, out, false);
                path.pop();
            }
        }
    }
}

fn leaf_path(path: &[String]) -> Vec<String> {
    if path.is_empty() {
        vec!["value".to_string()]
    } else {
        path.to_vec()
    }
}

fn flatten_document(doc: &Document) -> Vec<(Vec<String>, String)> {
    match (&doc.root, doc.meta.tabular_columns.as_ref()) {
        (Value::List(rows), Some(columns)) if doc.is_tabular() => {
            let mut out = Vec::new();
            for (row_index, row) in rows.iter().enumerate() {
                let row_number = (row_index + 1).to_string();
                match row {
                    Value::Object(values) => {
                        for column in columns {
                            if let Some(value) = values.get(column) {
                                let mut path = vec![row_number.clone(), column.clone()];
                                flatten_value(value, &mut path, &mut out, false);
                            }
                        }
                    }
                    _ => {
                        let mut path = vec![row_number];
                        flatten_value(row, &mut path, &mut out, true);
                    }
                }
            }
            out.push((vec!["__len__".to_string()], rows.len().to_string()));
            out
        }
        _ => {
            let mut out = Vec::new();
            flatten_value(&doc.root, &mut Vec::new(), &mut out, false);
            out
        }
    }
}

fn join_latex_row(cells: &[String]) -> String {
    cells
        .iter()
        .map(|cell| latex_escape(cell))
        .collect::<Vec<_>>()
        .join(" & ")
}

fn sanitize_macro_segment(segment: &str) -> String {
    let mut sanitized = String::new();
    let mut previous_underscore = false;
    for ch in segment.chars().flat_map(char::to_lowercase) {
        let next = if ch.is_ascii_alphanumeric() || ch == '_' {
            ch
        } else {
            '_'
        };
        if next == '_' && previous_underscore {
            continue;
        }
        previous_underscore = next == '_';
        sanitized.push(next);
    }
    let sanitized = sanitized.trim_matches('_').to_string();
    if sanitized.is_empty() {
        "value".to_string()
    } else {
        sanitized
    }
}

fn latex_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\textbackslash{}"),
            '{' => escaped.push_str("\\{"),
            '}' => escaped.push_str("\\}"),
            '$' => escaped.push_str("\\$"),
            '&' => escaped.push_str("\\&"),
            '%' => escaped.push_str("\\%"),
            '#' => escaped.push_str("\\#"),
            '_' => escaped.push_str("\\_"),
            '^' => escaped.push_str("\\textasciicircum{}"),
            '~' => escaped.push_str("\\textasciitilde{}"),
            '\n' => escaped.push(' '),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn into_render_error(req: &RenderRequest) -> impl FnOnce(std::fmt::Error) -> DocpackError + '_ {
    move |_| DocpackError::Render {
        backend: req.backend,
        artifact: req.artifact,
        detail: "failed to format LaTeX output".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::LatexBackend;
    use crate::backend::{ArtifactKind, Backend, BackendKind, RenderRequest};
    use crate::core::{Document, Origin, SourceFormat, SourceMeta, TopLevelShape, Value};

    #[test]
    fn renders_latex_expl3_module() {
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
            backend: BackendKind::Latex,
            artifact: ArtifactKind::DataModule,
            style: "latex-expl3".to_string(),
            root_name: "data".to_string(),
        };
        let rendered = LatexBackend.render(&doc, &req).unwrap();
        assert!(rendered.body.contains("\\prop_new:N \\g_docpack_data_prop"));
        assert!(rendered.body.contains("{active} {true}"));
    }
}
