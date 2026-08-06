// Live multi-turn: Gemini CLI (or Antigravity via `agy` when installed).
// Needs `gemini` or `agy` on PATH + eligible Google auth.
// Note: free-tier Gemini Code Assist may return IneligibleTierError.
//
//   medon run examples/live/gemini.rhai --print

let s = launch("gemini", #{
    yolo: true,
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
