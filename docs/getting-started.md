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

`medon adapters` lists product harnesses and which capability bits are currently advertised (live-proven only for product adapters).

## First green runs

### Offline (mock — no product CLI)

```bash
medon run examples/mock/smoke.rhai --print
medon run examples/mock/multi_turn.rhai --print
medon shot mock "hello" --scenario echo
```

### Live product harnesses (CLI + auth)

```bash
medon run examples/smoke.rhai --print          # grok
medon run examples/multi_turn.rhai --print     # grok multi-turn
medon run examples/wait_hooks.rhai --print     # pi + xAI (tools/hooks)
```

## Develop in-tree without installing

```bash
cargo run -p automedon-cli -- run examples/mock/smoke.rhai --print
```

## Project checks

```bash
make check   # fmt, clippy -D warnings, tests, line coverage ≥ 96% on crate automedon
make book    # build this handbook (needs mdbook)
```

## Next

- [First script](first-script.md) — write a multi-turn Rhai script  
- [CLI](cli.md) — `run`, `eval`, `shot`, `adapters`  
- [Live harnesses](live.md) — Grok, Pi, Aider, Copilot, …  
