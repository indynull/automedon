// Multi-turn Copilot via ACP (`copilot --acp`)
// Binary: copilot | Path: extra.acp = true
//
//   automedon run examples/harnesses/copilot_acp.rhai --print

let s = launch("copilot", #{
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
print(`OK session=${s.session_id()}`);
s.close();
"OK"
