// Live multi-turn: Aider via chat-history restore.
// Needs `aider` on PATH + model credentials (default xai/grok-4.5 + XAI_API_KEY).
//
//   medon run examples/live/aider.rhai --print

let s = launch("aider", #{
    model: "xai/grok-4.5",
    no_git: true,
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
