# OpenCode

| | |
|--|--|
| Adapter id | `opencode` |
| Binary | `opencode` |
| Auth | OpenCode with a configured provider |
| Stream | `opencode run --format json` |
| Multi-turn | `--session` / `--continue` |
| Yolo maps to | `--auto` |
| Example | `examples/harnesses/opencode.rhai` |

## Launch

```rust
let s = launch("opencode", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
```

Session id appears on `step_start` / frames as `sessionID`. Optional ACP: `acp: true` -> `opencode acp`.

## Daily smoke

```bash
opencode run "say hi" --format json
medon run examples/harnesses/opencode.rhai --print
```
