// Aider has no agent tool stream on --message path.
// Shared wait_tool / wait_hook must fail closed with a capability error.
//
//   medon run examples/harnesses/aider_no_tools.rhai --print

let s = launch("aider", #{
    no_git: true,
    multi_turn: false,
    timeout_ms: 5_000
});

// Capability bits: stream_tools=false, wait_hooks=false.
// These waits must error immediately (not hang).
let tool_err = "";
try {
    s.wait(timeout_ms(wait_tool_any(), 1_000));
    tool_err = "UNEXPECTED_OK";
} catch (e) {
    tool_err = `${e}`;
}
let hook_err = "";
try {
    s.wait(timeout_ms(wait_hook_started("PreToolUse"), 1_000));
    hook_err = "UNEXPECTED_OK";
} catch (e) {
    hook_err = `${e}`;
}

if tool_err == "UNEXPECTED_OK" || hook_err == "UNEXPECTED_OK" {
    throw "expected capability errors, got success";
}
assert_contains(tool_err, "capability");
assert_contains(hook_err, "capability");

print(`OK fail-closed tools=${tool_err} hooks=${hook_err}`);
s.close();
"OK"
