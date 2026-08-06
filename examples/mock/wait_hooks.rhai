// Offline waits on hooks + tools (mock only).
// medon run examples/mock/wait_hooks.rhai --print

let s = launch("mock", #{ scenario: "hooks", timeout_ms: 10_000 });
s.prompt("ping");

s.wait(wait_hook_started("PreToolUse"));
s.wait(wait_tool("run_terminal_command"));
s.wait(wait_hook_finished("PreToolUse"));
s.wait(wait_hook_phase("PostToolUse", "post"));
s.wait(wait_text("hooks_done:ping"));
s.wait(wait_turn_complete());

print("WAIT_HOOKS_OK");
"WAIT_HOOKS_OK"
