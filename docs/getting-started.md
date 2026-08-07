# Getting started

## Install

From a git checkout:

```bash
git clone https://github.com/indynull/automedon.git
cd automedon
cargo build -p automedon-cli --release
cargo install --path crates/automedon-cli   # puts `medon` on PATH
```

Needs a recent Rust toolchain (`rust-version` in the workspace `Cargo.toml`).

Check the binary:

```bash
medon --help
medon adapters
```

## Run offline first

The **mock** harness is in-process. No product CLI and no API keys.

```bash
medon run examples/mock/smoke.rhai --print
medon run examples/mock/multi_turn.rhai --print
medon shot mock "hello" --scenario echo
```

Without installing:

```bash
cargo run -p automedon-cli -- run examples/mock/smoke.rhai --print
```

## Drive a real product CLI

Install that product’s CLI, complete its normal login, then:

```bash
medon run examples/harnesses/grok.rhai --print
```

Catalog and layout: [Examples](examples.md). Per-product flags: [Adapters](adapters/index.md).

## Next

Open [Write a script](first-script.md) to walk through multi-turn. Use [Command line](cli.md) for `run` / `eval` / `shot`.
