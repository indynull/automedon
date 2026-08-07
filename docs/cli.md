# Command line

Binary: **`medon`** (crate `automedon-cli`).

```bash
cargo install --path crates/automedon-cli
medon --help
```

Errors print as `medon: …` on stderr with a non-zero exit code.

## `medon adapters`

Operator table: name, default binary, capabilities, multi-turn mechanism, and pointers to examples.

```bash
medon adapters
```

## `medon run`

```bash
medon run path/to/script.rhai
medon run path/to/script.rhai --print   # also print the script return value
```

## `medon eval`

```bash
medon eval 'let s = launch("mock", #{ scenario: "echo" }); s.run("z")'
```

## `medon shot`

One-shot without a script file:

```bash
medon shot mock "hello" --scenario echo
medon shot claude "say hi only" --yolo --timeout-ms 120000
medon shot grok "say hi" --yolo --cwd /path/to/workspace
```

| Flag | Meaning |
|------|---------|
| `--yolo` | Product allow-all / skip-permission flags |
| `--model` | Model id |
| `--cwd` | Child working directory |
| `--timeout-ms` | Default wait/expect timeout |
| `--scenario` | Mock only: `echo`, `multi`, `tools`, `hooks`, `permission`, `plan`, `goal`, `think`, `error` |

## Logging

```bash
RUST_LOG=automedon=debug medon run examples/harnesses/claude.rhai --print
```
