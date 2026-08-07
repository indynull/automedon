# GitHub Copilot CLI

| | |
|--|--|
| Adapter id | `copilot` |
| Binary | `copilot` |
| Multi-turn | `--resume` when SessionInfo is parsed from the Resume footer |
| Example | `examples/harnesses/copilot.rhai` |

## Launch

```rust
let s = launch("copilot", #{ yolo: true, timeout_ms: 180_000 });
```

Optional ACP prepare: `acp: true`. Requires Copilot CLI login.
