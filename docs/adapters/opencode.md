# OpenCode

| | |
|--|--|
| Adapter id | `opencode` |
| Binary | `opencode` |
| Multi-turn | `--session` / `--continue` |
| Live example | `examples/live/opencode.rhai` |
| Live status | Needs provider login |

## Launch

```rhai
let s = launch("opencode", #{ yolo: true, timeout_ms: 180_000 });
```

## Live test

```bash
AUTOMEDON_LIVE_OPENCODE=1 cargo test -p automedon --test live_harness live_opencode_launch -- --ignored --nocapture
```
