# Product harness examples

Multi-turn smokes for every specialized adapter. Each script header lists **binary**, **auth**, **stream flags**, and **multi-turn** mechanism.

```bash
medon run examples/harnesses/<name>.rhai --print
```

| Script | Adapter | Proves |
|--------|---------|--------|
| `claude.rhai` | `claude` | multi-turn resume |
| `codex.rhai` | `codex` | exec json + resume |
| `copilot.rhai` | `copilot` | JSONL + resume id |
| `cursor.rhai` | `cursor` | stream-json + resume |
| `gemini.rhai` | `gemini` | stream-json + resume |
| `grok.rhai` | `grok` | streaming-json + resume |
| `grok_smoke.rhai` | `grok` | one-turn |
| `grok_acp.rhai` | `grok` | ACP multi-turn |
| `grok_coding.rhai` | `grok` | multi-turn coding task |
| `opencode.rhai` | `opencode` | json + session |
| `pi.rhai` | `pi` | multi-turn |
| `pi_tools.rhai` | `pi` | tools + hooks |
| `aider.rhai` | `aider` | history multi-turn |

Markers: `AUTOMEDON_T1` then `AUTOMEDON_T2`. Transcript must contain both after turn 2.

**Before Automedon:** confirm the product CLI alone accepts a one-shot prompt with your login.

Vendor workflow: [docs/qa-playbook.md](../../docs/qa-playbook.md).
