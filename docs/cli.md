# Command line

Binary: **`medon`** (crate `automedon-cli`).

```bash
cargo install --path crates/automedon-cli
medon --help
```

Errors print as `medon: …` on stderr with a non-zero exit code.

## `medon run`

Run a Rhai script file.

```bash
medon run path/to/script.rhai
medon run path/to/script.rhai --print   # also print the script’s return value
```

Fails if the path is missing or the script errors (timeout, failed assert, harness error).

## `medon eval`

Evaluate a short snippet (useful for smoke checks):

```bash
medon eval 'let s = launch("mock", #{ scenario: "echo" }); s.run("z")'
```

## `medon shot`

One-shot prompt without a script file (Rust `run` path).

```bash
medon shot mock "hello" --scenario echo
medon shot grok "say hi only" --yolo
medon shot pi "list files" --yolo --model "your-model" --cwd /path/to/workspace
```

| Flag | Meaning |
|------|---------|
| `--yolo` | Map to the product’s allow-all / skip-permission flags when the adapter supports it |
| `--model` | Model id |
| `--cwd` | Working directory for the child |
| `--scenario` | **Mock only:** `echo`, `multi`, `tools`, `hooks`, `permission`, `plan`, `goal`, `think`, `error` |

On non-zero child exit, `shot` fails.

## `medon adapters`

Prints a capability table for product harnesses, then infrastructure adapters:

```text
NAME       LAUNCH  MULTI  TOOLS  SESSIONS  ACP  YOLO
claude     yes     yes    yes    yes       —    yes
…
```

| Column | Meaning |
|--------|---------|
| LAUNCH | Adapter can prepare a process or in-process session |
| MULTI | Multi-turn continuity implemented |
| TOOLS | Tool events on the stream |
| SESSIONS | Session / resume id handling |
| ACP | ACP / long-lived stdio path available (`extra.acp` where required) |
| YOLO | Preflight auto-approve flags mapped |

`yes` means the **driver implements** the surface. You still need the product CLI and auth for a live run.

## Logging

Uses `tracing`. Example:

```bash
RUST_LOG=automedon=debug medon run examples/mock/smoke.rhai --print
```
