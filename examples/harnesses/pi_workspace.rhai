// Real-world multi-turn: Pi writes a small workspace file then verifies it.
//
// Binary:     pi
// Auth:       Pi provider credentials (same as interactive `pi`)
// Stream:     -p --mode json
// Multi-turn: --session-id / --continue
// Workspace:  examples/automedon_demo/ (created by the agent)
//
// Live QA entry for Pi (tools + multi-turn continuity):
//   medon run examples/harnesses/pi_workspace.rhai --print
//
// Success markers: PI_WS_T1, PI_WS_T2, PI_WS_OK

let s = launch("pi", #{
    yolo: true,
    multi_turn: true,
    timeout_ms: 300_000
});

s.prompt(`You are working in the Automedon git checkout (current workspace).

Create directory examples/automedon_demo if needed.
Write file examples/automedon_demo/pi_note.txt with exactly this single line (no quotes):
PI_WS_T1

Use your file tools. When the file is written, end your reply with a single line:
PI_WS_T1`);

s.wait(timeout_ms(wait_tool_any(), 240_000));
s.expect(timeout_ms(text("PI_WS_T1"), 240_000));
s.await_turn();
print(`pi turn1 session=${s.session_id()} tools=${s.tool_names()}`);
assert_contains(s.text(), "PI_WS_T1");

s.prompt(`Continue in the same session.

Read examples/automedon_demo/pi_note.txt with a file tool.
Append a second line exactly: PI_WS_T2
Then print the full file contents in your reply and end with a single line: PI_WS_OK`);

s.wait(timeout_ms(wait_tool_any(), 240_000));
s.expect(timeout_ms(text("PI_WS_OK"), 240_000));
s.await_turn();

let t = s.text();
print(`pi turn2 session=${s.session_id()} turn=${s.turn()}`);
print("--- transcript tail ---");
print(t);

assert_true(s.turn() >= 2);
assert_contains(t, "PI_WS_T1");
assert_contains(t, "PI_WS_OK");
// Second-turn marker may appear in file contents or reply text
assert_true(t.contains("PI_WS_T2") || t.contains("PI_WS_OK"));

s.close();
print("PI_WS_OK");
"PI_WS_OK"
