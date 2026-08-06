# CI and releases

All automation lives in this repository under `.github/workflows/`.

## Continuous integration (`ci.yml`)

On push and pull request to `main`:

1. `cargo fmt --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace` (live harness tests stay `#[ignore]`)
4. Line coverage on crate `automedon` with fail-under **96%**
5. `mdbook build` (docs must compile)

No vendor secrets required. Live harnesses are not run in default CI.

## Documentation site (`pages.yml`)

On push to `main` (and workflow_dispatch):

1. Build the mdBook handbook (`mdbook build`)
2. Deploy to **GitHub Pages**

Enable in the GitHub repo: **Settings → Pages → Source: GitHub Actions**.

Site URL (after first deploy): `https://indynull.github.io/automedon/`  
(set `site-url` in `book.toml` if the repository name differs).

Local:

```bash
make book          # output in book/
make book-serve    # mdbook serve --open
```

## Releases (`release.yml`)

On tag push matching `v*` (example: `v0.1.0`):

1. Build release binaries for Linux (x86_64) and macOS (x86_64 and/or aarch64 as configured)
2. Upload `medon` artifacts
3. Create a GitHub Release with those assets

```bash
git tag v0.1.0
git push origin v0.1.0
```

Version field in workspace `Cargo.toml` should match the tag you cut.

## Self-contained checklist

| Piece | Location |
|-------|----------|
| Library + CLI | `crates/automedon`, `crates/automedon-cli` |
| Handbook source | `docs/` + `book.toml` |
| Examples | `examples/` |
| CI | `.github/workflows/ci.yml` |
| Pages | `.github/workflows/pages.yml` |
| Releases | `.github/workflows/release.yml` |
| Status matrix | `MATRIX.md` / [matrix.md](matrix.md) |
