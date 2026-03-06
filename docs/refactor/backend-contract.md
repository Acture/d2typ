# Backend Contract and Reference Fixtures

Status: Accepted for Phase 0 design

## Summary

Backends render normalized `Document` values into concrete document assets. The backend layer owns all output syntax and escaping rules. The input layer is forbidden from generating backend-specific strings.

## Public backend interface

```rust
pub enum BackendKind {
    Typst,
    Latex,
}

pub enum ArtifactKind {
    DataModule,
    TableFragment,
}

pub struct RenderRequest {
    pub backend: BackendKind,
    pub artifact: ArtifactKind,
    pub style: String,
    pub root_name: String,
}

pub struct RenderedArtifact {
    pub body: String,
}

pub trait Backend {
    fn kind(&self) -> BackendKind;
    fn render(&self, doc: &Document, req: &RenderRequest) -> Result<RenderedArtifact, DocpackError>;
}
```

Dispatcher rules:

- style validation happens before `render`
- unsupported `(backend, artifact, style)` combinations return `DocpackError::Render`
- `table-fragment` rendering validates `doc.meta.top_level_shape` before backend-specific work begins

## Supported styles

| Backend | Artifact | Style ID | Default |
| --- | --- | --- | --- |
| Typst | `data-module` | `typst-official` | yes |
| Typst | `table-fragment` | `typst-table` | yes |
| LaTeX | `data-module` | `latex-expl3` | yes |
| LaTeX | `data-module` | `latex-classic-macro` | no |
| LaTeX | `table-fragment` | `latex-booktabs-longtable` | yes |
| LaTeX | `table-fragment` | `latex-plain-tabular` | no |

## Backend rules

### Shared rules

- `root_name` is already sanitized by CLI/manifest resolution.
- `Object` keys are rendered in stable lexical order from `BTreeMap`.
- all outputs end with a trailing newline
- nested values are allowed in `data-module`
- `table-fragment` accepts only scalar cells; any nested cell value returns `DocpackError::Render`

### Typst data module: `typst-official`

Output contract:

- emit one binding only
- binding form is `#let <root_name> = <value>`
- strings are double-quoted
- object keys are always double-quoted
- `Null` renders as `none`
- `Bool`, `Integer`, and `Float` render as literal values
- `List` renders as a Typst sequence using parentheses
- `Object` renders as a Typst dictionary using parentheses and `key: value` pairs

Example shape:

```typ
#let data = ("active": true, "age": 30, "name": "Alice")
```

### Typst table fragment: `typst-table`

Preconditions:

- `doc.meta.top_level_shape` must be `TabularRecords` or `TabularMatrix`

Output contract:

- emit a single `#table(...)` expression
- `columns` equals tabular width
- header rows are emitted only when `tabular_columns` is present
- header order always follows `meta.tabular_columns`
- record-shaped data uses `meta.tabular_columns` to project row values in source column order
- `Null` renders as an empty cell
- scalar cell values render as text cells

Example shape:

```typ
#table(
  columns: 2,
  table.header[name][age],
  [Alice], [30],
  [Bob], [25],
)
```

### LaTeX data module: `latex-expl3`

Representation choice:

- data modules are flattened into one `prop` per source
- keys use slash-separated paths
- list lengths are stored at `__len__`
- scalar values are stored as escaped text tokens

Output contract:

- wrap the module in `\ExplSyntaxOn` / `\ExplSyntaxOff`
- create exactly one prop named `\g_docpack_<root_name>_prop`
- populate scalar leaf paths only
- do not generate nested props or seqs in the first implementation

Example shape:

```tex
\ExplSyntaxOn
\prop_new:N \g_docpack_data_prop
\prop_gput:Nnn \g_docpack_data_prop {active} {true}
\prop_gput:Nnn \g_docpack_data_prop {age} {30}
\prop_gput:Nnn \g_docpack_data_prop {name} {Alice}
\ExplSyntaxOff
```

### LaTeX data module: `latex-classic-macro`

Representation choice:

- emit one macro per scalar leaf path
- path segments are sanitized and joined with double underscores
- list lengths are stored in `__len`

Output contract:

- no package dependencies
- every leaf path emits exactly one `\def`

Example shape:

```tex
\def\docpack@data__active{true}
\def\docpack@data__age{30}
\def\docpack@data__name{Alice}
```

### LaTeX table fragment: `latex-booktabs-longtable`

Preconditions:

- source must be tabular
- every row width must be consistent before rendering

Output contract:

- emit a full `longtable` environment
- column alignment is `l` repeated to table width
- if header exists, emit repeated head sections with `\toprule` / `\midrule`
- if header does not exist, emit only body rows and `\bottomrule`
- cells are escaped as plain text

Example shape:

```tex
\begin{longtable}{ll}
\toprule
name & age \\
\midrule
\endfirsthead
\toprule
name & age \\
\midrule
\endhead
Alice & 30 \\
Bob & 25 \\
\bottomrule
\end{longtable}
```

### LaTeX table fragment: `latex-plain-tabular`

Output contract:

- emit a full `tabular` environment
- column alignment is `l` repeated to table width
- if header exists, emit one header row followed by `\hline`
- if header does not exist, emit body rows only

Example shape:

```tex
\begin{tabular}{ll}
name & age \\
\hline
Alice & 30 \\
Bob & 25 \\
\end{tabular}
```

## Escaping rules

### Typst

- escape backslash and double quote inside strings
- preserve newlines in strings with escaped newline syntax
- table-fragment cells are rendered as textual content blocks; literal `]` must be escaped

### LaTeX

- escape `\`, `{`, `}`, `$`, `&`, `%`, `#`, `_`, `^`, `~`
- `Null` in data modules becomes `none`
- `Null` in table fragments becomes an empty cell

## Reference fixtures

These fixtures are the snapshot baseline for later implementation. Each fixture defines:

- normalized IR
- default Typst `data-module` output
- default Typst `table-fragment` result
- default LaTeX output (`latex-expl3` for data modules, `latex-booktabs-longtable` for tables)

### Fixture 1: JSON object

Input:

```json
{"name":"Alice","age":30,"active":true}
```

Normalized IR:

```text
root = {"active": true, "age": 30, "name": "Alice"}
meta = { format: Json, top_level_shape: Object, tabular_columns: None, header_present: None }
```

Typst data module:

```typ
#let data = ("active": true, "age": 30, "name": "Alice")
```

Typst table fragment:

```text
error: artifact table-fragment requires tabular source metadata
```

LaTeX data module:

```tex
\ExplSyntaxOn
\prop_new:N \g_docpack_data_prop
\prop_gput:Nnn \g_docpack_data_prop {active} {true}
\prop_gput:Nnn \g_docpack_data_prop {age} {30}
\prop_gput:Nnn \g_docpack_data_prop {name} {Alice}
\ExplSyntaxOff
```

LaTeX table fragment:

```text
error: artifact table-fragment requires tabular source metadata
```

### Fixture 2: YAML list

Input:

```yaml
- alpha
- beta
- gamma
```

Normalized IR:

```text
root = ["alpha", "beta", "gamma"]
meta = { format: Yaml, top_level_shape: List, tabular_columns: None, header_present: None }
```

Typst data module:

```typ
#let data = ("alpha", "beta", "gamma")
```

Typst table fragment:

```text
error: artifact table-fragment requires tabular source metadata
```

LaTeX data module:

```tex
\ExplSyntaxOn
\prop_new:N \g_docpack_data_prop
\prop_gput:Nnn \g_docpack_data_prop {1} {alpha}
\prop_gput:Nnn \g_docpack_data_prop {2} {beta}
\prop_gput:Nnn \g_docpack_data_prop {3} {gamma}
\prop_gput:Nnn \g_docpack_data_prop {__len__} {3}
\ExplSyntaxOff
```

LaTeX table fragment:

```text
error: artifact table-fragment requires tabular source metadata
```

### Fixture 3: TOML datetime

Input:

```toml
title = "Report"
when = 2025-01-01T12:00:00Z
```

Normalized IR:

```text
root = {"title": "Report", "when": "2025-01-01T12:00:00Z"}
meta = { format: Toml, top_level_shape: Object, tabular_columns: None, header_present: None }
```

Typst data module:

```typ
#let data = ("title": "Report", "when": "2025-01-01T12:00:00Z")
```

Typst table fragment:

```text
error: artifact table-fragment requires tabular source metadata
```

LaTeX data module:

```tex
\ExplSyntaxOn
\prop_new:N \g_docpack_data_prop
\prop_gput:Nnn \g_docpack_data_prop {title} {Report}
\prop_gput:Nnn \g_docpack_data_prop {when} {2025-01-01T12:00:00Z}
\ExplSyntaxOff
```

LaTeX table fragment:

```text
error: artifact table-fragment requires tabular source metadata
```

### Fixture 4: CSV with header

Input:

```csv
name,age
Alice,30
Bob,25
```

Normalized IR:

```text
root = [
  {"age": 30, "name": "Alice"},
  {"age": 25, "name": "Bob"}
]
meta = {
  format: Csv,
  top_level_shape: TabularRecords,
  tabular_columns: ["name", "age"],
  header_present: Some(true)
}
```

Typst data module:

```typ
#let data = (("age": 30, "name": "Alice"), ("age": 25, "name": "Bob"))
```

Typst table fragment:

```typ
#table(
  columns: 2,
  table.header[name][age],
  [Alice], [30],
  [Bob], [25],
)
```

LaTeX data module:

```tex
\ExplSyntaxOn
\prop_new:N \g_docpack_data_prop
\prop_gput:Nnn \g_docpack_data_prop {1/name} {Alice}
\prop_gput:Nnn \g_docpack_data_prop {1/age} {30}
\prop_gput:Nnn \g_docpack_data_prop {2/name} {Bob}
\prop_gput:Nnn \g_docpack_data_prop {2/age} {25}
\prop_gput:Nnn \g_docpack_data_prop {__len__} {2}
\ExplSyntaxOff
```

LaTeX table fragment:

```tex
\begin{longtable}{ll}
\toprule
name & age \\
\midrule
\endfirsthead
\toprule
name & age \\
\midrule
\endhead
Alice & 30 \\
Bob & 25 \\
\bottomrule
\end{longtable}
```

### Fixture 5: CSV without header

Input:

```csv
Alice,30
Bob,25
```

Normalized IR:

```text
root = [("Alice", 30), ("Bob", 25)]
meta = {
  format: Csv,
  top_level_shape: TabularMatrix,
  tabular_columns: None,
  header_present: Some(false)
}
```

Typst data module:

```typ
#let data = (("Alice", 30), ("Bob", 25))
```

Typst table fragment:

```typ
#table(
  columns: 2,
  [Alice], [30],
  [Bob], [25],
)
```

LaTeX data module:

```tex
\ExplSyntaxOn
\prop_new:N \g_docpack_data_prop
\prop_gput:Nnn \g_docpack_data_prop {1/1} {Alice}
\prop_gput:Nnn \g_docpack_data_prop {1/2} {30}
\prop_gput:Nnn \g_docpack_data_prop {2/1} {Bob}
\prop_gput:Nnn \g_docpack_data_prop {2/2} {25}
\prop_gput:Nnn \g_docpack_data_prop {__len__} {2}
\ExplSyntaxOff
```

LaTeX table fragment:

```tex
\begin{longtable}{ll}
Alice & 30 \\
Bob & 25 \\
\bottomrule
\end{longtable}
```

### Fixture 6: XLSX sheet with header

Input:

```text
sheet = "Sales"
rows = [
  ["name", "region"],
  ["Alice", "East"],
  ["Bob", "West"]
]
```

Normalized IR:

```text
root = [
  {"name": "Alice", "region": "East"},
  {"name": "Bob", "region": "West"}
]
meta = {
  format: Xlsx,
  top_level_shape: TabularRecords,
  tabular_columns: ["name", "region"],
  header_present: Some(true)
}
```

Typst data module:

```typ
#let data = (("name": "Alice", "region": "East"), ("name": "Bob", "region": "West"))
```

Typst table fragment:

```typ
#table(
  columns: 2,
  table.header[name][region],
  [Alice], [East],
  [Bob], [West],
)
```

LaTeX data module:

```tex
\ExplSyntaxOn
\prop_new:N \g_docpack_data_prop
\prop_gput:Nnn \g_docpack_data_prop {1/name} {Alice}
\prop_gput:Nnn \g_docpack_data_prop {1/region} {East}
\prop_gput:Nnn \g_docpack_data_prop {2/name} {Bob}
\prop_gput:Nnn \g_docpack_data_prop {2/region} {West}
\prop_gput:Nnn \g_docpack_data_prop {__len__} {2}
\ExplSyntaxOff
```

LaTeX table fragment:

```tex
\begin{longtable}{ll}
\toprule
name & region \\
\midrule
\endfirsthead
\toprule
name & region \\
\midrule
\endhead
Alice & East \\
Bob & West \\
\bottomrule
\end{longtable}
```

### Fixture 7: Nested object

Input:

```json
{"profile":{"name":"Alice","tags":["a","b"]}}
```

Normalized IR:

```text
root = {"profile": {"name": "Alice", "tags": ["a", "b"]}}
meta = { format: Json, top_level_shape: Object, tabular_columns: None, header_present: None }
```

Typst data module:

```typ
#let data = ("profile": ("name": "Alice", "tags": ("a", "b")))
```

Typst table fragment:

```text
error: artifact table-fragment requires tabular source metadata
```

LaTeX data module:

```tex
\ExplSyntaxOn
\prop_new:N \g_docpack_data_prop
\prop_gput:Nnn \g_docpack_data_prop {profile/name} {Alice}
\prop_gput:Nnn \g_docpack_data_prop {profile/tags/1} {a}
\prop_gput:Nnn \g_docpack_data_prop {profile/tags/2} {b}
\prop_gput:Nnn \g_docpack_data_prop {profile/tags/__len__} {2}
\ExplSyntaxOff
```

LaTeX table fragment:

```text
error: artifact table-fragment requires tabular source metadata
```

### Fixture 8: Nested array

Input:

```json
[[1,2],[3,4]]
```

Normalized IR:

```text
root = [(1, 2), (3, 4)]
meta = { format: Json, top_level_shape: List, tabular_columns: None, header_present: None }
```

Typst data module:

```typ
#let data = ((1, 2), (3, 4))
```

Typst table fragment:

```text
error: artifact table-fragment requires tabular source metadata
```

LaTeX data module:

```tex
\ExplSyntaxOn
\prop_new:N \g_docpack_data_prop
\prop_gput:Nnn \g_docpack_data_prop {1/1} {1}
\prop_gput:Nnn \g_docpack_data_prop {1/2} {2}
\prop_gput:Nnn \g_docpack_data_prop {2/1} {3}
\prop_gput:Nnn \g_docpack_data_prop {2/2} {4}
\prop_gput:Nnn \g_docpack_data_prop {__len__} {2}
\ExplSyntaxOff
```

LaTeX table fragment:

```text
error: artifact table-fragment requires tabular source metadata
```
