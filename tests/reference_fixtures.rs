mod common;

use docpack::{
    ArtifactKind, BackendKind, DocpackError, Origin, RenderRequest, SourceFormat, SourceSpec,
    parse_source, render_document,
};

use common::make_xlsx;

#[test]
fn json_object_matches_reference_outputs() {
    let doc = parse_text_fixture(
        SourceFormat::Json,
        br#"{"name":"Alice","age":30,"active":true}"#,
    );

    assert_typst_data(
        &doc,
        "#let data = (\"active\": true, \"age\": 30, \"name\": \"Alice\")\n",
    );
    assert_table_error(&doc, BackendKind::Typst);
    assert_latex_data(
        &doc,
        "\\ExplSyntaxOn\n\\prop_new:N \\g_docpack_data_prop\n\\prop_gput:Nnn \\g_docpack_data_prop {active} {true}\n\\prop_gput:Nnn \\g_docpack_data_prop {age} {30}\n\\prop_gput:Nnn \\g_docpack_data_prop {name} {Alice}\n\\ExplSyntaxOff\n",
    );
    assert_table_error(&doc, BackendKind::Latex);
}

#[test]
fn yaml_list_matches_reference_outputs() {
    let doc = parse_text_fixture(SourceFormat::Yaml, b"- alpha\n- beta\n- gamma\n");

    assert_typst_data(&doc, "#let data = (\"alpha\", \"beta\", \"gamma\")\n");
    assert_table_error(&doc, BackendKind::Typst);
    assert_latex_data(
        &doc,
        "\\ExplSyntaxOn\n\\prop_new:N \\g_docpack_data_prop\n\\prop_gput:Nnn \\g_docpack_data_prop {1} {alpha}\n\\prop_gput:Nnn \\g_docpack_data_prop {2} {beta}\n\\prop_gput:Nnn \\g_docpack_data_prop {3} {gamma}\n\\prop_gput:Nnn \\g_docpack_data_prop {__len__} {3}\n\\ExplSyntaxOff\n",
    );
    assert_table_error(&doc, BackendKind::Latex);
}

#[test]
fn toml_datetime_matches_reference_outputs() {
    let doc = parse_text_fixture(
        SourceFormat::Toml,
        b"title = \"Report\"\nwhen = 2025-01-01T12:00:00Z\n",
    );

    assert_typst_data(
        &doc,
        "#let data = (\"title\": \"Report\", \"when\": \"2025-01-01T12:00:00Z\")\n",
    );
    assert_table_error(&doc, BackendKind::Typst);
    assert_latex_data(
        &doc,
        "\\ExplSyntaxOn\n\\prop_new:N \\g_docpack_data_prop\n\\prop_gput:Nnn \\g_docpack_data_prop {title} {Report}\n\\prop_gput:Nnn \\g_docpack_data_prop {when} {2025-01-01T12:00:00Z}\n\\ExplSyntaxOff\n",
    );
    assert_table_error(&doc, BackendKind::Latex);
}

#[test]
fn csv_with_header_matches_reference_outputs() {
    let doc = parse_text_fixture(SourceFormat::Csv, b"name,age\nAlice,30\nBob,25\n");

    assert_typst_data(
        &doc,
        "#let data = ((\"age\": 30, \"name\": \"Alice\"), (\"age\": 25, \"name\": \"Bob\"))\n",
    );
    assert_typst_table(
        &doc,
        "#table(\n  columns: 2,\n  table.header[name][age],\n  [Alice], [30],\n  [Bob], [25],\n)\n",
    );
    assert_latex_data(
        &doc,
        "\\ExplSyntaxOn\n\\prop_new:N \\g_docpack_data_prop\n\\prop_gput:Nnn \\g_docpack_data_prop {1/name} {Alice}\n\\prop_gput:Nnn \\g_docpack_data_prop {1/age} {30}\n\\prop_gput:Nnn \\g_docpack_data_prop {2/name} {Bob}\n\\prop_gput:Nnn \\g_docpack_data_prop {2/age} {25}\n\\prop_gput:Nnn \\g_docpack_data_prop {__len__} {2}\n\\ExplSyntaxOff\n",
    );
    assert_latex_table(
        &doc,
        "\\begin{longtable}{ll}\n\\toprule\nname & age \\\\\n\\midrule\n\\endfirsthead\n\\toprule\nname & age \\\\\n\\midrule\n\\endhead\nAlice & 30 \\\\\nBob & 25 \\\\\n\\bottomrule\n\\end{longtable}\n",
    );
}

#[test]
fn csv_without_header_reference_outputs() {
    let spec = SourceSpec::from_stdin(
        "data",
        SourceFormat::Csv,
        b"Alice,30\nBob,25\n".to_vec(),
        true,
        None,
    );
    let doc = parse_source(&spec).unwrap();

    assert_typst_data(&doc, "#let data = ((\"Alice\", 30), (\"Bob\", 25))\n");
    assert_typst_table(
        &doc,
        "#table(\n  columns: 2,\n  [Alice], [30],\n  [Bob], [25],\n)\n",
    );
    assert_latex_data(
        &doc,
        "\\ExplSyntaxOn\n\\prop_new:N \\g_docpack_data_prop\n\\prop_gput:Nnn \\g_docpack_data_prop {1/1} {Alice}\n\\prop_gput:Nnn \\g_docpack_data_prop {1/2} {30}\n\\prop_gput:Nnn \\g_docpack_data_prop {2/1} {Bob}\n\\prop_gput:Nnn \\g_docpack_data_prop {2/2} {25}\n\\prop_gput:Nnn \\g_docpack_data_prop {__len__} {2}\n\\ExplSyntaxOff\n",
    );
    assert_latex_table(
        &doc,
        "\\begin{longtable}{ll}\nAlice & 30 \\\\\nBob & 25 \\\\\n\\bottomrule\n\\end{longtable}\n",
    );
}

#[test]
fn xlsx_sheet_with_header_matches_reference_outputs() {
    let spec = SourceSpec::from_stdin(
        "data",
        SourceFormat::Xlsx,
        make_xlsx(
            "Sales",
            &[&["name", "region"], &["Alice", "East"], &["Bob", "West"]],
        ),
        false,
        Some("Sales".to_string()),
    );
    let doc = parse_source(&spec).unwrap();

    assert_typst_data(
        &doc,
        "#let data = ((\"name\": \"Alice\", \"region\": \"East\"), (\"name\": \"Bob\", \"region\": \"West\"))\n",
    );
    assert_typst_table(
        &doc,
        "#table(\n  columns: 2,\n  table.header[name][region],\n  [Alice], [East],\n  [Bob], [West],\n)\n",
    );
    assert_latex_data(
        &doc,
        "\\ExplSyntaxOn\n\\prop_new:N \\g_docpack_data_prop\n\\prop_gput:Nnn \\g_docpack_data_prop {1/name} {Alice}\n\\prop_gput:Nnn \\g_docpack_data_prop {1/region} {East}\n\\prop_gput:Nnn \\g_docpack_data_prop {2/name} {Bob}\n\\prop_gput:Nnn \\g_docpack_data_prop {2/region} {West}\n\\prop_gput:Nnn \\g_docpack_data_prop {__len__} {2}\n\\ExplSyntaxOff\n",
    );
    assert_latex_table(
        &doc,
        "\\begin{longtable}{ll}\n\\toprule\nname & region \\\\\n\\midrule\n\\endfirsthead\n\\toprule\nname & region \\\\\n\\midrule\n\\endhead\nAlice & East \\\\\nBob & West \\\\\n\\bottomrule\n\\end{longtable}\n",
    );
}

#[test]
fn nested_object_matches_reference_outputs() {
    let doc = parse_text_fixture(
        SourceFormat::Json,
        br#"{"profile":{"name":"Alice","tags":["a","b"]}}"#,
    );

    assert_typst_data(
        &doc,
        "#let data = (\"profile\": (\"name\": \"Alice\", \"tags\": (\"a\", \"b\")))\n",
    );
    assert_table_error(&doc, BackendKind::Typst);
    assert_latex_data(
        &doc,
        "\\ExplSyntaxOn\n\\prop_new:N \\g_docpack_data_prop\n\\prop_gput:Nnn \\g_docpack_data_prop {profile/name} {Alice}\n\\prop_gput:Nnn \\g_docpack_data_prop {profile/tags/1} {a}\n\\prop_gput:Nnn \\g_docpack_data_prop {profile/tags/2} {b}\n\\prop_gput:Nnn \\g_docpack_data_prop {profile/tags/__len__} {2}\n\\ExplSyntaxOff\n",
    );
    assert_table_error(&doc, BackendKind::Latex);
}

#[test]
fn nested_array_matches_reference_outputs() {
    let doc = parse_text_fixture(SourceFormat::Json, br#"[[1,2],[3,4]]"#);

    assert_typst_data(&doc, "#let data = ((1, 2), (3, 4))\n");
    assert_table_error(&doc, BackendKind::Typst);
    assert_latex_data(
        &doc,
        "\\ExplSyntaxOn\n\\prop_new:N \\g_docpack_data_prop\n\\prop_gput:Nnn \\g_docpack_data_prop {1/1} {1}\n\\prop_gput:Nnn \\g_docpack_data_prop {1/2} {2}\n\\prop_gput:Nnn \\g_docpack_data_prop {2/1} {3}\n\\prop_gput:Nnn \\g_docpack_data_prop {2/2} {4}\n\\prop_gput:Nnn \\g_docpack_data_prop {__len__} {2}\n\\ExplSyntaxOff\n",
    );
    assert_table_error(&doc, BackendKind::Latex);
}

fn parse_text_fixture(format: SourceFormat, bytes: &[u8]) -> docpack::Document {
    let spec = SourceSpec::from_stdin("data", format, bytes.to_vec(), false, None);
    parse_source(&spec).unwrap()
}

fn assert_typst_data(doc: &docpack::Document, expected: &str) {
    let rendered = render_document(doc, &typst_request(ArtifactKind::DataModule)).unwrap();
    assert_eq!(rendered.body, expected);
}

fn assert_typst_table(doc: &docpack::Document, expected: &str) {
    let rendered = render_document(doc, &typst_request(ArtifactKind::TableFragment)).unwrap();
    assert_eq!(rendered.body, expected);
}

fn assert_latex_data(doc: &docpack::Document, expected: &str) {
    let rendered = render_document(doc, &latex_request(ArtifactKind::DataModule)).unwrap();
    assert_eq!(rendered.body, expected);
}

fn assert_latex_table(doc: &docpack::Document, expected: &str) {
    let rendered = render_document(doc, &latex_request(ArtifactKind::TableFragment)).unwrap();
    assert_eq!(rendered.body, expected);
}

fn assert_table_error(doc: &docpack::Document, backend: BackendKind) {
    let request = match backend {
        BackendKind::Typst => typst_request(ArtifactKind::TableFragment),
        BackendKind::Latex => latex_request(ArtifactKind::TableFragment),
    };
    let error = render_document(doc, &request).unwrap_err();
    match error {
        DocpackError::Render { detail, .. } => {
            assert_eq!(
                detail,
                "artifact table-fragment requires tabular source metadata"
            );
        }
        other => panic!("expected render error, got {other}"),
    }
}

fn typst_request(artifact: ArtifactKind) -> RenderRequest {
    RenderRequest {
        backend: BackendKind::Typst,
        artifact,
        style: match artifact {
            ArtifactKind::DataModule => "typst-official".to_string(),
            ArtifactKind::TableFragment => "typst-table".to_string(),
        },
        root_name: "data".to_string(),
    }
}

fn latex_request(artifact: ArtifactKind) -> RenderRequest {
    RenderRequest {
        backend: BackendKind::Latex,
        artifact,
        style: match artifact {
            ArtifactKind::DataModule => "latex-expl3".to_string(),
            ArtifactKind::TableFragment => "latex-booktabs-longtable".to_string(),
        },
        root_name: "data".to_string(),
    }
}

#[test]
fn xlsx_missing_sheet_returns_structured_error() {
    let spec = SourceSpec::from_stdin(
        "data",
        SourceFormat::Xlsx,
        make_xlsx("Sales", &[&["name", "region"]]),
        false,
        Some("Missing".to_string()),
    );
    let error = parse_source(&spec).unwrap_err();
    match error {
        DocpackError::Parse { origin, detail, .. } => {
            assert_eq!(origin, Origin::Stdin);
            assert!(detail.contains("sheet 'Missing' was not found"));
            assert!(detail.contains("Sales"));
        }
        other => panic!("expected parse error, got {other}"),
    }
}
