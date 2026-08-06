# Aider

| | |
|--|--|
| Adapter id | `aider` |
| Binary | `aider` |
| Multi-turn | `--chat-history-file` + `--restore-chat-history` |
| Live example | `examples/live/aider.rhai` |

## Launch

```rhai
let s = launch("aider", #{
    model: "xai/grok-4.5",
    no_git: true,
    timeout_ms: 180_000
});
```

Optional: `chat_history_file` for a fixed history path.

No agent tool stream on the message path (unsupported for tool waits).

## Live test

```bash
AUTOMEDON_LIVE_AIDER_XAI=1 cargo test -p automedon --test live_harness live_aider_xai_multi_turn -- --ignored --nocapture
```
