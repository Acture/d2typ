# Architecture RFC: docpack Core Rewrite

Status: Accepted for Phase 0 design

## Summary

`docpack` will move from a single binary with parser-specific rendering logic to a `lib + CLI` architecture with:

- a single normalized value tree IR
- a dedicated input layer for format adapters
- a backend layer for Typst and LaTeX rendering
- a manifest layer for project-level orchestration
- a single explicit error model with no `todo!()`, `unimplemented!()`, or panic-based control flow

This RFC is the authoritative design for the Phase 1-5 implementation work.

## Why the current structure must be replaced

The current code has four structural problems:

1. Parsing and rendering are coupled. `src/parser/mod.rs` decides output shape while still reading input.
2. The data model is duplicated. `TypstValue` and `ParsedData` overlap and encode renderer-specific ideas.
3. The renderer boundary is not real. `src/parser/parsed_data/ser.rs` is a Typst-specific serializer hidden inside parser code, and it still contains unfinished `todo!()` branches.
4. Error behavior is unstable. Real inputs such as TOML datetimes still panic instead of returning structured failures.

Because of these issues, adding a second backend or a manifest-driven workflow on top of the current structure would multiply technical debt instead of reducing it.

## Goals

- Make `docpack` a reusable library with a thin CLI wrapper.
- Normalize all supported inputs into one stable IR.
- Preserve enough source metadata for table rendering without promoting table data to a separate IR node kind.
- Make backend rendering completely independent from input parsing.
- Make later Typst and LaTeX implementations follow one shared contract.
- Replace all panic paths with stable typed errors.

## Non-goals

- This RFC does not implement LaTeX or the new CLI yet.
- This RFC does not preserve the old one-command UX as a primary interface.
- This RFC does not introduce network input, plugins, or template engines.
- This RFC does not add a typed datetime or decimal node to the IR.

## Target package layout

The package remains a single crate, but it becomes a library with a binary entry point:

```text
src/
  lib.rs
  main.rs
  error.rs
  core/
    mod.rs
    value.rs
    document.rs
  input/
    mod.rs
    detect.rs
    source.rs
    csv.rs
    json.rs
    yaml.rs
    toml.rs
    xlsx.rs
  backend/
    mod.rs
    request.rs
    typst.rs
    latex.rs
  manifest/
    mod.rs
    model.rs
    infer.rs
    load.rs
```

Rules for these layers:

- `core` contains only normalized model types and pure helpers.
- `input` converts raw sources into `Document`.
- `backend` converts `Document + RenderRequest` into rendered output.
- `manifest` resolves project configuration into concrete build actions.
- `main.rs` and CLI parsing must not contain parsing or rendering logic.

## Canonical core model

### Value

`Value` replaces both `TypstValue` and the payload portion of `ParsedData`.

```rust
pub enum Value {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    Object(BTreeMap<String, Value>),
}
```

Design rules:

- `Object` uses `BTreeMap` so key order is stable across runs and backends.
- No dedicated `Table`, `Tuple`, or `Map` variants survive the rewrite.
- There is no datetime node. TOML/XLSX datetime-like values normalize to `Value::String`.

### Document

`Document` is the unit shared by input and backend code.

```rust
pub struct Document {
    pub source_id: String,
    pub root: Value,
    pub meta: SourceMeta,
}
```

### SourceMeta

`SourceMeta` preserves source-specific shape hints that the IR deliberately does not encode.

```rust
pub struct SourceMeta {
    pub format: SourceFormat,
    pub origin: Origin,
    pub top_level_shape: TopLevelShape,
    pub tabular_columns: Option<Vec<String>>,
    pub header_present: Option<bool>,
}
```

Supporting enums:

```rust
pub enum SourceFormat {
    Csv,
    Json,
    Yaml,
    Toml,
    Xlsx,
}

pub enum Origin {
    File(PathBuf),
    Stdin,
}

pub enum TopLevelShape {
    Scalar,
    List,
    Object,
    TabularRecords,
    TabularMatrix,
}
```

Semantics:

- `TabularRecords` means the root is `List<Object>`, produced only by CSV/XLSX with headers.
- `TabularMatrix` means the root is `List<List<Value>>`, produced only by CSV/XLSX without headers.
- A nested list in JSON or YAML does not become `TabularMatrix`; it remains `TopLevelShape::List`.
- `tabular_columns` is populated only for `TabularRecords`.
- `header_present` is populated only for CSV/XLSX sources.

## Input normalization rules

### Shared rules

- All inputs normalize into `Document`.
- All parser errors must return `DocpackError`; none may panic.
- Path-like diagnostics use JSON Pointer-style locations such as `/profile/name` or `/2/age`.

### JSON

- object -> `Value::Object`
- array -> `Value::List`
- null/bool/number/string -> corresponding scalar
- integer-like numbers become `Integer`; all others become `Float`

### YAML

- mapping -> `Value::Object`
- sequence -> `Value::List`
- null/bool/number/string -> corresponding scalar
- non-string mapping keys are rejected with `UnsupportedKey`

### TOML

- table -> `Value::Object`
- array -> `Value::List`
- string/integer/float/boolean -> corresponding scalar
- datetime -> `Value::String(datetime.to_string())`

### CSV

- with header -> `TopLevelShape::TabularRecords` and root `List<Object>`
- without header -> `TopLevelShape::TabularMatrix` and root `List<List<Value>>`
- scalar coercion is preserved for compatibility:
  - empty string -> `Null`
  - `none` -> `Null`
  - `true` / `false` -> `Bool`
  - integer parse success -> `Integer`
  - float parse success -> `Float`
  - otherwise -> `String`
- all rows must have consistent width
- when header mode is enabled, every data row must match the header width

### XLSX

- sheet selection is resolved before normalization
- with header -> `TopLevelShape::TabularRecords` and root `List<Object>`
- without header -> `TopLevelShape::TabularMatrix` and root `List<List<Value>>`
- all rows must have consistent width
- cell conversions:
  - empty -> `Null`
  - bool/int/float -> corresponding scalar
  - string -> `String`
  - date/datetime/duration/error text -> `String(cell.to_string())`

## Error model

`DocpackError` becomes the only public failure type returned by library entry points.

```rust
pub enum DocpackError {
    Io { origin: Origin, source: std::io::Error },
    DetectFormat { origin: Origin, detail: String },
    Parse { format: SourceFormat, origin: Origin, detail: String, path: Option<String> },
    UnsupportedKey { format: SourceFormat, origin: Origin, path: String, key_repr: String },
    InvalidSheet { path: PathBuf, requested: String, available: Vec<String> },
    InconsistentRowWidth { origin: Origin, expected: usize, actual: usize, row_index: usize },
    InvalidRootName { supplied: String },
    ManifestLoad { path: PathBuf, detail: String },
    ManifestInvalid { path: PathBuf, problems: Vec<String> },
    Inference { detail: String },
    Render { backend: BackendKind, artifact: ArtifactKind, detail: String },
}
```

Mapping from current failure modes:

- `detect_format(...).Err("Cannot detect format from input")` -> `DetectFormat`
- YAML non-string key failure -> `UnsupportedKey`
- TOML datetime panic -> no panic; normalize to string
- missing XLSX sheet -> `InvalidSheet`
- serializer `todo!()` paths -> removed entirely during backend rewrite

## Data flow

All future command flows resolve through one library pipeline:

```text
CLI or manifest
  -> SourceSpec
  -> input::parse_source(...)
  -> Document
  -> manifest::infer_render_request(...) or explicit RenderRequest
  -> backend::render(...)
  -> write output
```

This is the only permitted direction of dependency:

- input depends on core and error
- backend depends on core and error
- manifest depends on core and error
- CLI depends on all three
- core depends on nothing else inside the crate

## Current symbol disposition

| Current symbol | Disposition | Replacement |
| --- | --- | --- |
| `parser::value::TypstValue` | delete | `core::value::Value` |
| `parser::parsed_data::ParsedData` | delete | `core::document::Document` |
| `parser::parsed_data::ser` | delete | `backend::typst` and `backend::latex` |
| `parser::InputFormat` | migrate and rename | `SourceFormat` |
| `parser::detect_format` | migrate | `input::detect::detect_format` |
| `parser::parse_input` | delete | `input::parse_source` |
| `render::render_to_typst` | delete | backend dispatch through `RenderRequest` |
| `render::RenderMode` | delete | style selection inside `RenderRequest` |
| `main.rs` direct pipeline | replace | thin CLI calling library API |
| `cliargs::CliArgs` | keep temporarily | replaced by subcommand-based CLI in Phase 5 |

## Implementation constraints carried forward

- No backend may inspect raw CSV/XLSX state directly; it must consume only `Document`.
- No input adapter may emit backend-specific strings or syntax.
- The library API must compile without `main.rs`.
- `docpack` must stay publishable as a single crate even after the refactor.
