# Grok Build

| | |
|--|--|
| Adapter id | `grok` |
| Binary | `grok` |
| Auth | Grok Build login |
| Stream | `-p` + `--output-format streaming-json` |
| Multi-turn | `--resume <id>` / `--continue` when no id |
| Yolo maps to | `--always-approve` |
| Example | `examples/harnesses/grok.rhai` |

## Launch

```rust
let s = launch("grok", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
```

### ACP (long-lived process)

```rust
let s = launch("grok", #{ yolo: true, acp: true, timeout_ms: 180_000 });
```

Also: `examples/harnesses/grok_acp.rhai`, `grok_smoke.rhai`, `grok_coding.rhai`.

## Daily smoke

```bash
grok -p "say hi only" --output-format streaming-json --always-approve
medon run examples/harnesses/grok.rhai --print
```
