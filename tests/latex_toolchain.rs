mod common;

use std::path::Path;
use std::process::{Command, Stdio};

use common::{binary, temp_dir, write_file};

#[test]
fn latex_classic_macro_output_compiles_with_pdflatex() {
    if !has_pdflatex() {
        eprintln!("skipping pdflatex smoke test because pdflatex is unavailable");
        return;
    }

    let dir = temp_dir("latex-classic-toolchain");
    let input = dir.join("data.json");
    let output = dir.join("data.tex");
    let main = dir.join("main.tex");

    write_file(&input, r#"{"profile":{"name":"Alice","tags":["a","b"]}}"#);

    let emit = Command::new(binary())
        .args([
            "emit",
            input.to_str().unwrap(),
            "--backend",
            "latex",
            "--style",
            "latex-classic-macro",
            "--output",
            output.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        emit.status.success(),
        "emit failed: {}",
        String::from_utf8_lossy(&emit.stderr)
    );

    write_file(
        &main,
        r#"\documentclass{article}
\input{data.tex}
\begin{document}
Name: \csname docpack@data__profile__name\endcsname
Tag A: \csname docpack@data__profile__tags__1\endcsname
Tag B: \csname docpack@data__profile__tags__2\endcsname
\end{document}
"#,
    );

    let compile = Command::new("pdflatex")
        .current_dir(&dir)
        .args(["-interaction=nonstopmode", "-halt-on-error", "main.tex"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(compile.success(), "pdflatex compilation failed");
    assert!(Path::new(&dir.join("main.pdf")).exists());
}

fn has_pdflatex() -> bool {
    Command::new("pdflatex")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}
