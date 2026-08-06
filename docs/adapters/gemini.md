# Gemini CLI

| | |
|--|--|
| Adapter id | `gemini` |
| Binary | `gemini` (prefers `agy` when present) |
| Multi-turn | `-r` / resume |
| Live example | `examples/live/gemini.rhai` |
| Live status | Free tier often IneligibleTier |

## Launch

```rhai
let s = launch("gemini", #{ yolo: true, timeout_ms: 180_000 });
```

## Live test

```bash
AUTOMEDON_LIVE_GEMINI=1 cargo test -p automedon --test live_harness live_gemini_launch_and_text -- --ignored --nocapture
```
