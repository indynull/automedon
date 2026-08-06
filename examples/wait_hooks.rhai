// Live waits: Pi tool lifecycle → PreToolUse / PostToolUse + tool events.
// Needs `pi` on PATH + xAI credentials for Pi (`provider=xai`).
//
//   medon run examples/wait_hooks.rhai --print
//
// Offline: examples/mock/wait_hooks.rhai

let s = launch("pi", #{
    yolo: true,
    provider: "xai",
    model: "grok-4.5",
    multi_turn: true,
    timeout_ms: 180_000
});

s.prompt(`Use a shell/run tool exactly once to execute:
  echo AUTOMEDON_HOOKS
Then finish your reply with the line: AUTOMEDON_HOOKS_DONE`);

// Block until the stream emits each signal (tool lifecycle maps to hooks on Pi).
s.wait(timeout_ms(wait_hook_started("PreToolUse"), 120_000));
s.wait(timeout_ms(wait_tool_any(), 120_000));
s.wait(timeout_ms(wait_hook_finished("PostToolUse"), 120_000));
s.wait(timeout_ms(wait_text("AUTOMEDON_HOOKS_DONE"), 120_000));
s.wait(timeout_ms(wait_turn_complete(), 60_000));

print(`WAIT_HOOKS_OK session=${s.session_id()} tools=${s.tool_names()}`);
s.close();
"WAIT_HOOKS_OK"
