# Examples

| Directory | Role |
|-----------|------|
| [`mock/`](mock/) | Offline — no product CLI (CI / first-day install) |
| [`harnesses/`](harnesses/) | Product adapters — need that CLI + auth |

```bash
# Offline
medon run examples/mock/smoke.rhai --print
medon run examples/mock/multi_turn.rhai --print

# Product (after product login works alone)
medon run examples/harnesses/claude.rhai --print
```

Daily vendor QA: [docs/qa-playbook.md](../docs/qa-playbook.md).
