## [unreleased]

### 🚀 Features

- *(config)* Add git-cliff configuration for changelog generation
- Rename the project and CLI to `docpack`
## [0.1.3] - 2025-08-08

### 🐛 Bug Fixes

- *(parser)* Wrap serialized values in quotes for consistency
- *(parser)* Update serialization to wrap keys and values in quotes
- *(parser)* Switch serialization brackets from square to round
- *(parser)* Update serialization output to use round brackets
- *(ci)* Update Rust toolchain setup to include empty rustflags

### 📚 Documentation

- *(readme)* Enhance README formatting and content

### ⚙️ Miscellaneous Tasks

- Release version 0.1.3
- *(workflow)* Simplify release workflow and improve asset build process
## [0.1.2] - 2025-07-19

### 🐛 Bug Fixes

- *(parser)* Adjust serialization format and handle "none" as null value

### ⚙️ Miscellaneous Tasks

- Release version 0.1.2
## [0.1.1] - 2025-07-19

### 🚀 Features

- Initialize Rust project with basic setup
- Add data parsing and rendering functionality
- *(cli, parser)* Refactor CLI arguments and improve parsing logic
- *(parser, render)* Enhance data handling and rendering functionality
- *(parser)* Modularize `TypstValue` and `ParsedData` structures for enhanced maintainability
- *(parser, render)* Enhance serialization and display for `ParsedData`
- *(parser, render)* Update visibility of modules and improve test rendering
- *(parser)* Implement unit variant serialization and add tests for `PDSerializer`
- *(workflow)* Add GitHub Actions release workflow

### 🐛 Bug Fixes

- *(metadata)* Correct license field in Cargo.toml
- *(cli)* Correct file name extraction using `file_stem`

### 🚜 Refactor

- *(cli, render)* Remove `RenderMode` for simplified argument handling
- *(parser, cli, render)* Remove unused imports and simplify logic
- *(parser, cli)* Switch `Map` to `BTreeMap`, improve `Display` for `TypstValue`, and adjust CLI args parsing
- *(parser)* Remove unused imports and clean up test module
- *(parser)* Reintroduce `use super::*` in test module for improved accessibility
- *(parser)* Reinstate `use super::*` in tests and fix indentation
- *(parser)* Simplify error propagation and improve readability
- *(parser)* Reorder imports for consistency and fix test indentation

### 📚 Documentation

- Add README with usage, installation, and contribution guidelines
- *(tasks)* Add comprehensive improvement task list
- *(readme)* Update installation methods and change license details

### ⚙️ Miscellaneous Tasks

- *(gitignore)* Update to exclude `.junie` directory
- *(Cargo.toml)* Update metadata for project publishing
- Release version 0.1.1
