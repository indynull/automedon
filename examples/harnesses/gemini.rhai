// Multi-turn smoke: Gemini CLI
//
// Binary:     gemini (prefers agy if present)
// Auth:       Gemini / Google auth
// Stream:     -p -o stream-json
// Multi-turn: -r / resume
//
// Daily QA loop:
//   1. Confirm `gemini` works outside Automedon (login / one-shot prompt).
//   2. automedon run examples/harnesses/gemini.rhai --print
//   3. Expect AUTOMEDON_T1 then AUTOMEDON_T2; print session id after turn 1.
//
// Optional: raise timeout_ms for slow models; set model/cwd in the launch map.

let s = launch("gemini", #{
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
assert_contains(t, "AUTOMEDON_T1");
assert_contains(t, "AUTOMEDON_T2");
print(`turn2 session=${s.session_id()} text=${t}`);
s.close();
"OK"
