// One-turn smoke: Grok Build
// Binary: grok | Auth: Grok login | Stream: streaming-json
//
//   automedon run examples/harnesses/grok_smoke.rhai --print

let s = launch("grok", #{
    yolo: true,
    timeout_ms: 120_000
});

s.prompt("Reply with exactly one line: AUTOMEDON_SMOKE");
s.expect(timeout_ms(text("AUTOMEDON_SMOKE"), 90_000));
s.await_turn();
assert_contains(s.text(), "AUTOMEDON_SMOKE");
print(`session=${s.session_id()} text=${s.text()}`);
s.close();
"SMOKE_OK"
