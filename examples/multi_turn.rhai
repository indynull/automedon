// Live multi-turn: Grok Build (`--resume` continuity).
// Needs `grok` on PATH + grok auth.
//
//   medon run examples/multi_turn.rhai --print
//
// Offline (permission/plan/goal mock demos): examples/mock/multi_turn.rhai

let s = launch("grok", #{
    yolo: true,
    multi_turn: true,
    timeout_ms: 180_000
});

s.prompt("Reply with exactly: AUTOMEDON_T1 and nothing else");
s.expect(timeout_ms(text("AUTOMEDON_T1"), 120_000));
s.await_turn();
print(`turn1 session=${s.session_id()}`);

s.prompt("Reply with exactly: AUTOMEDON_T2 and nothing else");
s.expect(timeout_ms(text("AUTOMEDON_T2"), 120_000));
s.await_turn();

let t = s.text();
print(`turn2 session=${s.session_id()} text=${t}`);
assert_contains(t, "AUTOMEDON_T1");
assert_contains(t, "AUTOMEDON_T2");
assert_true(s.turn() >= 2);
s.close();
"MULTI_TURN_OK"
