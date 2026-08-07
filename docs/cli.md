# CLI (`medon`)

Binary name: **`medon`**. Package: `automedon-cli`.

```bash
cargo install --path crates/automedon-cli
medon --help
```

## Commands

### `run`

Run a Rhai script file.

```bash
medon run examples/mock/smoke.rhai --print
medon run examples/harnesses/grok.rhai --print
```

### `eval`

Evaluate a one-liner:

```bash
medon eval 'let s = launch("mock", #{ scenario: "echo" }); s.run("z")'
```

### `shot`

One-shot prompt without a script file:

```bash
medon shot mock "hello" --scenario echo
medon shot grok "say hi" --yolo
```

### `adapters`

List registered harnesses and capability flags:

```bash
medon adapters
```

## Exit codes

Non-zero when the script fails, the harness exits non-zero on `shot`, or the script path is missing. Use `--print` only for successful result inspection.

## Logging

Uses `tracing`. Example:

```bash
RUST_LOG=automedon=debug medon run examples/mock/smoke.rhai
```
