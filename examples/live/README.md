# Live harness examples

One multi-turn smoke per **product** adapter. These need the real CLI on `PATH` and vendor login/credentials.

Primary live scripts also live at repo root (`examples/smoke.rhai`, `multi_turn.rhai`, `wait_hooks.rhai`, `grok_hello.rhai`). Offline mock only: `examples/mock/`.

```bash
medon run examples/live/<name>.rhai --print
```

| Script | Adapter | Notes |
|--------|---------|-------|
| `grok.rhai` | `grok` | Headless streaming-json + resume |
| `grok_acp.rhai` | `grok` | `extra.acp` long-lived agent stdio |
| `pi.rhai` | `pi` | xAI provider path (`provider=xai`, model `grok-4.5`) |
| `aider.rhai` | `aider` | Chat-history multi-turn; model `xai/grok-4.5` |
| `copilot.rhai` | `copilot` | Non-interactive `-p`; resume from footer |
| `claude.rhai` | `claude` | Needs `claude` login / Anthropic |
| `codex.rhai` | `codex` | Needs OpenAI / Codex auth |
| `opencode.rhai` | `opencode` | Needs provider login |
| `cursor.rhai` | `cursor` | Needs `agent login` or `CURSOR_API_KEY` |
| `gemini.rhai` | `gemini` | May fail on free-tier IneligibleTier |

Each script asks for exact markers `AUTOMEDON_T1` then `AUTOMEDON_T2` so multi-turn continuity is obvious. Failures are usually auth/vendor, not Automedon argv shape.
