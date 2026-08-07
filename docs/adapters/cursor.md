# Cursor agent CLI

| | |
|--|--|
| Adapter id | `cursor` |
| Binary | `agent`, `cursor-agent`, or `cursor` |
| Multi-turn | `--resume` / `--continue`, stream-json |
| Example | `examples/harnesses/cursor.rhai` |

## Launch

```rust
let s = launch("cursor", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
```

Override binary with `binary: "…"` when needed. Requires Cursor agent authentication.
