use std::collections::BTreeMap;
use std::io::Cursor;

use calamine::{Data, Reader, open_workbook_auto_from_rs};

use crate::core::{Document, Origin, SourceMeta, TopLevelShape, Value};
use crate::error::{DocpackError, DocpackResult};
use crate::input::SourceSpec;

pub fn parse(spec: &SourceSpec) -> DocpackResult<Document> {
    let mut workbook =
        open_workbook_auto_from_rs(Cursor::new(spec.bytes.clone())).map_err(|error| {
            DocpackError::Parse {
                format: spec.format,
                origin: spec.origin.clone(),
                detail: error.to_string(),
                path: None,
            }
        })?;
    let sheet_name = spec.sheet.clone().unwrap_or_else(|| "Sheet1".to_string());
    let available = workbook.sheet_names().to_vec();
    if !available.iter().any(|name| name == &sheet_name) {
        return match &spec.origin {
            Origin::File(path) => Err(DocpackError::InvalidSheet {
                path: path.clone(),
                requested: sheet_name,
                available,
            }),
            Origin::Stdin => Err(DocpackError::Parse {
                format: spec.format,
                origin: spec.origin.clone(),
                detail: format!(
                    "sheet '{}' was not found. available sheets: {}",
                    sheet_name,
                    available.join(", ")
                ),
                path: None,
            }),
        };
    }
    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|error| DocpackError::Parse {
            format: spec.format,
            origin: spec.origin.clone(),
            detail: error.to_string(),
            path: None,
        })?;

    if spec.no_header {
        parse_matrix(spec, range.rows())
    } else {
        parse_records(spec, range.rows())
    }
}

fn parse_records<'a>(
    spec: &SourceSpec,
    mut rows: impl Iterator<Item = &'a [Data]>,
) -> DocpackResult<Document> {
    let header_row = rows.next().unwrap_or(&[]);
    let headers = header_row.iter().map(data_to_text).collect::<Vec<_>>();
    let expected = headers.len();
    let mut values = Vec::new();

    for (index, row) in rows.enumerate() {
        if row.len() != expected {
            return Err(DocpackError::InconsistentRowWidth {
                origin: spec.origin.clone(),
                expected,
                actual: row.len(),
                row_index: index + 2,
            });
        }
        let mut record = BTreeMap::new();
        for (header, cell) in headers.iter().zip(row.iter()) {
            record.insert(header.clone(), data_to_value(cell));
        }
        values.push(Value::Object(record));
    }

    Ok(Document {
        source_id: spec.source_id.clone(),
        root: Value::List(values),
        meta: SourceMeta {
            format: spec.format,
            origin: spec.origin.clone(),
            top_level_shape: TopLevelShape::TabularRecords,
            tabular_columns: Some(headers),
            header_present: Some(true),
        },
    })
}

fn parse_matrix<'a>(
    spec: &SourceSpec,
    rows: impl Iterator<Item = &'a [Data]>,
) -> DocpackResult<Document> {
    let mut values = Vec::new();
    let mut expected_width = None;

    for (index, row) in rows.enumerate() {
        let actual = row.len();
        match expected_width {
            Some(expected) if expected != actual => {
                return Err(DocpackError::InconsistentRowWidth {
                    origin: spec.origin.clone(),
                    expected,
                    actual,
                    row_index: index + 1,
                });
            }
            None => expected_width = Some(actual),
            _ => {}
        }
        values.push(Value::List(row.iter().map(data_to_value).collect()));
    }

    Ok(Document {
        source_id: spec.source_id.clone(),
        root: Value::List(values),
        meta: SourceMeta {
            format: spec.format,
            origin: spec.origin.clone(),
            top_level_shape: TopLevelShape::TabularMatrix,
            tabular_columns: None,
            header_present: Some(false),
        },
    })
}

fn data_to_value(cell: &Data) -> Value {
    match cell {
        Data::Empty => Value::Null,
        Data::Bool(value) => Value::Bool(*value),
        Data::Int(value) => Value::Integer(*value),
        Data::Float(value) => Value::Float(*value),
        Data::String(value) => Value::String(value.clone()),
        Data::DateTime(value) => Value::String(value.to_string()),
        Data::DateTimeIso(value) => Value::String(value.to_string()),
        Data::DurationIso(value) => Value::String(value.to_string()),
        Data::Error(value) => Value::String(value.to_string()),
    }
}

fn data_to_text(cell: &Data) -> String {
    match data_to_value(cell) {
        Value::Null => String::new(),
        Value::Bool(value) => value.to_string(),
        Value::Integer(value) => value.to_string(),
        Value::Float(value) => value.to_string(),
        Value::String(value) => value,
        Value::List(_) | Value::Object(_) => {
            unreachable!("XLSX cells never normalize to nested values")
        }
    }
}
