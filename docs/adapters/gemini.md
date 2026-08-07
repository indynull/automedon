# Gemini CLI

| | |
|--|--|
| Adapter id | `gemini` |
| Binary | `gemini` (prefers `agy` when present) |
| Multi-turn | `-r` / resume, stream-json |
| Example | `examples/harnesses/gemini.rhai` |

## Launch

```rust
let s = launch("gemini", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
```

Optional ACP prepare: `acp: true`. Requires Gemini / Google authentication for the CLI you install.
