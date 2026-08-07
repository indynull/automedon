# Cursor agent CLI

| | |
|--|--|
| Adapter id | `cursor` |
| Binary | `agent`, `cursor-agent`, or `cursor agent` (first found on PATH) |
| Auth | `agent login` or `CURSOR_API_KEY` |
| Stream | `--print` / `-p` + `--output-format stream-json` (+ partial stream) |
| Multi-turn | `--resume` / `--continue` |
| Yolo maps to | `--force` |
| Example | `examples/harnesses/cursor.rhai` |

## Launch

```rust
let s = launch("cursor", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
```

Override binary with `binary: "cursor-agent"` when needed.

## Daily smoke

```bash
cursor-agent -p "hi" --output-format text   # or: agent -p …
medon run examples/harnesses/cursor.rhai --print
```
