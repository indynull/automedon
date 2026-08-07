# OpenAI Codex

| | |
|--|--|
| Adapter id | `codex` |
| Binary | `codex` |
| Auth | OpenAI / Codex login |
| Stream | `codex exec --json` (JSONL events) |
| Multi-turn | `codex exec resume <session_id\|--last> --json <prompt>` |
| Yolo maps to | `--dangerously-bypass-approvals-and-sandbox` |
| Example | `examples/harnesses/codex.rhai` |

## Launch

```rust
let s = launch("codex", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
```

Session id comes from `thread.started` (`thread_id`). Optional ACP: `acp: true` (community ACP package via `npx`).

## Daily smoke

```bash
codex exec --json "say hi only"
medon run examples/harnesses/codex.rhai --print
```
