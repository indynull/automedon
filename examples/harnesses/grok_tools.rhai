// Tools + hooks smoke: Grok
// Binary: grok | Auth: Grok Build credentials
// Stream: streaming-json; tool_call / tool_call_update → PreToolUse / PostToolUse
//
//   automedon run examples/harnesses/grok_tools.rhai --print

let s = launch("grok", #{
    yolo: true,
    multi_turn: true,
    timeout_ms: 180_000
});

s.prompt(`Use run_terminal_command exactly once to execute:
  echo AUTOMEDON_HOOKS
Then finish your reply with the line: AUTOMEDON_HOOKS_DONE`);

s.wait(timeout_ms(wait_hook_started("PreToolUse"), 120_000));
s.wait(timeout_ms(wait_tool("run_terminal_command"), 120_000));
s.wait(timeout_ms(wait_hook_finished("PostToolUse"), 120_000));
s.wait(timeout_ms(wait_text("AUTOMEDON_HOOKS_DONE"), 120_000));
s.wait(timeout_ms(wait_turn_complete(), 60_000));

print(`OK session=${s.session_id()} tools=${s.tool_names()}`);
s.close();
"OK"
