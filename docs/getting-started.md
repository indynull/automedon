# Getting started

## Requirements

- Rust toolchain matching workspace `rust-version` (**1.85+**)
- For product runs: that product’s CLI on `PATH` and its normal authentication

## Install `medon`

```bash
git clone https://github.com/indynull/automedon.git
cd automedon
cargo install --path crates/automedon-cli
```

`medon` lands in Cargo’s bin directory (usually `~/.cargo/bin`). Put that directory on your `PATH`.

```bash
medon --version
medon --help
medon adapters
```

`medon adapters` prints each product adapter’s default binary, capability flags, and multi-turn mechanism.

### Run without installing

```bash
cargo build -p automedon-cli --release
./target/release/medon run examples/mock/smoke.rhai --print
```

## Offline first (always available)

No product CLI and no API keys:

```bash
medon run examples/mock/smoke.rhai --print
medon run examples/mock/multi_turn.rhai --print
medon run examples/mock/wait_hooks.rhai --print
medon shot mock "hello" --scenario echo
```

## Product harness (after product login)

```bash
# Prove the product works alone, then:
medon run examples/harnesses/claude.rhai --print   # or grok, copilot, …
medon shot claude "say hi only" --yolo --timeout-ms 120000
```

Catalog: [Examples](examples.md). Daily QA loop: [Testing your harness (QA)](qa-playbook.md).

## Develop in-tree

```bash
cargo run -p automedon-cli -- run examples/mock/smoke.rhai --print
make check
```

## Next

- [Testing your harness (QA)](qa-playbook.md) if you own a product CLI  
- [Write a script](first-script.md) for the multi-turn pattern  
- [Command line](cli.md) for `run` / `eval` / `shot` / `adapters`  
