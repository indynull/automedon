# Examples

Public examples are **product harness scripts** under `examples/harnesses/`.
Each needs that product CLI on `PATH` and a working product login.

| Path | Purpose |
|------|---------|
| [`examples/harnesses/`](../examples/harnesses/) | Real product adapters (what you run day to day) |
| [`examples/harnesses/README.md`](../examples/harnesses/README.md) | Full adapter table |

## Recommended first runs

| Job | Script | Needs |
|-----|--------|-------|
| Pi workspace multi-turn | `examples/harnesses/pi_workspace.rhai` | `pi` + auth |
| Grok coding multi-turn | `examples/harnesses/grok_workspace.rhai` | `grok` + auth |
| Claude multi-turn markers | `examples/harnesses/claude.rhai` | `claude` + auth |
| Pi tools + hooks | `examples/harnesses/pi_tools.rhai` | `pi` + auth |
| Grok multi-turn markers | `examples/harnesses/grok.rhai` | `grok` + auth |

```bash
# After the product CLI alone accepts a prompt
automedon run examples/harnesses/pi_workspace.rhai --print
automedon run examples/harnesses/grok_workspace.rhai --print
automedon run examples/harnesses/claude.rhai --print
```

Product scripts document binary, auth, stream, and multi-turn in the file header.
They assert concrete markers (for example `PI_WS_OK`, `GROK_WS_OK`, or `AUTOMEDON_T1` / `T2`)
and print `session_id` after turn 1 when multi-turn.

Checklist: [Smoke checklist](qa-playbook.md).

## Contributor note

A private `mock` adapter and scripts under `examples/mock/` exist for unit tests and
continuous integration only. They are not a substitute for product runs and are not
the public getting-started path.
