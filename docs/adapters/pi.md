# Pi

| | |
|--|--|
| Adapter id | `pi` |
| Binary | `pi` |
| Auth | Pi provider credentials for your account |
| Stream | `-p` + `--mode json` |
| Multi-turn | `--session-id` / `--continue` |
| Yolo maps to | `--approve` |
| Example | `examples/harnesses/pi.rhai` |

## Launch

```rust
let s = launch("pi", #{
    yolo: true,
    multi_turn: true,
    timeout_ms: 180_000
    // provider: "...",
    // model: "...",
});
```

| Extra | Meaning |
|-------|---------|
| `provider` | Pi provider id |
| `model` | Model id |
| `tools` / `exclude_tools` | Tool allow/deny lists |
| `extension` / `extensions` | Extension paths |

Tool lifecycle maps to `ToolCall` + hooks (`PreToolUse` / `PostToolUse`). Tools smoke: `examples/harnesses/pi_tools.rhai`.

## Daily smoke

```bash
pi -p "say hi only" --mode json
automedon run examples/harnesses/pi.rhai --print
automedon run examples/harnesses/pi_tools.rhai --print
```
