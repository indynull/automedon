// Richer OpenAI Codex check: multi-turn + tools + content expects.
//
// Binary:     codex
// Auth:       OpenAI / Codex login
// Stream:     codex exec --json
// Multi-turn: exec resume <id|--last>
// Workspace:  examples/automedon_demo/ (created by the agent)
//
//   automedon run examples/harnesses/codex_workspace.rhai --print
//
// Markers: CODEX_WS_T1, CODEX_WS_OK

let s = launch("codex", #{
    yolo: true,
    multi_turn: true,
    timeout_ms: 300_000
});

s.prompt(`You are working in this Automedon git checkout (current workspace).

Create directory examples/automedon_demo if needed.
Write file examples/automedon_demo/codex_note.txt with exactly this single line (no quotes):
CODEX_WS_T1

Use your file tools. When the file is written, end your reply with a single line:
CODEX_WS_T1`);

s.wait(timeout_ms(wait_tool_any(), 240_000));
s.expect(timeout_ms(text("CODEX_WS_T1"), 240_000));
s.await_turn();
print(`codex turn1 session=${s.session_id()} tools=${s.tool_names()}`);
assert_contains(s.text(), "CODEX_WS_T1");
assert_true(s.session_id() != "");
assert_true(s.turn() >= 1);

s.prompt(`Continue in the same session (resume).

Read examples/automedon_demo/codex_note.txt with a file tool.
Append a second line exactly: CODEX_WS_T2
Then print the full file contents in your reply and end with a single line: CODEX_WS_OK`);

s.wait(timeout_ms(wait_tool_any(), 240_000));
s.expect(timeout_ms(text("CODEX_WS_OK"), 240_000));
s.await_turn();

let t = s.text();
print(`codex turn2 session=${s.session_id()} turn=${s.turn()}`);
print("--- transcript tail ---");
print(t);

assert_true(s.turn() >= 2);
assert_contains(t, "CODEX_WS_T1");
assert_contains(t, "CODEX_WS_OK");
assert_true(t.contains("CODEX_WS_T2") || t.contains("CODEX_WS_OK"));
assert_true(s.session_id() != "");

s.close();
print("CODEX_WS_OK");
"CODEX_WS_OK"
