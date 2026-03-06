# CLI and Manifest Specification

Status: Accepted for Phase 0 design

## Summary

The new CLI is manifest-first. One-shot conversion still exists, but it is explicitly the secondary flow.

The four public commands are:

- `docpack build [manifest-path]`
- `docpack emit <input>`
- `docpack inspect <input-or-manifest>`
- `docpack init [path]`

## Command model

### `docpack build [manifest-path]`

Purpose:

- load a manifest
- resolve all sources and outputs
- render all outputs in manifest order

Defaults:

- if `manifest-path` is omitted, use `./docpack.toml`

Behavior:

- treat the argument as a manifest path only; `build` does not inspect raw source files
- stop on the first failure
- already written files are not rolled back
- relative output paths are resolved against `project.output_dir` when set, otherwise against the manifest directory

### `docpack emit <input>`

Purpose:

- convert one source into one output without creating a manifest

Rules:

- `<input>` may be a file path or `-` for stdin
- when `<input>` is `-`, `--format` is required
- `--output` is optional
- if `--output` is omitted, output is written to stdout
- when writing to stdout, `--backend` is required because backend inference from extension is unavailable

Supported options:

```text
docpack emit <input> \
  [--output <path>] \
  [--format <csv|json|yaml|toml|xlsx>] \
  [--backend <typst|latex>] \
  [--artifact <data-module|table-fragment>] \
  [--style <style-id>] \
  [--root-name <identifier>] \
  [--no-header] \
  [--sheet <sheet-name>]
```

### `docpack inspect <input-or-manifest>`

Purpose:

- show the normalized source shape, source metadata, and inferred rendering configuration

Ambiguity rule:

- `inspect` accepts either a source file or a manifest, so it must support `--as <source|manifest>`
- without `--as`, detection rules are:
  - if basename is exactly `docpack.toml`, treat as manifest
  - otherwise, if the parsed TOML contains `[[sources]]` or `[[outputs]]`, treat as manifest
  - otherwise treat as source input

Supported options:

```text
docpack inspect <input-or-manifest> \
  [--as <source|manifest>] \
  [--format <csv|json|yaml|toml|xlsx>] \
  [--output <path>] \
  [--backend <typst|latex>] \
  [--artifact <data-module|table-fragment>] \
  [--style <style-id>] \
  [--no-header] \
  [--sheet <sheet-name>]
```

Rules:

- `--output`, `--backend`, `--artifact`, and `--style` are valid only when inspecting a source
- `inspect` never writes files
- output format is human-readable text only in the first implementation

Source inspection output sections:

- `Source`
- `Normalized Shape`
- `Metadata`
- `Resolved Render Defaults`

Manifest inspection output sections:

- `Project`
- `Sources`
- `Outputs`
- `Resolved Build Plan`

### `docpack init [path]`

Purpose:

- create a minimal `docpack.toml`

Rules:

- if `path` is omitted, create `./docpack.toml`
- if `path` is a directory, create `<path>/docpack.toml`
- if `path` ends with `.toml`, create that file exactly
- overwrite requires `--force`

The generated template must include:

- one `project` table
- one commented `sources` example
- one commented `outputs` example

## Manifest schema

Manifest format is TOML.

Top-level structure:

```toml
[project]
name = "quarterly-report"
output_dir = "generated"

[[sources]]
id = "sales"
path = "data/sales.csv"
format = "csv"
no_header = false

[[outputs]]
id = "sales_typst"
source = "sales"
path = "sales.typ"
backend = "typst"
artifact = "data-module"
style = "typst-official"
root_name = "sales"
```

### `[project]`

Fields:

- `name: string` optional
- `output_dir: string` optional

Rules:

- `output_dir` is resolved relative to the manifest directory
- no other project-level defaults are supported in the first implementation

### `[[sources]]`

Fields:

- `id: string` required, unique within the manifest
- `path: string` required
- `format: string` optional
- `no_header: bool` optional, valid only for CSV/XLSX
- `sheet: string` optional, valid only for XLSX

Rules:

- relative `path` values resolve relative to the manifest directory
- `format` defaults to extension-based detection
- `sheet` defaults to `Sheet1` for XLSX when not provided
- `id` is the fallback basis for root name inference

### `[[outputs]]`

Fields:

- `id: string` required, unique within the manifest
- `source: string` required, references `sources.id`
- `path: string` required
- `backend: string` optional
- `artifact: string` optional
- `style: string` optional
- `root_name: string` optional

Rules:

- relative `path` values resolve under `project.output_dir` when present
- `root_name` overrides inferred root naming for this output only
- caption, label, alignment, and template customization are explicitly out of scope in the first redesign

## Inference rules

Inference order is fixed and must be applied identically in `build`, `emit`, and `inspect`.

### Backend inference

Precedence:

1. explicit CLI flag or `outputs.backend`
2. output extension:
   - `.typ` -> `typst`
   - `.tex` -> `latex`
3. style-implied backend
4. fail with `DocpackError::Inference`

Notes:

- stdout output has no extension, so `emit` to stdout requires explicit backend

### Artifact inference

Precedence:

1. explicit CLI flag or `outputs.artifact`
2. style-implied artifact
3. default to `data-module`

Artifact implication by style:

- `typst-table` -> `table-fragment`
- `latex-booktabs-longtable` -> `table-fragment`
- `latex-plain-tabular` -> `table-fragment`
- `typst-official` -> `data-module`
- `latex-expl3` -> `data-module`
- `latex-classic-macro` -> `data-module`

Hard rule:

- `table-fragment` is valid only when `SourceMeta.top_level_shape` is `TabularRecords` or `TabularMatrix`
- there is no implicit downgrade from `table-fragment` to `data-module`

### Style inference

If style is not explicit, defaults are:

- `typst + data-module` -> `typst-official`
- `typst + table-fragment` -> `typst-table`
- `latex + data-module` -> `latex-expl3`
- `latex + table-fragment` -> `latex-booktabs-longtable`

### Root name inference

Precedence:

1. explicit CLI `--root-name`
2. `outputs.root_name`
3. source `id` in manifest mode
4. input file stem in one-shot mode
5. `data`

Sanitization algorithm:

1. lowercase ASCII
2. replace every non-`[a-z0-9_]` character with `_`
3. collapse repeated underscores
4. trim leading and trailing underscores
5. if the result starts with a digit, prefix `data_`
6. if the result is empty, use `data`

Invalid explicit root names are not rejected before sanitization; the sanitized result is used. Only a fully empty post-sanitization value becomes `data`.

## CLI examples

### One-shot Typst data module

```bash
docpack emit data/input.json --output build/input.typ
```

Resolved render request:

- backend: `typst`
- artifact: `data-module`
- style: `typst-official`

### One-shot LaTeX table fragment

```bash
docpack emit data/table.csv \
  --output build/table.tex \
  --artifact table-fragment
```

Resolved render request:

- backend: `latex`
- artifact: `table-fragment`
- style: `latex-booktabs-longtable`

### Stdin source

```bash
cat data.yaml | docpack emit - \
  --format yaml \
  --backend typst \
  --output build/data.typ
```

### Project build

```bash
docpack build
```

## Minimal manifest template for `init`

```toml
[project]
name = "example"
output_dir = "generated"

[[sources]]
id = "data"
path = "data/input.json"

[[outputs]]
id = "data_typst"
source = "data"
path = "data.typ"
```
