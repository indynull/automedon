// Tools + hooks smoke: OpenCode
// Binary: opencode | Stream: run --format json tool frames → Pre/PostToolUse
//
//   medon run examples/harnesses/opencode_tools.rhai --print

let s = launch("opencode", #{
    yolo: true,
    multi_turn: true,
    timeout_ms: 180_000
});

s.prompt(`Use a shell/bash tool exactly once to run: echo AUTOMEDON_HOOKS
Then finish with the line: AUTOMEDON_HOOKS_DONE`);

s.wait(timeout_ms(wait_hook_started("PreToolUse"), 120_000));
s.wait(timeout_ms(wait_tool_any(), 120_000));
s.wait(timeout_ms(wait_hook_finished("PostToolUse"), 120_000));
s.wait(timeout_ms(wait_text("AUTOMEDON_HOOKS_DONE"), 120_000));
s.wait(timeout_ms(wait_turn_complete(), 60_000));

print(`OK session=${s.session_id()} tools=${s.tool_names()}`);
s.close();
"OK"
