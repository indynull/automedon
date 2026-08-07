# Claude Code

| | |
|--|--|
| Adapter id | `claude` |
| Binary | `claude` |
| Multi-turn | `--resume` / `--continue`, stream-json |
| Example | `examples/harnesses/claude.rhai` |

## Launch

```rust
let s = launch("claude", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
```

Uses `--output-format stream-json`, `--verbose`, and `--include-hook-events` so tools and hooks appear on the stream.

Requires Claude Code login / Anthropic credentials for the product CLI.
