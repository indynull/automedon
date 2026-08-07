# Aider

| | |
|--|--|
| Adapter id | `aider` |
| Binary | `aider` |
| Multi-turn | `--chat-history-file` + `--restore-chat-history` |
| Example | `examples/harnesses/aider.rhai` |

## Launch

```rust
let s = launch("aider", #{
    model: "your-provider/model-id",
    no_git: true,
    timeout_ms: 180_000
});
```

Optional: `chat_history_file` for a fixed history path. Set `model` (and any env vars that Aider needs for that model).

The non-interactive `--message` path does not expose an agent tool stream; tool waits are not available on this adapter.
