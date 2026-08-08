# Aider

| | |
|--|--|
| Adapter id | `aider` |
| Binary | `aider` |
| Auth | API keys for the model you configure |
| Stream | Plain text on `--message` (no agent tool stream) |
| Multi-turn | `--chat-history-file` + `--restore-chat-history` |
| Yolo maps to | `--yes-always` (always set for non-interactive) |
| Example | `examples/harnesses/aider.rhai` |

## Launch

```rust
let s = launch("aider", #{
    // model: "provider/model-id",
    no_git: true,
    multi_turn: true,
    timeout_ms: 180_000
});
```

Optional: `chat_history_file`, `openai_api_base`, `xai_api_key` / `openai_api_key` env via extras.

**Tools:** not available on this path (`stream_tools` is false). Use text markers only.

## Daily smoke

```bash
aider --help
automedon run examples/harnesses/aider.rhai --print   # set model first if required
```
