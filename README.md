# docpack

[![crates.io](https://img.shields.io/crates/v/docpack.svg)](https://crates.io/crates/docpack)
[![homebrew](https://img.shields.io/badge/homebrew-acture/tools-blue)](https://github.com/Acture/homebrew-tools)
[![Release](https://github.com/acture/docpack/actions/workflows/release.yml/badge.svg)](https://github.com/acture/docpack/actions/workflows/release.yml)
[![Typst Compatible](https://img.shields.io/badge/typst-compatible-brightgreen)](https://typst.app)
[![License: AGPL v3](https://img.shields.io/badge/license-AGPL--3.0-blue)](LICENSE)

> **Package structured data into document-ready snapshot modules.**

**docpack** packages external data files into static declarations that can ship alongside your documents. The current output target is Typst, producing `.typ` modules that keep reports, certificates, and other generated documents self-contained, portable, and reproducible.

The name is intentionally broader than Typst: the tool is optimized for Typst today, while leaving room for future document backends without changing the core workflow.

## Features

- Packages common structured formats into document-ready snapshot modules
- Supported input: **CSV**, **JSON**, **YAML**, **TOML**, **Excel (.xlsx)**
- Current output: valid Typst `#let` declarations
- Format auto-detection via file extension
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
# Convert CSV
docpack input.csv > data.typ

# Convert JSON
docpack input.json -o data.typ

# Read from stdin (stdin requires an explicit format)
cat input.yaml | docpack --format yaml

# Force input format and output file
docpack data.xlsx --sheet Sheet1 --format xlsx -o out.typ
```

### CLI Options

```bash
Usage: docpack [OPTIONS] [INPUT]

Arguments:
  [INPUT]  Input file (omit for stdin)

Options:
  -o, --output <OUTPUT>  Output file (omit for stdout)
  -f, --format <FORMAT>  Force input format [default: auto] [possible values: auto, csv, json, yaml, toml, xlsx]
      --no-header        For CSV input: treat as no header
      --sheet <SHEET>    For XLSX input: select sheet
  -h, --help             Print help
  -V, --version          Print version
```
