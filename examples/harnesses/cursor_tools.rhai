// Tools + hooks smoke: Cursor
// Binary: cursor-agent | Stream: tool_call shell/edit → Pre/PostToolUse
//
//   automedon run examples/harnesses/cursor_tools.rhai --print

let s = launch("cursor", #{
    yolo: true,
    multi_turn: true,
    timeout_ms: 180_000
});

s.prompt(`Use the shell tool once to run: echo AUTOMEDON_HOOKS
Then reply with the line: AUTOMEDON_HOOKS_DONE`);

s.wait(timeout_ms(wait_hook_started("PreToolUse"), 120_000));
s.wait(timeout_ms(wait_tool("shell"), 120_000));
s.wait(timeout_ms(wait_hook_finished("PostToolUse"), 120_000));
s.wait(timeout_ms(wait_text("AUTOMEDON_HOOKS_DONE"), 120_000));
s.wait(timeout_ms(wait_turn_complete(), 60_000));

print(`OK session=${s.session_id()} tools=${s.tool_names()}`);
s.close();
"OK"
