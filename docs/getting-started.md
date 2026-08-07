# Getting started

## Install

### From a git checkout

```bash
git clone https://github.com/indynull/automedon.git
cd automedon
cargo build -p automedon-cli --release
cargo install --path crates/automedon-cli   # installs `medon` on PATH
```

Requires a recent Rust toolchain (see `rust-version` in the workspace `Cargo.toml`).

### Verify

```bash
medon --help
medon adapters
```

`medon adapters` lists harnesses and which capability bits each driver advertises.

## First green runs

### Offline (mock — no product CLI)

```bash
medon run examples/mock/smoke.rhai --print
medon run examples/mock/multi_turn.rhai --print
medon shot mock "hello" --scenario echo
```

### Product harnesses (CLI + auth)

```bash
medon run examples/harnesses/grok.rhai --print
medon run examples/harnesses/pi.rhai --print
# see examples/harnesses/README.md
```

## Develop in-tree without installing

```bash
cargo run -p automedon-cli -- run examples/mock/smoke.rhai --print
```

## Project checks

```bash
make check   # fmt, clippy -D warnings, tests, line coverage on crate automedon
make book    # handbook (needs mdbook)
```

## Next

- [First script](first-script.md)  
- [CLI](cli.md)  
- [Live harnesses](live.md) / [Adapters](adapters/index.md)  
