# Live harnesses

Live runs need:

1. Product CLI on `PATH`
2. Vendor authentication / API access
3. A script under `examples/` / `examples/live/` or your own

## Primary examples (repo root)

| Script | Adapter | Notes |
|--------|---------|-------|
| `examples/smoke.rhai` | `grok` | one-turn text marker |
| `examples/multi_turn.rhai` | `grok` | headless multi-turn |
| `examples/wait_hooks.rhai` | `pi` | tools + Pre/PostToolUse waits |
| `examples/grok_hello.rhai` | `grok` | multi-turn coding task |

## Per-adapter catalog (`examples/live/`)

| Script | Adapter | Notes |
|--------|---------|-------|
| `examples/live/grok.rhai` | `grok` | Headless multi-turn |
| `examples/live/grok_acp.rhai` | `grok` | `acp: true` |
| `examples/live/pi.rhai` | `pi` | `provider=xai`, model `grok-4.5` |
| `examples/live/aider.rhai` | `aider` | history multi-turn, `xai/grok-4.5` |
| `examples/live/copilot.rhai` | `copilot` | resume from footer |
| `examples/live/claude.rhai` | `claude` | needs Claude login |
| `examples/live/codex.rhai` | `codex` | needs OpenAI auth |
| `examples/live/opencode.rhai` | `opencode` | needs provider login |
| `examples/live/cursor.rhai` | `cursor` | needs agent login / key |
| `examples/live/gemini.rhai` | `gemini` | may hit IneligibleTier |

```bash
medon run examples/smoke.rhai --print
medon run examples/live/grok.rhai --print
```

Details and launch extras: [Adapters](adapters/index.md).

## Library live tests

Env-gated integration tests (not run in default CI):

```bash
AUTOMEDON_LIVE_GROK=1 cargo test -p automedon --test live_harness live_grok_multi_turn -- --ignored --nocapture
```

Gates include `AUTOMEDON_LIVE_PI_XAI`, `AUTOMEDON_LIVE_AIDER_XAI`, `AUTOMEDON_LIVE_COPILOT`, `AUTOMEDON_LIVE_GROK_ACP`, `AUTOMEDON_LIVE_CLAUDE`, etc.

## Status

What is live-proven on a given machine: [matrix.md](matrix.md). Capability bits stay false until proven.
