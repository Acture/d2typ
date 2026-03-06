use std::collections::BTreeMap;
use std::io::Cursor;

use crate::core::{Document, SourceMeta, TopLevelShape, Value};
use crate::error::{DocpackError, DocpackResult};
use crate::input::{SourceSpec, coerce_text_cell};

pub fn parse(spec: &SourceSpec) -> DocpackResult<Document> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(!spec.no_header)
        .from_reader(Cursor::new(&spec.bytes));

    if spec.no_header {
        parse_matrix(spec, &mut reader)
    } else {
        parse_records(spec, &mut reader)
    }
}

fn parse_records<R: std::io::Read>(
    spec: &SourceSpec,
    reader: &mut csv::Reader<R>,
) -> DocpackResult<Document> {
    let headers = reader
        .headers()
        .map_err(|error| DocpackError::Parse {
            format: spec.format,
            origin: spec.origin.clone(),
            detail: error.to_string(),
            path: None,
        })?
        .iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let expected = headers.len();
    let mut rows = Vec::new();

    for (index, record) in reader.records().enumerate() {
        let record = record.map_err(|error| DocpackError::Parse {
            format: spec.format,
            origin: spec.origin.clone(),
            detail: error.to_string(),
            path: None,
        })?;
        if record.len() != expected {
            return Err(DocpackError::InconsistentRowWidth {
                origin: spec.origin.clone(),
                expected,
                actual: record.len(),
                row_index: index + 2,
            });
        }
        let mut row = BTreeMap::new();
        for (column, cell) in headers.iter().zip(record.iter()) {
            row.insert(column.clone(), coerce_text_cell(cell));
        }
        rows.push(Value::Object(row));
    }

    Ok(Document {
        source_id: spec.source_id.clone(),
        root: Value::List(rows),
        meta: SourceMeta {
            format: spec.format,
            origin: spec.origin.clone(),
            top_level_shape: TopLevelShape::TabularRecords,
            tabular_columns: Some(headers),
            header_present: Some(true),
        },
    })
}

fn parse_matrix<R: std::io::Read>(
    spec: &SourceSpec,
    reader: &mut csv::Reader<R>,
) -> DocpackResult<Document> {
    let mut rows = Vec::new();
    let mut expected_width = None;

    for (index, record) in reader.records().enumerate() {
        let record = record.map_err(|error| DocpackError::Parse {
            format: spec.format,
            origin: spec.origin.clone(),
            detail: error.to_string(),
            path: None,
        })?;
        let actual = record.len();
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
        rows.push(Value::List(record.iter().map(coerce_text_cell).collect()));
    }

    Ok(Document {
        source_id: spec.source_id.clone(),
        root: Value::List(rows),
        meta: SourceMeta {
            format: spec.format,
            origin: spec.origin.clone(),
            top_level_shape: TopLevelShape::TabularMatrix,
            tabular_columns: None,
            header_present: Some(false),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::core::{SourceFormat, TopLevelShape, Value};
    use crate::input::SourceSpec;

    #[test]
    fn parses_csv_with_header_as_records() {
        let spec = SourceSpec::from_stdin(
            "people",
            SourceFormat::Csv,
            b"name,age\nAlice,30\nBob,25\n".to_vec(),
            false,
            None,
        );
        let doc = parse(&spec).unwrap();
        assert_eq!(doc.meta.top_level_shape, TopLevelShape::TabularRecords);
        assert_eq!(
            doc.meta.tabular_columns,
            Some(vec!["name".into(), "age".into()])
        );
        match &doc.root {
            Value::List(rows) => assert_eq!(rows.len(), 2),
            _ => panic!("expected list root"),
        }
    }

    #[test]
    fn parses_csv_without_header_as_matrix() {
        let spec = SourceSpec::from_stdin(
            "people",
            SourceFormat::Csv,
            b"Alice,30\nBob,25\n".to_vec(),
            true,
            None,
        );
        let doc = parse(&spec).unwrap();
        assert_eq!(doc.meta.top_level_shape, TopLevelShape::TabularMatrix);
        match &doc.root {
            Value::List(rows) => assert_eq!(rows.len(), 2),
            _ => panic!("expected list root"),
        }
    }
}
