# Claude Code

| | |
|--|--|
| Adapter id | `claude` |
| Binary | `claude` |
| Auth | Claude Code / Anthropic login (`claude` must work alone first) |
| Stream | `-p` + `--output-format stream-json` + `--verbose` + `--include-hook-events` |
| Multi-turn | `--resume <id>` / `--continue` |
| Yolo maps to | `--dangerously-skip-permissions` |
| Examples | `examples/harnesses/claude.rhai`, `claude_workspace.rhai` |

## Launch

```rust
let s = launch("claude", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
```

Useful extras: `model`, `max_turns`, `allowed_tools`, `permission_mode`, `settings`, `resume`, `session_id`.

## Daily smoke

```bash
claude -p "hi" --output-format text   # product alone
medon run examples/harnesses/claude.rhai --print
# multi-turn + tools:
medon run examples/harnesses/claude_workspace.rhai --print
```

Markers script: `AUTOMEDON_T1` then `AUTOMEDON_T2`, and a `session_id` after turn 1.
Workspace script: tools + `CLAUDE_WS_*` markers; stable session id across turns.

Live cargo tests (ignored by default):

```bash
AUTOMEDON_LIVE_CLAUDE=1 cargo test -p automedon --test live_harness live_claude -- --ignored
```
