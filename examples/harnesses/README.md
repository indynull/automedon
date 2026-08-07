# Product harness examples

One multi-turn smoke per specialized adapter (plus a few Grok/Pi extras).
Each script needs that product’s CLI on `PATH` and normal product authentication.

```bash
medon run examples/harnesses/<name>.rhai --print
```

| Script | Adapter | Notes |
|--------|---------|--------|
| `grok_smoke.rhai` | `grok` | One-turn text marker |
| `grok.rhai` | `grok` | Multi-turn resume |
| `grok_acp.rhai` | `grok` | `acp: true` → `grok agent stdio` |
| `grok_coding.rhai` | `grok` | Multi-turn coding task under `examples/automedon_demo/` |
| `pi.rhai` | `pi` | Multi-turn; optional `provider` / `model` extras |
| `pi_tools.rhai` | `pi` | Wait on tools + Pre/PostToolUse hooks |
| `aider.rhai` | `aider` | Chat-history multi-turn; set `model` for your backend |
| `copilot.rhai` | `copilot` | Non-interactive path + resume footer |
| `claude.rhai` | `claude` | stream-json + resume/continue |
| `codex.rhai` | `codex` | `exec --json` + resume |
| `opencode.rhai` | `opencode` | `run --format json` + session |
| `cursor.rhai` | `cursor` | agent stream-json + resume |
| `gemini.rhai` | `gemini` | stream-json + resume (`agy` preferred when present) |

Scripts ask for markers `AUTOMEDON_T1` then `AUTOMEDON_T2` so multi-turn continuity is obvious.
