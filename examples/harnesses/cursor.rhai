// Multi-turn smoke: Cursor agent
//
// Binary:     cursor-agent (preferred; bare `agent` collides with Grok Build)
// Auth:       cursor-agent login or CURSOR_API_KEY
// Stream:     print + stream-json
// Multi-turn: --resume / --continue
//
// Daily QA loop:
//   1. Confirm `cursor-agent` works outside Automedon (login / one-shot prompt).
//   2. medon run examples/harnesses/cursor.rhai --print
//   3. Expect AUTOMEDON_T1 then AUTOMEDON_T2; print session id after turn 1.
//
// Optional: raise timeout_ms for slow models; set model/cwd in the launch map.

let s = launch("cursor", #{
    yolo: true,
    multi_turn: true,
    timeout_ms: 180_000,
    binary: "cursor-agent"
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
