use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_docpack")
}

fn temp_dir(name: &str) -> PathBuf {
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

fn write_file(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

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
