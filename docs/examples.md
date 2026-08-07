# Examples

| Path | Purpose |
|------|---------|
| `examples/mock/` | Offline mock — no product CLI |
| `examples/harnesses/` | Real product adapters — CLI + auth |

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

Each multi-turn script:

- Documents binary, auth, stream, and multi-turn mechanism in the header  
- Uses `AUTOMEDON_T1` → `AUTOMEDON_T2`  
- Asserts both markers appear in the final transcript  
- Prints `session_id` after turn 1  

Full table: `examples/harnesses/README.md`. Daily loop: [Testing your harness (QA)](qa-playbook.md).
