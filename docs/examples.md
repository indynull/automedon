# Examples

Two directories only:

| Path | Purpose |
|------|---------|
| [`examples/mock/`](https://github.com/indynull/automedon/tree/main/examples/mock) | Offline mock — no product CLI |
| [`examples/harnesses/`](https://github.com/indynull/automedon/tree/main/examples/harnesses) | Real product adapters — CLI + auth |

## Offline

```bash
medon run examples/mock/smoke.rhai --print
medon run examples/mock/multi_turn.rhai --print
medon run examples/mock/wait_hooks.rhai --print
medon shot mock "hello" --scenario echo
```

## Product harnesses

```bash
medon run examples/harnesses/<name>.rhai --print
```

| Script | Adapter |
|--------|---------|
| `grok_smoke.rhai` | `grok` one-turn |
| `grok.rhai` | `grok` multi-turn |
| `grok_acp.rhai` | `grok` ACP |
| `grok_coding.rhai` | `grok` small coding task |
| `pi.rhai` | `pi` multi-turn |
| `pi_tools.rhai` | `pi` tools + hooks |
| `aider.rhai` | `aider` |
| `copilot.rhai` | `copilot` |
| `claude.rhai` | `claude` |
| `codex.rhai` | `codex` |
| `opencode.rhai` | `opencode` |
| `cursor.rhai` | `cursor` |
| `gemini.rhai` | `gemini` |

Multi-turn smokes use markers `AUTOMEDON_T1` then `AUTOMEDON_T2` so continuity is obvious in the transcript.
