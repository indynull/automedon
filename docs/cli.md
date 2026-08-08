# Command line

Binary: **`automedon`** (crate `automedon-cli`).

```bash
cargo install --path crates/automedon-cli
automedon --help
```

Errors print as `automedon: ...` on stderr with a non-zero exit code.

## `automedon adapters`

Operator table: name, default binary, capabilities, multi-turn mechanism, and pointers to examples.

```bash
automedon adapters
```

## `automedon run`

```bash
automedon run path/to/script.rhai
automedon run path/to/script.rhai --print   # also print the script return value
```

## `automedon eval`

```bash
automedon eval 'let s = launch("grok", #{ yolo: true, timeout_ms: 120_000 }); s.run("say hi")'
```

## `automedon shot`

One-shot without a script file:

```bash
automedon shot claude "say hi only" --yolo --timeout-ms 120000
automedon shot grok "say hi" --yolo --cwd /path/to/workspace
```

| Flag | Meaning |
|------|---------|
| `--yolo` | Product allow-all / skip-permission flags |
| `--model` | Model id |
| `--cwd` | Child working directory |
| `--timeout-ms` | Default wait/expect timeout |
| `--scenario` | Internal mock adapter only (not for product runs) |

## Logging

```bash
RUST_LOG=automedon=debug automedon run examples/harnesses/claude.rhai --print
```
