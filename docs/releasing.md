# Releasing docpack

`docpack` releases are driven by the tag-triggered GitHub Actions workflow in
[`release.yml`](/Users/acture/repos/d2typ/.github/workflows/release.yml).

## What the workflow does

Pushing a `vX.Y.Z` tag triggers one release pipeline:

1. verify the tag format
2. run `cargo fmt --check`
3. run `cargo clippy --all-targets --all-features -- -D warnings`
4. run `cargo test`
5. publish the crate to crates.io
6. create the GitHub Release from `CHANGELOG.md`
7. upload binary artifacts to that release
8. update the Homebrew tap formula in `Acture/homebrew-tools`

The Homebrew formula is rendered from [`scripts/render_homebrew_formula.py`](/Users/acture/repos/d2typ/scripts/render_homebrew_formula.py) and points at the source tarball for the same tag.

## One-time setup

### crates.io

Pick one of these:

- Recommended: configure crates.io trusted publishing for the GitHub repository and let the workflow use OIDC.
- Fallback: add a `CARGO_REGISTRY_TOKEN` repository secret with publish access.

### Homebrew tap

Add a `HOMEBREW_TAP_TOKEN` repository secret that can push to `Acture/homebrew-tools`.

The token needs:

- access to the tap repository
- permission to write repository contents

## Release checklist

1. Update `Cargo.toml` version.
2. Update `CHANGELOG.md` for the new version.
3. Commit the release prep.
4. Create and push the tag:

```bash
git tag v0.1.4
git push origin v0.1.4
```

## Operational notes

- Release automation is tag-only. Re-runs should use GitHub Actions' built-in rerun support on the tag workflow run.
- The tap update removes the legacy `d2typ.rb` formula and writes `docpack.rb`.
- If the tap repo already matches the newly rendered formula, the workflow exits cleanly without creating a no-op tap commit.
