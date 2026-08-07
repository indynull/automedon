# OpenAI Codex

| | |
|--|--|
| Adapter id | `codex` |
| Binary | `codex` |
| Auth | OpenAI / Codex login |
| Stream | `codex exec --json` (JSONL events) |
| Multi-turn | `codex exec resume <session_id\|--last> --json <prompt>` |
| Yolo maps to | `--dangerously-bypass-approvals-and-sandbox` |
| Examples | `examples/harnesses/codex.rhai`, `codex_workspace.rhai` |

## Launch

```rust
let s = launch("codex", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
```

Session id comes from `thread.started` (`thread_id`). File writes show up as
`item` type `file_change` and map to tool events named `file_change` (path in
tool input). Shell tools use `command_execution`. Optional ACP: `acp: true`
(community ACP package via `npx`).

## Daily smoke

```bash
codex exec --json "say hi only"
medon run examples/harnesses/codex.rhai --print
# multi-turn + tools:
medon run examples/harnesses/codex_workspace.rhai --print
```

Markers script: `AUTOMEDON_T1` / `AUTOMEDON_T2` and `session_id` from `thread.started`.
Workspace script: tools + `CODEX_WS_*` markers; stable session across resume.

Live cargo tests (ignored by default):

```bash
AUTOMEDON_LIVE_CODEX=1 cargo test -p automedon --test live_harness live_codex -- --ignored
```
