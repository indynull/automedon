// Tools + hooks smoke: Pi
// Binary: pi | Auth: Pi provider credentials
// Stream: --mode json; tool lifecycle → PreToolUse / PostToolUse
//
//   medon run examples/harnesses/pi_tools.rhai --print
//
// Optional launch extras: provider, model, tools, extension

let s = launch("pi", #{
    yolo: true,
    multi_turn: true,
    timeout_ms: 180_000
});

s.prompt(`Use a shell/run tool exactly once to execute:
  echo AUTOMEDON_HOOKS
Then finish your reply with the line: AUTOMEDON_HOOKS_DONE`);

s.wait(timeout_ms(wait_hook_started("PreToolUse"), 120_000));
s.wait(timeout_ms(wait_tool_any(), 120_000));
s.wait(timeout_ms(wait_hook_finished("PostToolUse"), 120_000));
s.wait(timeout_ms(wait_text("AUTOMEDON_HOOKS_DONE"), 120_000));
s.wait(timeout_ms(wait_turn_complete(), 60_000));

print(`OK session=${s.session_id()} tools=${s.tool_names()}`);
s.close();
"OK"
