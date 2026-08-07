# GitHub Copilot CLI

| | |
|--|--|
| Adapter id | `copilot` |
| Binary | `copilot` |
| Multi-turn | `--resume=<id>` / `--continue`; session id from JSONL `result.sessionId` |
| Example | `examples/harnesses/copilot.rhai` |

## Launch

```rust
let s = launch("copilot", #{ yolo: true, timeout_ms: 180_000 });
```

By default Automedon uses `--output-format json` (JSONL) so text deltas, toolRequests, and the final `result.sessionId` are structured. `yolo` maps to `--allow-all`.

Optional ACP: `acp: true` → `copilot --acp`. Requires Copilot CLI login.
