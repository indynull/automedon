// Richer Cursor agent check: multi-turn + tools via cursor-agent.
//
// Binary:     cursor-agent (explicit; avoid bare agent = Grok Build)
// Auth:       cursor-agent login or CURSOR_API_KEY
// Stream:     print + stream-json
// Multi-turn: --resume / --continue
// Workspace:  examples/automedon_demo/
//
//   medon run examples/harnesses/cursor_workspace.rhai --print
//
// Markers: CURSOR_WS_T1, CURSOR_WS_OK

let s = launch("cursor", #{
    yolo: true,
    multi_turn: true,
    timeout_ms: 300_000,
    binary: "cursor-agent"
});

s.prompt(`You are working in this Automedon git checkout (current workspace).

Create directory examples/automedon_demo if needed.
Write file examples/automedon_demo/cursor_note.txt with exactly this single line (no quotes):
CURSOR_WS_T1

Use your file tools. When the file is written, end your reply with a single line:
CURSOR_WS_T1`);

s.wait(timeout_ms(wait_tool_any(), 240_000));
s.expect(timeout_ms(text("CURSOR_WS_T1"), 240_000));
s.await_turn();
print(`cursor turn1 session=${s.session_id()} tools=${s.tool_names()}`);
assert_contains(s.text(), "CURSOR_WS_T1");
assert_true(s.session_id() != "");

s.prompt(`Continue in the same session (resume).

Read examples/automedon_demo/cursor_note.txt with a file tool.
Append a second line exactly: CURSOR_WS_T2
Then print the full file contents in your reply and end with a single line: CURSOR_WS_OK`);

s.wait(timeout_ms(wait_tool_any(), 240_000));
s.expect(timeout_ms(text("CURSOR_WS_OK"), 240_000));
s.await_turn();

let t = s.text();
print(`cursor turn2 session=${s.session_id()} turn=${s.turn()}`);
print("--- transcript tail ---");
print(t);

assert_true(s.turn() >= 2);
assert_contains(t, "CURSOR_WS_T1");
assert_contains(t, "CURSOR_WS_OK");
assert_true(s.session_id() != "");

s.close();
print("CURSOR_WS_OK");
"CURSOR_WS_OK"
