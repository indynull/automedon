// Multi-turn smoke: Aider
//
// Binary:     aider
// Auth:       API keys for the model you configure (OpenAI, xAI, etc.)
// Stream:     plain text on --message path (no agent tool stream)
// Multi-turn: --chat-history-file + --restore-chat-history
//
// Daily QA loop:
//   1. Confirm `aider --help` works and your model credentials are set.
//   2. Set model below (or via env expected by Aider).
//   3. medon run examples/harnesses/aider.rhai --print
//
// Note: multi_turn continuity uses a chat history file path as session id.

let s = launch("aider", #{
    // model: "provider/model-id",
    no_git: true,
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
