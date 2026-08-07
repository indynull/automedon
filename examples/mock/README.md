# Offline mock examples

In-process `mock` harness only — no product CLI or credentials.
Used for continuous integration and learning the DSL offline.

```bash
medon run examples/mock/smoke.rhai --print
medon run examples/mock/multi_turn.rhai --print
medon run examples/mock/wait_hooks.rhai --print
medon shot mock "hello" --scenario echo
```

Product harness scripts: [`../harnesses/`](../harnesses/).
