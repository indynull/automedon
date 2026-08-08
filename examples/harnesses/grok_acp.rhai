// Multi-turn Grok via ACP (`grok agent stdio`)
// Binary: grok | Auth: Grok login | Path: extra.acp = true
//
//   automedon run examples/harnesses/grok_acp.rhai --print

let s = launch("grok", #{
    yolo: true,
    acp: true,
    timeout_ms: 180_000
});

s.prompt("Reply with exactly: AUTOMEDON_T1 and nothing else");
s.expect(timeout_ms(text("AUTOMEDON_T1"), 120_000));
s.await_turn();
print(`turn1 session=${s.session_id()}`);

s.prompt("Reply with exactly: AUTOMEDON_T2 and nothing else");
s.expect(timeout_ms(text("AUTOMEDON_T2"), 120_000));
s.await_turn();
assert_contains(s.text(), "AUTOMEDON_T1");
assert_contains(s.text(), "AUTOMEDON_T2");
s.close();
"OK"
