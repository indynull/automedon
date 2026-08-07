# Pi

| | |
|--|--|
| Adapter id | `pi` |
| Binary | `pi` |
| Multi-turn | `--session-id` / `--continue` |
| Example | `examples/harnesses/pi.rhai` |

## Launch

```rust
let s = launch("pi", #{
    yolo: true,
    multi_turn: true,
    timeout_ms: 180_000
});
```

Optional extras:

| Key | Meaning |
|-----|---------|
| `provider` | Pi provider id for your account |
| `model` | Model id string |
| `extension` / `extensions` | Pi extension paths |

Tool lifecycle events map to `ToolCall` / hooks (`PreToolUse`, `PostToolUse`). See `examples/harnesses/pi_tools.rhai`.

Requires `pi` on `PATH` and whatever provider credentials Pi expects for your setup.
