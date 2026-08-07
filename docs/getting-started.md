# Getting started

Automedon is **alpha**. You need a real product CLI on `PATH` (and its own login)
to drive anything useful. Adapters and stream parsers can still surprise you.

## Requirements

- Rust toolchain matching workspace `rust-version` (**1.85+**)
- A product coding-agent CLI on `PATH` (for example `pi`, `grok`, `claude`) and that product's normal authentication

## Install `medon`

```bash
git clone https://github.com/indynull/automedon.git
cd automedon
cargo install --path crates/automedon-cli
```

`medon` lands in Cargo's bin directory (usually `~/.cargo/bin`). Put that directory on your `PATH`.

```bash
medon --version
medon --help
medon adapters
```

`medon adapters` lists product adapters, default binaries, and multi-turn mechanisms.

### Run without installing

```bash
cargo build -p automedon-cli --release
./target/release/medon run examples/harnesses/pi.rhai --print
```

## First product run

Confirm the product works alone, then run a harness script:

```bash
# Examples -- pick one you already use
which pi && medon run examples/harnesses/pi_workspace.rhai --print
which grok && medon run examples/harnesses/grok_workspace.rhai --print
which claude && medon run examples/harnesses/claude.rhai --print

medon shot claude "say hi only" --yolo --timeout-ms 120000
```

Catalog: [Examples](examples.md). Checklist: [Smoke checklist](qa-playbook.md).

## Develop in-tree

```bash
cargo run -p automedon-cli -- run examples/harnesses/grok.rhai --print
make check
```

`make check` runs unit tests and a private mock adapter used only in the suite.
Operators should not need mock for day-to-day use.

## Next

- [Smoke checklist](qa-playbook.md) for multi-turn against a product CLI
- [Write a script](first-script.md) for the multi-turn pattern
- [Command line](cli.md) for `run` / `eval` / `shot` / `adapters`
