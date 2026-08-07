# Examples

| Directory | Role |
|-----------|------|
| [`mock/`](mock/) | Offline only — in-process mock harness, no product CLI |
| [`harnesses/`](harnesses/) | Product adapters — need the real binary + that product’s login |

```bash
# Offline
medon run examples/mock/smoke.rhai --print
medon run examples/mock/multi_turn.rhai --print
medon shot mock "hello" --scenario echo

# Product (examples)
medon run examples/harnesses/grok.rhai --print
medon run examples/harnesses/pi.rhai --print
```

Generated demo files under `examples/automedon_demo/` are gitignored.
