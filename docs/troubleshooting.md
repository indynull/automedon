# Troubleshooting

| Symptom | What to check |
|---------|----------------|
| Binary not found | Install the product CLI; or set `bin` / `binary` in launch options |
| Auth / login errors from the child | Log in with that product’s own CLI or set its env/API keys |
| Expect timeout | Raise `timeout_ms`; confirm stream format with `RUST_LOG=automedon=debug` |
| Multi-turn lost context | Confirm adapter multi-turn flags and that `session_id` appears after turn 1 |
| Capability error on approve/plan | That harness does not implement interactive mid-flight control |
| Script parse error | Check Rhai syntax; see [rhai.md](rhai.md) |

Offline smoke:

```bash
medon run examples/mock/smoke.rhai --print
```
