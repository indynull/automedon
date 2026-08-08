# Examples

| Directory | Role |
|-----------|------|
| [`harnesses/`](harnesses/) | **Product scripts** -- real CLIs + auth (what you run) |
| [`mock/`](mock/) | Internal only -- unit tests / continuous integration |
| [`DRIVER_SURFACE.md`](DRIVER_SURFACE.md) | Contributor method map (includes mock for suite coverage) |

## Product scenarios

| Scenario | Path |
|----------|------|
| Pi workspace multi-turn | `harnesses/pi_workspace.rhai` |
| Grok coding multi-turn | `harnesses/grok_workspace.rhai` |
| Claude multi-turn markers | `harnesses/claude.rhai` |
| Pi tools + hooks | `harnesses/pi_tools.rhai` |
| Grok multi-turn markers | `harnesses/grok.rhai` |

```bash
# After product login works alone
automedon run examples/harnesses/pi_workspace.rhai --print
automedon run examples/harnesses/grok_workspace.rhai --print
automedon run examples/harnesses/claude.rhai --print
```

Full table: [harnesses/README.md](harnesses/README.md).
Checklist: [docs/qa-playbook.md](../docs/qa-playbook.md).
