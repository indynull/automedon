# Product harness examples

Multi-turn smokes for specialized adapters. Each script header lists **binary**,
**auth**, **stream flags**, and **multi-turn** mechanism. Project is alpha:
scripts can break when product CLIs change.

```bash
medon run examples/harnesses/<name>.rhai --print
```

| Script | Adapter | Checks |
|--------|---------|--------|
| **`pi_workspace.rhai`** | `pi` | multi-turn workspace tools (`PI_WS_*`) |
| **`grok_workspace.rhai`** | `grok` | multi-turn coding (`DONE:fib`, `GROK_WS_OK`) |
| `claude.rhai` | `claude` | multi-turn resume |
| `codex.rhai` | `codex` | exec json + resume |
| `copilot.rhai` | `copilot` | JSONL + resume id |
| `cursor.rhai` | `cursor` | stream-json + resume |
| `gemini.rhai` | `gemini` | stream-json + resume |
| `grok.rhai` | `grok` | streaming-json + resume markers |
| `grok_smoke.rhai` | `grok` | one-turn |
| `grok_acp.rhai` | `grok` | ACP multi-turn |
| `grok_coding.rhai` | `grok` | multi-turn coding (alt markers) |
| `opencode.rhai` | `opencode` | json + session |
| `pi.rhai` | `pi` | multi-turn markers |
| `pi_tools.rhai` | `pi` | tools + hooks |
| `aider.rhai` | `aider` | history multi-turn |

Recommended live checks:

```bash
medon run examples/harnesses/pi_workspace.rhai --print
medon run examples/harnesses/grok_workspace.rhai --print
```

Marker multi-turn scripts use `AUTOMEDON_T1` then `AUTOMEDON_T2` (both must appear after turn 2).

Before Automedon: confirm the product CLI alone accepts a one-shot prompt with your login.

Checklist: [docs/qa-playbook.md](../../docs/qa-playbook.md).
