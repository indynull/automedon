// Live multi-turn via Grok ACP (`grok agent stdio`).
// Needs `grok` on PATH + grok auth.
//
//   medon run examples/live/grok_acp.rhai --print

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
print(`done session=${s.session_id()} text=${s.text()}`);
s.close();
