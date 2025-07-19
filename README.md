# d2typ

A command-line tool for converting structured data to [Typst](https://typst.app/) format.

## Features

- Convert data from various formats to Typst syntax
- Supported input formats:
  - CSV
  - JSON
  - YAML
  - TOML
  - Excel (XLSX)
- Automatic format detection based on file extension
- Read from files or standard input
- Write to files or standard output
- Customizable options for different input formats

## Installation

### From crates.io

```bash
cargo install d2typ
```

### Using Homebrew

```bash
# If formula is submitted to Homebrew
brew install d2typ
```

### From source

```bash
git clone https://github.com/yourusername/d2typ.git
cd d2typ
cargo install --path .
```

## Usage

### Basic Usage

```bash
# Convert a CSV file to Typst format
d2typ input.csv > output.typ

# Convert a JSON file to Typst format
d2typ input.json > output.typ

# Specify output file
d2typ input.yaml -o output.typ

# Read from stdin
cat input.toml | d2typ > output.typ
```

### Command-line Options

```
Usage: d2typ [OPTIONS] [INPUT]

Arguments:
  [INPUT]  Input file (omit for stdin)

Options:
  -o, --output <OUTPUT>    Output file (omit for stdout)
  -f, --format <FORMAT>    Force input format [default: auto] [possible values: auto, csv, json, yaml, toml, xlsx]
      --no-header          For CSV input: treat as no header
      --sheet <SHEET>      For XLSX input: select sheet
  -h, --help               Print help
  -V, --version            Print version
```

### Examples

#### CSV to Typst

Input (data.csv):
```csv
name,age,city
Alice,30,New York
Bob,25,San Francisco
Charlie,35,Seattle
```

Command:
```bash
d2typ data.csv > data.typ
```

Output (data.typ):
```typst
#let data = [
  { name: Alice, age: 30, city: New York },
  { name: Bob, age: 25, city: San Francisco },
  { name: Charlie, age: 35, city: Seattle }
]
```

#### JSON to Typst

Input (data.json):
```json
{
  "users": [
    {"name": "Alice", "age": 30},
    {"name": "Bob", "age": 25}
  ],
  "version": 1.0
}
```

Command:
```bash
d2typ data.json > data.typ
```

Output (data.typ):
```typst
#let data = {
  users: [
    { name: Alice, age: 30 },
    { name: Bob, age: 25 }
  ],
  version: 1.0
}
```

## Integration with Typst

Once you've converted your data to Typst format, you can use it in your Typst documents:

```typst
#import "data.typ"

// Access the data
#for user in data.users [
  - #user.name is #user.age years old
]
```

## Building from Source

### Prerequisites
- Rust toolchain (rustc, cargo)
- Recommended: Rust version 1.75.0 or later

### Building the Project
1. Clone the repository
2. Build the project:
   ```bash
   cargo build
   ```
3. Run the project:
   ```bash
   cargo run -- [arguments]
   ```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0-only) - see the LICENSE file for details.
