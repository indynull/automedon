# Gemini CLI

| | |
|--|--|
| Adapter id | `gemini` |
| Binary | `gemini` (prefers `agy` when present) |
| Auth | Gemini / Google auth for the CLI you install |
| Stream | `-p` + `-o stream-json` |
| Multi-turn | `-r` / resume (`latest` when no id) |
| Yolo maps to | `-y` / `--approval-mode yolo` |
| Example | `examples/harnesses/gemini.rhai` |

## Launch

```rust
let s = launch("gemini", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
```

Optional: `acp: true`, `approval_mode`, `worktree`, `allowed_tools`, `binary`.

## Daily smoke

```bash
gemini -p "hi" -o text -y
medon run examples/harnesses/gemini.rhai --print
```
