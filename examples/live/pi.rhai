// Live multi-turn: Pi with xAI / Grok model.
// Needs `pi` on PATH + xAI credentials configured for pi (`provider=xai`).
//
//   medon run examples/live/pi.rhai --print

let s = launch("pi", #{
    yolo: true,
    provider: "xai",
    model: "grok-4.5",
    multi_turn: true,
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
