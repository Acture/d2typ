#![allow(dead_code)]

use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

pub fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_docpack")
}

pub fn temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "docpack-test-{name}-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

pub fn write_file(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

pub fn make_xlsx(sheet_name: &str, rows: &[&[&str]]) -> Vec<u8> {
    let cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    zip.start_file("[Content_Types].xml", options).unwrap();
    write!(
        zip,
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">"#,
            r#"<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>"#,
            r#"<Default Extension="xml" ContentType="application/xml"/>"#,
            r#"<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>"#,
            r#"<Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>"#,
            r#"</Types>"#
        )
    )
    .unwrap();

    zip.add_directory("_rels/", options).unwrap();
    zip.start_file("_rels/.rels", options).unwrap();
    write!(
        zip,
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
            r#"<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>"#,
            r#"</Relationships>"#
        )
    )
    .unwrap();

    zip.add_directory("xl/", options).unwrap();
    zip.start_file("xl/workbook.xml", options).unwrap();
    write!(
        zip,
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            r#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" "#,
            r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#,
            r#"<sheets><sheet name="{sheet_name}" sheetId="1" r:id="rId1"/></sheets>"#,
            r#"</workbook>"#
        ),
        sheet_name = xml_escape(sheet_name),
    )
    .unwrap();

    zip.add_directory("xl/_rels/", options).unwrap();
    zip.start_file("xl/_rels/workbook.xml.rels", options)
        .unwrap();
    write!(
        zip,
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
            r#"<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>"#,
            r#"</Relationships>"#
        )
    )
    .unwrap();

    zip.add_directory("xl/worksheets/", options).unwrap();
    zip.start_file("xl/worksheets/sheet1.xml", options).unwrap();
    write!(zip, "{}", worksheet_xml(rows)).unwrap();

    zip.finish().unwrap().into_inner()
}

fn worksheet_xml(rows: &[&[&str]]) -> String {
    let mut xml = String::from(concat!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
        r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">"#,
        r#"<sheetData>"#
    ));
    for (row_index, row) in rows.iter().enumerate() {
        let row_number = row_index + 1;
        xml.push_str(&format!(r#"<row r="{row_number}">"#));
        for (col_index, cell) in row.iter().enumerate() {
            xml.push_str(&format!(
                r#"<c r="{}{row_number}" t="inlineStr"><is><t>{}</t></is></c>"#,
                column_name(col_index),
                xml_escape(cell)
            ));
        }
        xml.push_str("</row>");
    }
    xml.push_str("</sheetData></worksheet>");
    xml
}

fn column_name(index: usize) -> String {
    let letter = char::from_u32(b'A' as u32 + index as u32).unwrap();
    letter.to_string()
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
