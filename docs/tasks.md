# docpack Implementation Tasks

This file tracks the implementation work after the accepted refactor design in `docs/refactor/`.

## Completed foundation

1. [x] Replace legacy `parser/render` flow with `lib + CLI`
2. [x] Introduce unified `Value` / `Document` / `SourceMeta` core model
3. [x] Add explicit `DocpackError` and remove panic-based parser control flow
4. [x] Normalize JSON / YAML / TOML / CSV / XLSX through `input::parse_source`
5. [x] Implement Typst and LaTeX backend dispatch
6. [x] Replace the one-shot CLI with `build`, `emit`, `inspect`, and `init`

## Remaining backend hardening

1. [ ] Add snapshot fixtures for all reference cases in `docs/refactor/backend-contract.md`
2. [ ] Tighten Typst table escaping for edge-case content blocks
3. [ ] Add XLSX fixture coverage for sheet selection and missing-sheet errors
4. [ ] Verify LaTeX classic macro output against a real TeX toolchain
5. [ ] Decide whether classic macro naming should stay on `\csname` or move to a documented direct macro scheme

## Manifest and CLI follow-up

1. [ ] Add richer manifest validation for format-specific options
2. [ ] Add `inspect --as manifest` integration coverage
3. [ ] Improve user-facing diagnostics for inference failures
4. [ ] Add end-to-end tests for stdout/stderr failure paths

## Quality and polish

1. [ ] Add rustdoc comments to public library entry points
2. [ ] Add benchmark fixtures for large CSV/XLSX inputs
3. [ ] Revisit memory behavior once manifest builds start handling larger source sets
