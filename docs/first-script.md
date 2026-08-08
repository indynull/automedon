# Write a script

Scripts are Rhai files. Handbook fences use the `rust` language tag for highlighting
(highlight.js has no Rhai grammar; [Rhai's own book](https://rhai.rs/book/about/related.html)
recommends that).

## Multi-turn against a product CLI

Needs the product binary on `PATH` and a working login. Example with Grok:

```rust
// Same shape as examples/harnesses/grok.rhai

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
automedon run examples/harnesses/grok.rhai --print
```

Other ready scripts: Pi (`pi_workspace.rhai`), Claude (`claude.rhai`), and the rest under
`examples/harnesses/`.

## Session API (what you call)

| Call | Role |
|------|------|
| `launch(name, opts)` | Open a session for one adapter |
| `prompt(...)` | Send a user turn |
| `expect` / `wait` | Block until a stream condition matches |
| `await_turn()` | Drain until the current turn ends |
| `close()` | Tear down the session |

Tools and hooks: `examples/harnesses/pi_tools.rhai`. Details in [Waiting on the stream](waits.md).

Full language: [Rhai scripts](rhai.md). Events and multi-turn: [How it works](concepts.md).
