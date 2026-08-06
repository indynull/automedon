# Pi

| | |
|--|--|
| Adapter id | `pi` |
| Binary | `pi` |
| Multi-turn | `--session-id` / `--continue` |
| Live example | `examples/live/pi.rhai` |

## xAI path

```rhai
let s = launch("pi", #{
    yolo: true,
    provider: "xai",
    model: "grok-4.5",
    multi_turn: true,
    timeout_ms: 180_000
});
```

Optional: `extension` / `extensions` for Pi extension paths.

## Live test

```bash
AUTOMEDON_LIVE_PI_XAI=1 cargo test -p automedon --test live_harness live_pi_xai_multi_turn -- --ignored --nocapture
AUTOMEDON_LIVE_PI_XAI_TOOLS=1 cargo test -p automedon --test live_harness live_pi_xai_tools_and_hooks -- --ignored --nocapture
```
