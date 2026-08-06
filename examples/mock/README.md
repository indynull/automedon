# Offline mock examples

In-process `mock` harness only — no product CLI or credentials. Used by continuous integration and for learning the DSL without a live agent.

```bash
medon run examples/mock/smoke.rhai --print
medon run examples/mock/multi_turn.rhai --print
medon run examples/mock/wait_hooks.rhai --print
medon shot mock "hello" --scenario echo
```

Product examples live in `examples/` and `examples/live/`.
