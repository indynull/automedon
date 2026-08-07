# Claude Code

| | |
|--|--|
| Adapter id | `claude` |
| Binary | `claude` |
| Auth | Claude Code / Anthropic login (`claude` must work alone first) |
| Stream | `-p` + `--output-format stream-json` + `--verbose` + `--include-hook-events` |
| Multi-turn | `--resume <id>` / `--continue` |
| Yolo maps to | `--dangerously-skip-permissions` |
| Example | `examples/harnesses/claude.rhai` |

## Launch

```rust
let s = launch("claude", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
```

Useful extras: `model`, `max_turns`, `allowed_tools`, `permission_mode`, `settings`, `resume`, `session_id`.

## Daily smoke

```bash
claude -p "hi" --output-format text   # product alone
medon run examples/harnesses/claude.rhai --print
```

Expect `AUTOMEDON_T1` then `AUTOMEDON_T2`, and a `session_id` after turn 1 when the stream emits system init.
