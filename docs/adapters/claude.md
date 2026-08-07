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

Requires Claude Code login / Anthropic credentials for the product CLI.
