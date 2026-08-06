# OpenAI Codex CLI

| | |
|--|--|
| Adapter id | `codex` |
| Binary | `codex` |
| Multi-turn | `exec resume <id>` + `--json` |
| Live example | `examples/live/codex.rhai` |
| Live status | Needs OpenAI / Codex auth |

## Launch

```rhai
let s = launch("codex", #{ yolo: true, timeout_ms: 180_000 });
```

Optional ACP prepare: `acp: true`.

## Live test

```bash
AUTOMEDON_LIVE_CODEX=1 cargo test -p automedon --test live_harness live_codex_launch -- --ignored --nocapture
```
