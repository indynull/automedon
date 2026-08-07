# GitHub Copilot CLI

| | |
|--|--|
| Adapter id | `copilot` |
| Binary | `copilot` |
| Multi-turn | `--resume` from stderr Resume footer → SessionInfo |
| Live example | `examples/live/copilot.rhai` |

## Launch

```rust
let s = launch("copilot", #{ yolo: true, timeout_ms: 180_000 });
```

ACP prepare exists (`acp: true`) but is not live-proven here.

## Live test

```bash
AUTOMEDON_LIVE_COPILOT=1 cargo test -p automedon --test live_harness live_copilot_multi_turn -- --ignored --nocapture
```
