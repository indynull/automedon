# Write a script

Scripts are Rhai files. Handbook fences use the `rust` language tag for highlighting (highlight.js has no Rhai grammar; [Rhai’s own book](https://rhai.rs/book/about/related.html) recommends that).

## Offline: multi-turn with mock

```rust
// examples/mock/multi_turn.rhai (simplified)

let s = launch("mock", #{ scenario: "multi", timeout_ms: 10_000 });

s.prompt("alpha");
s.expect(text("T1:alpha"));
s.await_turn();

s.prompt("beta");
s.expect(text("prior=T1:alpha"));  // second turn sees prior context
s.await_turn();

s.close();
```

```bash
medon run examples/mock/multi_turn.rhai --print
```

## Same shape on a product harness

Swap the adapter id and drop mock-only options. Example with Grok (needs `grok` on `PATH` + login):

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

## Session API (what you call)

| Call | Role |
|------|------|
| `launch(name, opts)` | Open a session for one adapter |
| `prompt(...)` | Send a user turn |
| `expect` / `wait` | Block until a stream condition matches |
| `await_turn()` | Drain until the current turn ends |
| `close()` | Tear down the session |

Waits on tools and hooks: offline `examples/mock/wait_hooks.rhai`, product `examples/harnesses/pi_tools.rhai`. Details in [Waiting on the stream](waits.md).

Full surface: [Rhai scripts](rhai.md). Events and multi-turn model: [How it works](concepts.md).
