# GitHub Copilot CLI

| | |
|--|--|
| Adapter id | `copilot` |
| Binary | `copilot` |
| Auth | GitHub Copilot login |
| Stream | `-p` + `--output-format json` (JSONL) |
| Multi-turn | `--resume=<id>` / `--continue`; id from final `result.sessionId` |
| Yolo maps to | `--allow-all` |
| Example | `examples/harnesses/copilot.rhai` |

## Launch

```rust
let s = launch("copilot", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
```

Text comes from `assistant.message_delta` (not re-emitted on the full `assistant.message` frame). Optional ACP: `acp: true`.

## Daily smoke

```bash
copilot -p "say hi only" --output-format json --allow-all
medon run examples/harnesses/copilot.rhai --print
```
