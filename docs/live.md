# Product harness examples

Product scripts live under `examples/harnesses/`. Each needs:

1. Product CLI on `PATH`
2. That product’s authentication
3. The matching script (or your own)

## Catalog

| Script | Adapter | Notes |
|--------|---------|-------|
| `examples/harnesses/grok_smoke.rhai` | `grok` | one-turn |
| `examples/harnesses/grok.rhai` | `grok` | multi-turn |
| `examples/harnesses/grok_acp.rhai` | `grok` | `acp: true` |
| `examples/harnesses/grok_coding.rhai` | `grok` | coding task |
| `examples/harnesses/pi.rhai` | `pi` | multi-turn |
| `examples/harnesses/pi_tools.rhai` | `pi` | tools + hooks |
| `examples/harnesses/aider.rhai` | `aider` | history multi-turn |
| `examples/harnesses/copilot.rhai` | `copilot` | resume footer |
| `examples/harnesses/claude.rhai` | `claude` | stream-json |
| `examples/harnesses/codex.rhai` | `codex` | exec json |
| `examples/harnesses/opencode.rhai` | `opencode` | session continue |
| `examples/harnesses/cursor.rhai` | `cursor` | agent CLI |
| `examples/harnesses/gemini.rhai` | `gemini` | stream-json |

```bash
medon run examples/harnesses/grok.rhai --print
```

Offline mock: `examples/mock/`. Details: [Adapters](adapters/index.md), [matrix.md](matrix.md).
