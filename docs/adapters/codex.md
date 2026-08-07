# OpenAI Codex

| | |
|--|--|
| Adapter id | `codex` |
| Binary | `codex` |
| Multi-turn | `codex exec resume <thread_id>` + `--json` |
| Example | `examples/harnesses/codex.rhai` |

## Launch

```rust
let s = launch("codex", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
```

Multi-turn uses `codex exec resume <session_id| --last> --json <prompt>`.

Optional ACP prepare: `acp: true`. Requires OpenAI / Codex authentication.
