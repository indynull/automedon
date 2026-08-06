// Live multi-turn: GitHub Copilot CLI.
// Needs `copilot` on PATH + GitHub Copilot auth.
//
//   medon run examples/live/copilot.rhai --print

let s = launch("copilot", #{
    yolo: true,
    timeout_ms: 180_000
});

s.prompt("Reply with exactly: AUTOMEDON_T1 and nothing else");
s.expect(timeout_ms(text("AUTOMEDON_T1"), 120_000));
s.await_turn();
print(`turn1 session=${s.session_id()}`);

s.prompt("You previously said AUTOMEDON_T1. Reply with exactly: AUTOMEDON_T2 and nothing else");
s.expect(timeout_ms(text("AUTOMEDON_T2"), 120_000));
print(`done session=${s.session_id()} text=${s.text()}`);
s.close();
