# First script

Start offline with **mock**, then point `launch` at a product id.

## Offline multi-turn (mock)

```rust
// examples/mock/multi_turn.rhai (simplified)

let s = launch("mock", #{ scenario: "multi", timeout_ms: 10_000 });

s.prompt("alpha");
s.expect(text("T1:alpha"));
s.await_turn();

s.prompt("beta");
s.expect(text("prior=T1:alpha"));
s.await_turn();

s.close();
```

```bash
medon run examples/mock/multi_turn.rhai --print
```

## Product multi-turn (Grok)

Needs `grok` on `PATH` and Grok authentication.

```rust
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

```bash
medon run examples/harnesses/grok.rhai --print
```

## Pattern

| Step | Purpose |
|------|---------|
| `launch(name, opts)` | Start a session for one adapter |
| `prompt(...)` | User turn |
| `expect` / `wait` | Block until a stream condition matches |
| `await_turn()` | Drain until the current turn ends |
| `close()` | Tear down |

## Waits on tools and hooks

Offline: `examples/mock/wait_hooks.rhai`.  
Product (Pi): `examples/harnesses/pi_tools.rhai`.

## Next

- [Concepts](concepts.md)  
- [Rhai scripting](rhai.md)  
- [Adapters](adapters/index.md)  
