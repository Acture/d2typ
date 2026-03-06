mod common;

use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

use common::{binary, make_xlsx, temp_dir, write_file};

#[test]
fn emit_reads_json_from_stdin() {
    let mut child = Command::new(binary())
        .args(["emit", "-", "--format", "json", "--backend", "typst"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(br#"{"name":"Alice","age":30}"#)
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, "#let data = (\"age\": 30, \"name\": \"Alice\")\n");
}

#[test]
fn emit_requires_backend_for_stdout() {
    let dir = temp_dir("emit-stdout-error");
    let input = dir.join("input.json");
    write_file(&input, r#"{"name":"Alice"}"#);

    let output = Command::new(binary())
        .args(["emit", input.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("stdout output requires explicit --backend"));
}

#[test]
fn build_writes_manifest_outputs() {
    let dir = temp_dir("build");
    let manifest = dir.join("docpack.toml");
    let input = dir.join("data.json");
    let output = dir.join("generated").join("data.typ");

    write_file(&input, r#"{"name":"Alice","age":30}"#);
    write_file(
        &manifest,
        r#"[project]
output_dir = "generated"

[[sources]]
id = "people"
path = "data.json"
format = "json"

[[outputs]]
id = "people_typst"
source = "people"
path = "data.typ"
backend = "typst"
artifact = "data-module"
"#,
    );

    let status = Command::new(binary())
        .args(["build", manifest.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success());
    let built = fs::read_to_string(output).unwrap();
    assert_eq!(built, "#let people = (\"age\": 30, \"name\": \"Alice\")\n");
}

#[test]
fn init_creates_manifest_template() {
    let dir = temp_dir("init");
    let target_dir = dir.join("workspace");
    let status = Command::new(binary())
        .args(["init", target_dir.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success());
    let manifest = fs::read_to_string(target_dir.join("docpack.toml")).unwrap();
    assert!(manifest.contains("[project]"));
    assert!(manifest.contains("[[sources]]"));
    assert!(manifest.contains("[[outputs]]"));
}

#[test]
fn inspect_supports_manifest_override() {
    let dir = temp_dir("inspect-manifest");
    let manifest = dir.join("project-config.toml");
    let input = dir.join("data.json");

    write_file(&input, r#"{"name":"Alice"}"#);
    write_file(
        &manifest,
        r#"[project]
name = "demo"

[[sources]]
id = "people"
path = "data.json"
format = "json"

[[outputs]]
id = "people_typst"
source = "people"
path = "people.typ"
backend = "typst"
"#,
    );

    let output = Command::new(binary())
        .args(["inspect", manifest.to_str().unwrap(), "--as", "manifest"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Project"));
    assert!(stdout.contains("Resolved Build Plan"));
    assert!(stdout.contains("backend=typst"));
}

#[test]
fn emit_stdin_requires_explicit_format() {
    let output = Command::new(binary())
        .args(["emit", "-", "--backend", "typst"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("stdin input requires an explicit --format"));
}

#[test]
fn build_rejects_manifest_with_invalid_source_options() {
    let dir = temp_dir("invalid-manifest");
    let manifest = dir.join("docpack.toml");
    let input = dir.join("data.json");

    write_file(&input, r#"{"name":"Alice"}"#);
    write_file(
        &manifest,
        r#"[[sources]]
id = "people"
path = "data.json"
format = "json"
no_header = true

[[outputs]]
id = "people_typst"
source = "people"
path = "people.typ"
"#,
    );

    let output = Command::new(binary())
        .args(["build", manifest.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("no_header"));
    assert!(stderr.contains("only valid for csv or xlsx sources"));
}

#[test]
fn emit_xlsx_reports_missing_sheet() {
    let dir = temp_dir("xlsx-missing-sheet");
    let workbook = dir.join("sales.xlsx");
    fs::write(
        &workbook,
        make_xlsx("Sales", &[&["name", "region"], &["Alice", "East"]]),
    )
    .unwrap();

    let output = Command::new(binary())
        .args([
            "emit",
            workbook.to_str().unwrap(),
            "--backend",
            "typst",
            "--sheet",
            "Missing",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("sheet 'Missing'"));
    assert!(stderr.contains("available sheets: Sales"));
}

#[test]
fn emit_reports_unsupported_output_extension_for_backend_inference() {
    let dir = temp_dir("bad-extension");
    let input = dir.join("input.json");

    write_file(&input, r#"{"name":"Alice"}"#);

    let output = Command::new(binary())
        .args([
            "emit",
            input.to_str().unwrap(),
            "--output",
            dir.join("output.txt").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("extension '.txt' is not mapped to a backend"));
    assert!(stderr.contains("use --backend or write to .typ/.tex"));
}

#[test]
fn emit_reports_non_tabular_table_fragment_request() {
    let dir = temp_dir("non-tabular-fragment");
    let input = dir.join("input.json");

    write_file(&input, r#"{"name":"Alice"}"#);

    let output = Command::new(binary())
        .args([
            "emit",
            input.to_str().unwrap(),
            "--backend",
            "typst",
            "--artifact",
            "table-fragment",
            "--output",
            dir.join("output.typ").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("source shape is object"));
    assert!(stderr.contains("use data-module instead"));
}

#[test]
fn emit_reports_supported_styles_for_invalid_style_choice() {
    let dir = temp_dir("invalid-style");
    let input = dir.join("input.json");

    write_file(&input, r#"{"name":"Alice"}"#);

    let output = Command::new(binary())
        .args([
            "emit",
            input.to_str().unwrap(),
            "--output",
            dir.join("output.typ").to_str().unwrap(),
            "--style",
            "latex-expl3",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("style 'latex-expl3' is not valid for typst data-module"));
    assert!(stderr.contains("supported styles: typst-official"));
}
