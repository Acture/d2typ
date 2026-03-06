# Phased Migration Plan

Status: Accepted for Phase 0 design

## Summary

Implementation is staged. Each phase must leave the repository in a releasable state, even when old and new code temporarily coexist.

The migration sequence is:

1. Phase 0: design only
2. Phase 1: library spine and error model
3. Phase 2: input normalization rewrite
4. Phase 3: Typst backend rewrite
5. Phase 4: LaTeX backend implementation
6. Phase 5: CLI and manifest rewrite

## Global rules

- No phase may introduce new panic paths.
- New code goes into the target module layout from the RFC, not into legacy parser/render files.
- Compatibility shims are allowed only as temporary adapters with explicit removal notes.
- Snapshot fixtures must be created before renderer rewrites begin.

## Phase 0: Design assets

Deliverables:

- `docs/refactor/architecture-rfc.md`
- `docs/refactor/cli-manifest-spec.md`
- `docs/refactor/backend-contract.md`
- `docs/refactor/migration-plan.md`

Exit criteria:

- the design documents are decision complete
- no production code changes are required in this phase

## Phase 1: Library spine and error model

Primary goal:

- create a stable library boundary without changing end-user behavior yet

Implementation work:

- add `src/lib.rs`
- add `src/error.rs`
- add `src/core/` with `Value`, `Document`, and `SourceMeta`
- add initial `src/input/mod.rs`, `src/backend/mod.rs`, and `src/manifest/mod.rs`
- move top-level result types and shared enums out of `main.rs`
- introduce `DocpackError`
- make the existing `main.rs` call into the new library API

Legacy code status after Phase 1:

- `src/parser/*` still exists
- `src/render.rs` still exists
- `src/cliargs.rs` still exists
- `TypstValue` and `ParsedData` may still exist internally, but only behind adapters

Exit criteria:

- library compiles and is exercised by current CLI
- all public library entry points return `DocpackError`
- no new behavior regressions in existing tests

## Phase 2: Input normalization rewrite

Primary goal:

- make input parsing produce `Document` directly

Implementation work:

- implement `input::parse_source(...)`
- move format detection into `src/input/detect.rs`
- add one adapter per supported format
- normalize TOML datetime to string
- convert YAML non-string keys into structured errors
- enforce row-width checks for CSV/XLSX
- codify CSV scalar coercion rules in one place

Files expected to become legacy-only by the end of this phase:

- `src/parser/mod.rs`
- `src/parser/value.rs`

Temporary compatibility layer:

- old parser entry points may delegate into `input::parse_source(...)` until Phase 3 lands

Exit criteria:

- `Document` is the only value crossing from input to rendering code
- current panic reproduction cases now return errors or normalized values
- new normalization unit tests cover every fixture from the backend contract

## Phase 3: Typst backend rewrite

Primary goal:

- replace the serializer-driven Typst path with explicit backend rendering

Implementation work:

- add `src/backend/request.rs`
- implement `src/backend/typst.rs`
- move root-name handling to request resolution rather than render-time guessing
- implement `data-module` and `table-fragment`
- build snapshot tests from the default Typst outputs in `backend-contract.md`

Files scheduled for removal at the end of this phase:

- `src/render.rs`
- `src/parser/parsed_data/ser.rs`
- `src/parser/parsed_data/mod.rs`

Exit criteria:

- no code path depends on `ParsedData` or the old serializer
- Typst snapshots pass for all reference fixtures
- current CLI still works through the library shim

## Phase 4: LaTeX backend implementation

Primary goal:

- add the second backend without changing the core model

Implementation work:

- implement `src/backend/latex.rs`
- support `latex-expl3`
- support `latex-classic-macro`
- support `latex-booktabs-longtable`
- support `latex-plain-tabular`
- add backend snapshot tests using the reference fixtures

Optional split:

- if Phase 4 is too large, split it into:
  - Phase 4A: `latex-expl3` + `latex-booktabs-longtable`
  - Phase 4B: `latex-classic-macro` + `latex-plain-tabular`

Constraint:

- the backend request and style registry defined in earlier phases must not need redesign if this split happens

Exit criteria:

- both backend/artifact categories work for LaTeX
- unsupported combinations fail cleanly with `DocpackError::Render`
- default LaTeX styles are snapshot-tested

## Phase 5: CLI and manifest rewrite

Primary goal:

- replace the current single-command UX with the new manifest-first command surface

Implementation work:

- replace `CliArgs` with subcommand-based CLI parsing
- implement `build`, `emit`, `inspect`, and `init`
- add `manifest::model`, `manifest::load`, and `manifest::infer`
- add root-name sanitization and inference
- add backend, artifact, and style inference
- add manifest validation and source/output resolution
- add integration tests for manifest builds and one-shot emits

Files scheduled for removal at the end of this phase:

- legacy parser shim functions
- old one-command CLI path in `main.rs`
- any compatibility wrappers kept from Phase 1-3

Exit criteria:

- manifest-first CLI is the only public command surface
- `docpack emit` supports stdin with explicit `--format`
- `docpack inspect` handles both source and manifest inputs using `--as` override
- all integration tests pass

## Test rollout plan

The test suite expands in parallel with the phases:

### Phase 1

- keep existing unit tests running
- add error model smoke tests

### Phase 2

- add `tests/fixtures/inputs/`
- add normalization tests for all eight reference fixtures
- add regression tests for:
  - stdin without explicit format
  - TOML datetime
  - YAML non-string keys
  - XLSX missing sheet
  - inconsistent row width

### Phase 3 and Phase 4

- add `tests/snapshots/typst/`
- add `tests/snapshots/latex/`
- snapshot every reference fixture for default backend outputs
- snapshot one explicit non-default example for:
  - `latex-classic-macro`
  - `latex-plain-tabular`

### Phase 5

- add `tests/integration/cli_emit.rs`
- add `tests/integration/cli_build.rs`
- add `tests/integration/cli_inspect.rs`
- add `tests/integration/cli_init.rs`
- add manifest fixtures under `tests/fixtures/manifests/`

## Removal checklist

These legacy components are expected to disappear by the end of the refactor:

- `TypstValue`
- `ParsedData`
- custom serializer in `src/parser/parsed_data/ser.rs`
- `RenderMode`
- direct `main -> detect_format -> parse_input -> render_to_typst` control flow

These concepts survive in renamed or relocated form:

- input format detection -> `input::detect`
- source parsing -> `input::parse_source`
- root binding name -> `RenderRequest.root_name`
- file-based CLI entry point -> `docpack emit`

## Release gates

Implementation should not merge the whole redesign in one commit. Use release gates:

- Gate 1: library spine merged, old CLI still active
- Gate 2: input normalization merged, no panic regressions
- Gate 3: Typst backend merged, serializer removed
- Gate 4: LaTeX backend merged
- Gate 5: new CLI and manifest flow merged, old CLI removed

Each gate should have a passing `cargo test` run and updated user documentation before release.
