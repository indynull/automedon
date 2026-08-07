# Troubleshooting

## Installation

| Symptom | What to try |
|---------|-------------|
| `medon: command not found` | `cargo install --path crates/automedon-cli`; put `~/.cargo/bin` on `PATH` |
| Rust version error | Upgrade rustc to workspace `rust-version` (1.85+) |

## Scripts

| Symptom | What to try |
|---------|-------------|
| `script not found` | Paths are relative to your current working directory |
| Expect / wait timeout | Raise `timeout_ms` / `--timeout-ms`; confirm the product emits that exact marker |
| `assert_contains` failed | Use `--print`; inspect `s.text()` |
| Capability error on approve/plan | That driver does not implement interactive mid-flight control |

## Product CLIs (vendor QA)

| Symptom | What to try |
|---------|-------------|
| Binary not found | Install the product CLI; or set `bin` / `binary` |
| Auth errors from the child | Complete **product** login; Automedon does not inject credentials |
| Timeout but product works alone | Stream format mismatch — open an issue with `RUST_LOG=automedon=debug` capture |
| Empty `session_id` on turn 2 | Resume frame not seen; product may still work via `--continue` |
| Hang with no output | Child waiting on stdin; product one-shot should use null stdin (adapter) |
| Multi-turn lost context | Print `session_id` after turn 1; confirm multi-turn flags in the adapter page |

## Offline sanity

```bash
medon adapters
medon run examples/mock/smoke.rhai --print
medon run examples/mock/multi_turn.rhai --print
```

If mock works and a product harness fails, debug the product CLI and auth first.

Vendor daily loop: [Testing your harness (QA)](qa-playbook.md).
