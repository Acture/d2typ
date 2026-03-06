# docpack

[![crates.io](https://img.shields.io/crates/v/docpack.svg)](https://crates.io/crates/docpack)
[![homebrew](https://img.shields.io/badge/homebrew-acture/tools-blue)](https://github.com/Acture/homebrew-tools)
[![Release](https://github.com/acture/docpack/actions/workflows/release.yml/badge.svg)](https://github.com/acture/docpack/actions/workflows/release.yml)
[![Typst Compatible](https://img.shields.io/badge/typst-compatible-brightgreen)](https://typst.app)
[![License: AGPL v3](https://img.shields.io/badge/license-AGPL--3.0-blue)](LICENSE)

> **Package structured data into document-ready snapshot modules.**

**docpack** packages external data files into static declarations that can ship alongside your documents. The current output target is Typst, producing `.typ` modules that keep reports, certificates, and other generated documents self-contained, portable, and reproducible.

The name is intentionally broader than Typst: the tool supports Typst and LaTeX render targets on top of one shared normalization pipeline, while keeping room for future document backends without changing the core workflow.

## Features

- Packages common structured formats into document-ready snapshot modules
- Supported input: **CSV**, **JSON**, **YAML**, **TOML**, **Excel (.xlsx)**
- Current outputs: Typst data modules and table fragments, plus LaTeX data modules and table fragments
- Format auto-detection via file extension
- Manifest-first CLI with one-shot `emit` support
- CLI-oriented: stdin/stdout support
- Excel sheet selection, CSV header toggle
- Useful when you want frozen data snapshots instead of runtime file reads

## Installation

### From [crates.io](https://crates.io/crates/docpack)

```bash
cargo install docpack
```

### Via Homebrew (custom tap)

```bash
brew tap acture/tools
brew install docpack
```

### From source

```bash
git clone https://github.com/acture/docpack.git
cd docpack
cargo install --path .
```

## Usage

```bash
# Initialize a manifest
docpack init

# One-shot Typst module
docpack emit data/input.json --output build/input.typ

# One-shot LaTeX table fragment
docpack emit data/table.csv \
  --output build/table.tex \
  --artifact table-fragment

# Read from stdin (stdin requires an explicit format and stdout requires an explicit backend)
cat input.yaml | docpack emit - --format yaml --backend typst

# Inspect a source and resolved render defaults
docpack inspect data/sales.csv --output build/sales.typ

# Build every output declared in a manifest
docpack build
```

### Commands

```bash
docpack build [manifest-path]
docpack emit <input> [--output <path>] [--format <format>] [--backend <backend>]
docpack inspect <input-or-manifest> [--as <source|manifest>] [...]
docpack init [path] [--force]
```

### Minimal manifest

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
```

### Output styles

| Backend | Artifact | Default style | Alternatives |
| --- | --- | --- | --- |
| Typst | `data-module` | `typst-official` | none |
| Typst | `table-fragment` | `typst-table` | none |
| LaTeX | `data-module` | `latex-expl3` | `latex-classic-macro` |
| LaTeX | `table-fragment` | `latex-booktabs-longtable` | `latex-plain-tabular` |
