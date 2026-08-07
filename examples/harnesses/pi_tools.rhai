// Pi tool lifecycle → PreToolUse / PostToolUse + tool waits.
// Requires: `pi` on PATH and Pi provider credentials for your account.
// Optional launch extras: provider, model.
//
//   medon run examples/harnesses/pi_tools.rhai --print

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

print(`WAIT_HOOKS_OK session=${s.session_id()} tools=${s.tool_names()}`);
s.close();
"WAIT_HOOKS_OK"
