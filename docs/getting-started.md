# Getting started

## Requirements

- A recent Rust toolchain (workspace `rust-version` is **1.85**)
- For product harnesses later: that product’s CLI on `PATH` and its normal login

## Install `medon`

From a git checkout of this repository:

```bash
git clone https://github.com/indynull/automedon.git
cd automedon
cargo install --path crates/automedon-cli
```

That installs the **`medon`** binary to Cargo’s bin directory (typically `~/.cargo/bin`). Ensure that directory is on your `PATH`.

```bash
medon --version
medon --help
```

### Run without installing

```bash
cargo build -p automedon-cli --release
./target/release/medon run examples/mock/smoke.rhai --print
```

## First offline runs

The **mock** adapter is in-process. No product CLI and no API keys.

```bash
medon run examples/mock/smoke.rhai --print
medon run examples/mock/multi_turn.rhai --print
medon run examples/mock/wait_hooks.rhai --print
medon shot mock "hello" --scenario echo
```

List adapters and capability flags:

```bash
medon adapters
```

You should see a table of product harnesses (`grok`, `pi`, `claude`, …) plus infrastructure (`mock`, `generic`).

## First product run

1. Install the product CLI (for example Grok Build) and complete its own authentication.  
2. Confirm it works outside Automedon.  
3. Run a harness example:

```bash
medon run examples/harnesses/grok.rhai --print
```

More scripts: [Examples](examples.md). Per-product flags: [Adapters](adapters/index.md).

## Develop in the tree

```bash
cargo run -p automedon-cli -- run examples/mock/smoke.rhai --print
make check   # fmt, clippy, tests, coverage
```

## Next

- [Write a script](first-script.md) — multi-turn pattern  
- [Command line](cli.md) — `run`, `eval`, `shot`, `adapters`  
- [How it works](concepts.md) — events, turns, capabilities  
