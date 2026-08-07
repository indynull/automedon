# Rhai scripting

Scripts are ordinary Rhai files (`.rhai`). Entry points: `medon run` and `medon eval`.

Handbook code fences use the **`rust`** language tag for highlighting: mdBook’s highlight.js has no Rhai grammar, and the [Rhai book](https://rhai.rs/book/about/related.html) recommends Rust (or JavaScript when you need string interpolation).

## Launch

```rust
let s = launch("grok", #{
    yolo: true,
    model: "optional-model-id",
    multi_turn: true,
    timeout_ms: 120_000
    // product extras as needed: provider, acp, chat_history_file, …
});
```

| Key | Meaning |
|-----|---------|
| `yolo` | Map to product allow-all / skip-permission flags when supported |
| `model` | Model id string |
| `timeout_ms` | Default wait/expect timeout |
| `cwd` | Working directory for the child |
| `bin` | Override binary path |
| `scenario` | Mock only |
| extras | Adapter-specific (see [adapters](adapters/index.md)) |

## Session methods

| Call | Role |
|------|------|
| `s.prompt(text)` | Start a user turn |
| `s.expect(pred)` | Wait for predicate |
| `s.wait(w)` | Same family as expect, wait constructors |
| `s.await_turn()` | Drain until turn ends |
| `s.close()` | End session |
| `s.run(text)` | prompt + await_turn + result text |
| `s.approve()` / `s.deny()` | Interactive permission (needs capability) |
| `s.approve_plan()` / `s.reject_plan()` | Plan control (needs capability) |
| `s.text()` / `s.session_id()` / `s.harness()` | Inspect state |

## Predicates and waits

Common constructors (see product and mock examples):

```rust
text("marker")
tool("bash")
tool_any()
permission()
turn_complete()
wait_hook_started("PreToolUse")
wait_hook_finished("PostToolUse")
wait_tool_any()
wait_text("done")
timeout_ms(text("x"), 60_000)
timeout_ms(wait_text("x"), 60_000)
```

## Examples

| Path | Role |
|------|------|
| `examples/mock/*` | Offline mock (CI + learning) |
| `examples/harnesses/*` | Product adapters |

See [live.md](live.md) and `examples/README.md`.
