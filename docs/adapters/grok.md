# Grok

| | |
|--|--|
| Adapter id | `grok` |
| Binary | `grok` |
| Multi-turn | `--resume <sessionId>`; ACP long-lived process |
| Live example | `examples/live/grok.rhai`, `examples/live/grok_acp.rhai` |

## Headless

```rhai
let s = launch("grok", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
```

## ACP

```rhai
let s = launch("grok", #{ yolo: true, acp: true, timeout_ms: 180_000 });
```

## Live test

```bash
AUTOMEDON_LIVE_GROK=1 cargo test -p automedon --test live_harness live_grok_multi_turn -- --ignored --nocapture
AUTOMEDON_LIVE_GROK_ACP=1 cargo test -p automedon --test live_harness live_grok_acp_multi_turn_and_tools -- --ignored --nocapture
```
