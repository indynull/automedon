# First script

Prefer a **product** harness. Examples under `examples/` use Grok or Pi. For offline-only learning of the DSL, use `examples/mock/`.

## Multi-turn continuity (Grok)

Needs `grok` on `PATH` and grok auth.

```rust
// examples/multi_turn.rhai (simplified)

let s = launch("grok", #{
    yolo: true,
    multi_turn: true,
    timeout_ms: 180_000
});

s.prompt("Reply with exactly: AUTOMEDON_T1 and nothing else");
s.expect(timeout_ms(text("AUTOMEDON_T1"), 120_000));
s.await_turn();

s.prompt("Reply with exactly: AUTOMEDON_T2 and nothing else");
s.expect(timeout_ms(text("AUTOMEDON_T2"), 120_000));
s.await_turn();

s.close();
```

Run:

```bash
medon run examples/multi_turn.rhai --print
```

## Pattern

| Step | Purpose |
|------|---------|
| `launch(name, opts)` | Start a session for one adapter |
| `prompt(...)` | User turn (increments turn; may spawn a process) |
| `expect` / `wait` | Block until a stream condition matches |
| `await_turn()` | Drain until the current turn ends |
| `close()` | Tear down the session |

## Waits on tools and hooks (Pi)

Needs `pi` on `PATH` and xAI credentials for Pi.

```bash
medon run examples/wait_hooks.rhai --print
```

```rust
let s = launch("pi", #{
    yolo: true,
    provider: "xai",
    model: "grok-4.5",
    timeout_ms: 180_000
});
s.prompt("Run a shell tool once: echo hi. End with DONE.");
s.wait(timeout_ms(wait_hook_started("PreToolUse"), 120_000));
s.wait(timeout_ms(wait_tool_any(), 120_000));
s.wait(timeout_ms(wait_text("DONE"), 120_000));
s.close();
```

## Offline mock

```bash
medon run examples/mock/multi_turn.rhai --print
medon run examples/mock/wait_hooks.rhai --print
```

## Next

- [Concepts](concepts.md) — events, turn end vs session end, capabilities  
- [Rhai scripting](rhai.md) — full surface  
- [Live harnesses](live.md) — every product adapter under `examples/live/`  
