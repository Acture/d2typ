# d2typ

[![crates.io](https://img.shields.io/crates/v/d2typ.svg)](https://crates.io/crates/d2typ)
[![homebrew](https://img.shields.io/badge/homebrew-acture/tools-blue)](https://github.com/Acture/homebrew-tools)
[![CI](https://github.com/acture/d2typ/actions/workflows/ci.yml/badge.svg)](https://github.com/acture/d2typ/actions/workflows/ci.yml)
[![Typst Compatible](https://img.shields.io/badge/typst-compatible-brightgreen)](https://typst.app)
[![License: AGPL v3](https://img.shields.io/badge/license-AGPL--3.0-blue)](LICENSE)

> **Convert structured data (CSV, JSON, YAML, etc.) into static Typst declarations.**

**d2typ** is a command-line tool that transforms external data files into `.typ` files, making your Typst documents **self-contained**, **portable**, and **reproducible**. Ideal for certificate
generation, static reporting, or when working in constrained rendering environments.

---

## ✨ Features

- Converts common formats to Typst syntax
- Supported input: **CSV**, **JSON**, **YAML**, **TOML**, **Excel (.xlsx)**
- Output: valid Typst `#let` declarations
- Format auto-detection via file extension
- CLI-oriented: stdin/stdout support
- Excel sheet selection, CSV header toggle
- Works with Typst web/cloud by **avoiding `read()` and `csv()`**

---

## 📦 Installation

### From [crates.io](https://crates.io/crates/d2typ)

```bash
cargo install d2typ
```

### Via Homebrew (custom tap)
```bash
brew tap acture/tools
brew install d2typ
```

### From source

```bash
git clone https://github.com/acture/d2typ.git
cd d2typ
cargo install --path .
```

---

## 🚀 Usage

```bash
# Convert CSV
d2typ input.csv > data.typ

# Convert JSON
d2typ input.json -o data.typ

# Read from stdin
cat input.yaml | d2typ

# Force input format & output file
d2typ data.xlsx --sheet Sheet1 --format xlsx -o out.typ
```


### CLI Options
```bash
Usage: d2typ [OPTIONS] [INPUT]

Arguments:
  [INPUT]  Input file (omit for stdin)

Options:
  -o, --output <OUTPUT>      Output file (omit for stdout)
  -f, --format <FORMAT>      Force input format [default: auto]
                             [values: auto, csv, json, yaml, toml, xlsx]
      --no-header            CSV: treat first row as data
      --sheet <SHEET>        XLSX: select sheet
  -h, --help                 Print help
  -V, --version              Print version
```