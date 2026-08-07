# Grok

| | |
|--|--|
| Adapter id | `grok` |
| Binary | `grok` |
| Multi-turn | `--resume <sessionId>` |
| Example | `examples/harnesses/grok.rhai` |

## Launch

```rust
let s = launch("grok", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
```

## ACP

Long-lived agent stdio:

```rust
let s = launch("grok", #{ yolo: true, acp: true, timeout_ms: 180_000 });
```

Requires product authentication for the Grok Build CLI.
