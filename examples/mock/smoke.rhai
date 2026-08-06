// Offline smoke — in-process mock only (no network).
// Run: medon run examples/mock/smoke.rhai --print

let s = launch("mock", #{ scenario: "tools", timeout_ms: 5_000 });
s.prompt("list the workspace");
s.expect(tool("list_dir"));
s.expect(tool_result("list_dir"));
s.expect(text("listed"));
s.expect(turn_complete());

print(`harness=${s.harness()} text=${s.text()}`);
assert_contains(s.text(), "listed");
s.text()
