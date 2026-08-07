// Multi-turn smoke: Aider
//
// Binary:     aider
// Auth:       API keys for the model you configure (OpenAI, xAI, etc.)
// Stream:     plain text on --message path (no agent tool stream)
// Multi-turn: --chat-history-file + --restore-chat-history
//
// Daily QA loop:
//   1. Confirm `aider --help` works and your model credentials are set.
//   2. Set model below (required when no Aider defaults/keys exist).
//   3. medon run examples/harnesses/aider.rhai --print
//
// Tools/hooks: not on this path — wait_tool / wait_hook fail closed
// (capability not supported). See examples/harnesses/aider_no_tools.rhai.
//
// Note: multi_turn continuity uses a chat history file path as session id.

let s = launch("aider", #{
    // Required for non-interactive headless: provider/model your keys support.
    // model: "openrouter/auto",
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
