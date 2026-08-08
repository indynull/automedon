// Richer Claude Code check: multi-turn + tools + content expects.
//
// Binary:     claude
// Auth:       Claude Code / Anthropic login
// Stream:     stream-json + --include-hook-events
// Multi-turn: --resume / --continue
// Workspace:  examples/automedon_demo/ (created by the agent)
//
//   automedon run examples/harnesses/claude_workspace.rhai --print
//
// Markers: CLAUDE_WS_T1, CLAUDE_WS_OK

let s = launch("claude", #{
    yolo: true,
    multi_turn: true,
    timeout_ms: 300_000,
    max_turns: 12
});

s.prompt(`You are working in this Automedon git checkout (current workspace).

Create directory examples/automedon_demo if needed.
Write file examples/automedon_demo/claude_note.txt with exactly this single line (no quotes):
CLAUDE_WS_T1

Use your file tools. When the file is written, end your reply with a single line:
CLAUDE_WS_T1`);

// Prefer structured stream signals when the product emits them.
s.wait(timeout_ms(wait_tool_any(), 240_000));
// Optional: write payload often appears in tool input as path or content.
// Do not hard-fail if the product uses a different tool name — tool_any already ran.
s.expect(timeout_ms(text("CLAUDE_WS_T1"), 240_000));
s.await_turn();
print(`claude turn1 session=${s.session_id()} tools=${s.tool_names()}`);
assert_contains(s.text(), "CLAUDE_WS_T1");
assert_true(s.session_id() != "");
assert_true(s.turn() >= 1);

s.prompt(`Continue in the same session (resume).

Read examples/automedon_demo/claude_note.txt with a file tool.
Append a second line exactly: CLAUDE_WS_T2
Then print the full file contents in your reply and end with a single line: CLAUDE_WS_OK`);

s.wait(timeout_ms(wait_tool_any(), 240_000));
s.expect(timeout_ms(text("CLAUDE_WS_OK"), 240_000));
s.await_turn();

let t = s.text();
print(`claude turn2 session=${s.session_id()} turn=${s.turn()}`);
print("--- transcript tail ---");
print(t);

assert_true(s.turn() >= 2);
assert_contains(t, "CLAUDE_WS_T1");
assert_contains(t, "CLAUDE_WS_OK");
assert_true(t.contains("CLAUDE_WS_T2") || t.contains("CLAUDE_WS_OK"));
// Multi-turn should keep a stable session id when system/result frames provide one.
assert_true(s.session_id() != "");

s.close();
print("CLAUDE_WS_OK");
"CLAUDE_WS_OK"
