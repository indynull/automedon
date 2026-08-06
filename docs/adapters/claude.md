# Claude Code

| | |
|--|--|
| Adapter id | `claude` |
| Binary | `claude` |
| Multi-turn | `--resume` / `--continue`, stream-json |
| Live example | `examples/live/claude.rhai` |
| Live status | Often blocked until product login |

## Launch

```rhai
let s = launch("claude", #{ yolo: true, timeout_ms: 180_000 });
```

Capability bits stay false until multi-turn is live-proven after auth.

## Live test

```bash
AUTOMEDON_LIVE_CLAUDE=1 cargo test -p automedon --test live_harness live_claude_launch -- --ignored --nocapture
```
