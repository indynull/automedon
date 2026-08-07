# Troubleshooting

## Installation

| Symptom | What to try |
|---------|-------------|
| `medon: command not found` | `cargo install --path crates/automedon-cli` and ensure `~/.cargo/bin` is on `PATH` |
| Rust version error | Upgrade rustc to the workspace `rust-version` (currently 1.85+) |

## Scripts

| Symptom | What to try |
|---------|-------------|
| `script not found` | Path is relative to your current working directory |
| Expect / wait timeout | Raise `timeout_ms`; confirm the product actually emits that marker; use `RUST_LOG=automedon=debug` |
| `assert_contains` failed | Print `s.text()` / use `--print` on `medon run` |
| Capability error on approve/plan | That adapter does not implement interactive mid-flight control |

## Product CLIs

| Symptom | What to try |
|---------|-------------|
| Binary not found | Install the product CLI; or set `bin` / `binary` in launch options |
| Auth errors from the child | Use that product’s own login / env keys; Automedon does not log you in |
| Multi-turn lost context | Check `s.session_id()` after turn 1; confirm multi-turn flags and resume parse |
| Hang with no output | Child may be waiting on stdin; product one-shot paths should use null stdin (adapter responsibility) |
| Session ends after one turn | Adapter must not map per-turn results to session `Done` — file a bug if a product adapter does |

## Offline sanity

```bash
medon run examples/mock/smoke.rhai --print
medon adapters
```

If mock works and a product harness fails, the problem is almost always the product CLI, auth, or model configuration — not Rhai syntax.
