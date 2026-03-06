# docpack

[![crates.io](https://img.shields.io/crates/v/docpack.svg)](https://crates.io/crates/docpack)
[![homebrew](https://img.shields.io/badge/homebrew-acture/tools-blue)](https://github.com/Acture/homebrew-tools)
[![Release](https://github.com/acture/docpack/actions/workflows/release.yml/badge.svg)](https://github.com/acture/docpack/actions/workflows/release.yml)
[![License: AGPL v3](https://img.shields.io/badge/license-AGPL--3.0-blue)](LICENSE)
[![Typst](https://img.shields.io/badge/output-Typst-brightgreen)](https://typst.app)
[![LaTeX](https://img.shields.io/badge/output-LaTeX-005F87)](https://www.latex-project.org/)

> Freeze structured data into document-native modules.

`docpack` turns external data sources into checked-in document assets.

Feed it `CSV`, `JSON`, `YAML`, `TOML`, or `XLSX`. Get back generated Typst or LaTeX code that you can commit, diff, review, and ship with the rest of your document sources.

It is built for the cases where runtime data loading is the wrong tradeoff:

- reproducible reports
- compliance or approval workflows
- offline or single-file document bundles
- generated artifacts that need stable diffs

## Why docpack

- `Manifest-first`: use `docpack build` for repeatable project outputs.
- `CLI-friendly`: use `docpack emit` for one-shot pipes and previews.
- `Backend-aware`: Typst and LaTeX outputs share one normalized data model.
- `Explicit failures`: format, inference, sheet, and shape errors are surfaced with concrete diagnostics.
- `Reviewable output`: generated artifacts are plain text modules, not opaque caches.

## Capability Matrix

| Area | Support |
| --- | --- |
| Inputs | `csv`, `json`, `yaml`, `toml`, `xlsx` |
| Backends | `typst`, `latex` |
| Artifacts | `data-module`, `table-fragment` |
| Typst styles | `typst-official`, `typst-table` |
| LaTeX styles | `latex-expl3`, `latex-classic-macro`, `latex-booktabs-longtable`, `latex-plain-tabular` |
| Workflow modes | `build`, `emit`, `inspect`, `init` |

## Quickstart

### Install

```bash
cargo install docpack
```

Or via Homebrew:

```bash
brew tap acture/tools
brew install docpack
```

### Emit to stdout

`emit` is the fast path. If you do not pass `--output`, generated code is written to `stdout`.

```bash
docpack emit data/profile.json --backend typst
```

Example output:

```typ
#let profile = ("name": "Alice", "role": "Engineer")
```

### Emit to a file

```bash
docpack emit data/table.csv \
  --output build/table.tex \
  --artifact table-fragment
```

### Pipe from stdin

Stdin requires an explicit format, and stdout requires an explicit backend.

```bash
cat data/input.yaml | docpack emit - --format yaml --backend typst
```

### Bootstrap a manifest

```bash
docpack init
```

### Build all declared outputs

```bash
docpack build
```

## Manifest-First Workflow

Minimal `docpack.toml`:

```toml
[project]
name = "quarterly-report"
output_dir = "generated"

[[sources]]
id = "sales"
path = "data/sales.csv"
format = "csv"

[[outputs]]
id = "sales_typst"
source = "sales"
path = "sales.typ"
backend = "typst"
artifact = "data-module"
style = "typst-official"
root_name = "sales"

[[outputs]]
id = "sales_tex"
source = "sales"
path = "sales.tex"
backend = "latex"
artifact = "table-fragment"
style = "latex-booktabs-longtable"
```

Then:

```bash
docpack build
```

## Command Surface

```bash
docpack build [manifest-path]
docpack emit <input> [--output <path>] [--format <format>] [--backend <backend>]
docpack inspect <input-or-manifest> [--as <source|manifest>] [...]
docpack init [path] [--force]
```

### What each command is for

- `build`: resolve a manifest and write all outputs in order.
- `emit`: convert one source into one artifact.
- `inspect`: show normalized shape, metadata, and resolved render defaults.
- `init`: generate a starter manifest.

## Output Styles

| Backend | Artifact | Default | Alternatives |
| --- | --- | --- | --- |
| Typst | `data-module` | `typst-official` | none |
| Typst | `table-fragment` | `typst-table` | none |
| LaTeX | `data-module` | `latex-expl3` | `latex-classic-macro` |
| LaTeX | `table-fragment` | `latex-booktabs-longtable` | `latex-plain-tabular` |

## How It Works

`docpack` is intentionally small at the center:

```text
source bytes
  -> input normalization
  -> Value + SourceMeta
  -> backend render request
  -> Typst / LaTeX artifact
```

Current library layers:

- `core`: normalized value tree and source metadata
- `input`: format adapters and normalization
- `backend`: Typst and LaTeX rendering
- `manifest`: project resolution, inference, and build planning
- `error`: structured failure model

## Examples

### Inspect before generating

```bash
docpack inspect data/sales.csv --output build/sales.typ
```

This prints:

- source format
- normalized top-level shape
- tabular metadata
- inferred backend / artifact / style / root name

### Force a specific sheet

```bash
docpack emit workbook.xlsx \
  --sheet Sales \
  --backend typst \
  --artifact table-fragment
```

### Generate classic LaTeX macros

```bash
docpack emit data/profile.json \
  --backend latex \
  --style latex-classic-macro
```

## Status

What is already in place:

- manifest-driven build flow
- one-shot emit and inspect flow
- Typst data modules and table fragments
- LaTeX `expl3`, classic macro, `longtable`, and plain `tabular` outputs
- reference fixture coverage across `json`, `yaml`, `toml`, `csv`, and `xlsx`
- real `pdflatex` smoke coverage for classic macro output
- structured diagnostics for format inference, tabular shape mismatches, and missing sheets

What is still open:

- benchmark fixtures for large `csv` / `xlsx` sources
- memory profiling for larger manifest builds

## Development

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Design Notes

`docpack` is not trying to become a general-purpose ETL framework or a plugin platform.

The core idea is narrower and more useful:

- normalize external structured data once
- render it into document-native code
- keep the generated artifact reviewable and reproducible

## License

`docpack` is licensed under `AGPL-3.0-only`.
