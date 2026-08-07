# Cursor agent CLI

| | |
|--|--|
| Adapter id | `cursor` |
| Binary | **`cursor-agent`** preferred (bare `agent` collides with Grok Build); fallback `agent` / `cursor agent` |
| Auth | `cursor-agent login` or `CURSOR_API_KEY` |
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
cursor-agent -p "hi" --output-format text --force
medon run examples/harnesses/cursor.rhai --print
```

Always prefer the **`cursor-agent`** name. On machines with Grok Build, bare `agent`
is often Grok's binary, not Cursor.
