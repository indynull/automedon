// Multi-turn Cursor agent CLI (stream-json + resume/continue).
// Requires: `agent` / `cursor-agent` / `cursor` on PATH and Cursor authentication.
//
//   medon run examples/harnesses/cursor.rhai --print

let s = launch("cursor", #{
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
print(`done session=${s.session_id()} text=${s.text()}`);
s.close();
