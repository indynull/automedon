# Cursor agent CLI

| | |
|--|--|
| Adapter id | `cursor` |
| Binary | `agent` / `cursor-agent` / `cursor agent` |
| Multi-turn | `--resume` / `--continue` |
| Live example | `examples/live/cursor.rhai` |
| Live status | Needs `agent login` or `CURSOR_API_KEY` |

## Launch

```rhai
let s = launch("cursor", #{ yolo: true, timeout_ms: 180_000 });
```

## Live test

```bash
AUTOMEDON_LIVE_CURSOR=1 cargo test -p automedon --test live_harness live_cursor_launch -- --ignored --nocapture
```
