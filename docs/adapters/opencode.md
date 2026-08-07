# OpenCode

| | |
|--|--|
| Adapter id | `opencode` |
| Binary | `opencode` |
| Multi-turn | `--session` / `--continue` |
| Example | `examples/harnesses/opencode.rhai` |

## Launch

```rust
let s = launch("opencode", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
```

Optional ACP: `acp: true`. Requires OpenCode with a configured provider.
