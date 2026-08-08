// Offline tour of every Rhai driver method and expect/wait constructor.
// No product binary. Primary method→script matrix entry for the full surface.
//
//   automedon run examples/mock/full_driver_surface.rhai --print
//
// Markers: FULL_DRIVER_SURFACE_OK

// ---------------------------------------------------------------------------
// 1) Echo: launch, run, accessors, process_exit + done, drain, close, finished
// ---------------------------------------------------------------------------
let echo = launch("mock", #{ scenario: "echo", timeout_ms: 8_000 });
assert_true(echo.harness() == "mock");
assert_true(echo.turn() == 0 || echo.turn() == 1);

let out = echo.run("hello-surface");
assert_contains(out, "ECHO:hello-surface");
assert_contains(echo.text(), "ECHO:hello-surface");
assert_contains(echo.turn_text(), "ECHO:");
assert_contains(echo.thinking(), "echoing:");
assert_true(echo.session_id() != "");
print(`echo harness=${echo.harness()} session=${echo.session_id()} turn=${echo.turn()}`);

// Session already emitted process exit + done via mock one-shot footer.
echo.expect(timeout_ms(process_exit(), 3_000));
echo.expect(timeout_ms(done(), 3_000));
assert_true(echo.finished());
echo.drain();
echo.close();
assert_true(echo.finished());
print("section echo ok");

// ---------------------------------------------------------------------------
// 2) Thinking + session_info + turn_complete via expect
// ---------------------------------------------------------------------------
let th = launch("mock", #{ scenario: "think", timeout_ms: 8_000 });
th.prompt("ponder");
th.expect(session_info());
th.expect(thinking("thinking"));
th.expect(text("done:ponder"));
th.expect(turn_complete());
assert_contains(th.thinking(), "thinking");
assert_contains(th.text(), "done:ponder");
th.close();
print("section think ok");

// ---------------------------------------------------------------------------
// 3) Tools: tool / tool_any / tool_result + wait_* variants + tool_names
// ---------------------------------------------------------------------------
let tools = launch("mock", #{ scenario: "tools", timeout_ms: 8_000 });
tools.prompt("list workspace");
// Named tools first (cursor advances); tool_any would consume list_dir and skip name match.
// tool_input also matches name; do not double-expect the same ToolCall.
tools.expect(tool_input("list_dir", "."));
tools.expect(tool_result_contains("list_dir", "Cargo.toml"));
tools.expect(tool_input("read_file", "Cargo.toml"));
tools.expect(tool_result_contains("read_file", "workspace"));
tools.expect(text("listed"));
tools.expect(turn_complete());
// Bare tool / tool_result on a fresh session
let tools2 = launch("mock", #{ scenario: "tools", timeout_ms: 8_000 });
tools2.prompt("bare");
tools2.expect(tool("list_dir"));
tools2.expect(tool_result("list_dir"));
tools2.expect(tool("read_file"));
tools2.expect(tool_result("read_file"));
tools2.close();
let names = tools.tool_names();
print(`tools names=${names}`);
assert_contains(tools.text(), "listed");
tools.close();

// tool_any + wait_tool_any / wait / wait_for / wait_timeout_ms on a fresh session
let tw = launch("mock", #{ scenario: "tools", timeout_ms: 8_000 });
tw.prompt("again");
tw.expect(tool_any()); // matches first tool call (list_dir)
tw.wait(wait_tool_result("list_dir"));
// wait_tool_any matches next tool call (read_file)
tw.wait(wait_tool_any());
tw.wait(wait_tool_result("read_file"));
tw.wait_for(wait_timeout_ms(wait_text("listed"), 5_000));
tw.wait(timeout_ms(wait_turn_complete(), 5_000));
// Content waits (one predicate per event)
let tw2 = launch("mock", #{ scenario: "tools", timeout_ms: 8_000 });
tw2.prompt("named");
tw2.wait(wait_tool_input("list_dir", "."));
tw2.wait(wait_tool_result_contains("list_dir", "src/"));
tw2.wait(wait_tool("read_file"));
tw2.wait(wait_tool_result_error("read_file", false));
tw2.close();
tw.close();
print("section tools ok");

// ---------------------------------------------------------------------------
// 4) Hooks: expect hook* + wait hook*
// ---------------------------------------------------------------------------
let hk = launch("mock", #{ scenario: "hooks", timeout_ms: 10_000 });
hk.prompt("ping");
hk.expect(hook_started("PreToolUse"));
hk.expect(tool("run_terminal_command"));
hk.expect(hook_finished("PreToolUse"));
// PostToolUse start then finished (phase=post on both).
hk.expect(hook_phase("PostToolUse", "post"));
hk.expect(hook("PostToolUse"));
hk.expect(text("hooks_done:ping"));
hk.expect(turn_complete());
hk.close();

let hw = launch("mock", #{ scenario: "hooks", timeout_ms: 10_000 });
hw.prompt("pong");
// Explicit wait_hook_started, then tool / finished / named / phase.
hw.wait(wait_hook_started("PreToolUse"));
hw.wait(wait_tool("run_terminal_command"));
hw.wait(wait_hook_finished("PreToolUse"));
hw.wait(wait_hook("PostToolUse"));
hw.wait(wait_hook_phase("PostToolUse", "post"));
hw.wait(wait_text("hooks_done:pong"));
hw.wait(wait_turn_complete());
hw.close();

// wait_hook_any on a short session
let hw2 = launch("mock", #{ scenario: "hooks", timeout_ms: 10_000 });
hw2.prompt("anyw");
hw2.wait(wait_hook_any());
hw2.wait(wait_text("hooks_done:anyw"));
hw2.close();

// Expect-side hook_any (not wait)
let ha = launch("mock", #{ scenario: "hooks", timeout_ms: 10_000 });
ha.prompt("any");
ha.expect(hook_any());
ha.expect(text("hooks_done:any"));
ha.close();
print("section hooks ok");

// ---------------------------------------------------------------------------
// 5) Permission: wait_permission, approve, deny
// ---------------------------------------------------------------------------
// wait_permission path + approve
let pa = launch("mock", #{ scenario: "permission", timeout_ms: 8_000 });
pa.prompt("allow-me");
pa.wait(wait_permission());
pa.approve();
pa.expect(text("ran:allow-me"));
pa.expect(turn_complete());
print("permission approve ok");

// expect(permission) path + deny
let pd = launch("mock", #{ scenario: "permission", timeout_ms: 8_000 });
pd.prompt("block-me");
pd.expect(permission());
pd.deny();
pd.expect(text("denied"));
pd.expect(turn_complete());
print("permission deny ok");

// ---------------------------------------------------------------------------
// 6) Plan: wait_plan, plan*, approve_plan, reject_plan
// ---------------------------------------------------------------------------
// wait_plan then approve (do not re-expect the same PlanPresented)
let pl = launch("mock", #{ scenario: "plan", timeout_ms: 8_000 });
pl.prompt("ship-it");
pl.wait(wait_plan());
pl.approve_plan();
pl.expect(plan_resolved(true));
pl.expect(text("executing plan"));
print("plan approve ok");

// plan() then reject (plan and plan_summary both match PlanPresented — use one each session)
let pr = launch("mock", #{ scenario: "plan", timeout_ms: 8_000 });
pr.prompt("nope");
pr.expect(plan());
pr.reject_plan();
pr.expect(plan_resolved(false));
pr.expect(text("plan rejected"));
print("plan reject ok");

let ps = launch("mock", #{ scenario: "plan", timeout_ms: 8_000 });
ps.prompt("summary-path");
ps.expect(plan_summary("summary-path"));
ps.approve_plan();
ps.expect(plan_resolved(true));
ps.expect(text("executing plan"));
print("plan summary ok");

// ---------------------------------------------------------------------------
// 7) Goals: wait_goal + goal*
// ---------------------------------------------------------------------------
// wait_goal consumes GoalStarted; remaining progress/completed/text follow
let g = launch("mock", #{ scenario: "goal", timeout_ms: 8_000 });
g.prompt("mission");
g.wait(wait_goal());
g.expect(goal_progress());
g.expect(goal_completed(true));
g.expect(text("goal_ok:mission"));
g.expect(turn_complete());
g.wait(wait_done());
assert_true(g.finished());
print("section goals ok");

// expect(goal()) alone (same event as goal_title — separate session)
let g2 = launch("mock", #{ scenario: "goal", timeout_ms: 8_000 });
g2.prompt("mission2");
g2.expect(goal());
g2.expect(goal_progress());
g2.expect(goal_completed(true));
g2.close();

let g3 = launch("mock", #{ scenario: "goal", timeout_ms: 8_000 });
g3.prompt("mission3");
g3.expect(goal_title("mission3"));
g3.expect(goal_progress());
g3.expect(goal_completed(true));
g3.close();
print("section goals expect ok");

// ---------------------------------------------------------------------------
// 8) Multi-turn: prompt, await_turn, session continuity, assert_*
// ---------------------------------------------------------------------------
let m = launch("mock", #{ scenario: "multi", multi_turn: true, timeout_ms: 10_000 });
m.prompt("alpha");
m.expect(timeout_ms(text("T1:alpha"), 5_000));
m.await_turn();
assert_true(!m.finished());
print(`multi t1 session=${m.session_id()} turn=${m.turn()}`);

m.prompt("beta");
m.expect(text("T2:beta"));
m.expect(text("prior=T1:alpha"));
m.await_turn();
assert_contains(m.text(), "T1:alpha");
assert_contains(m.text(), "prior=T1:alpha");
assert_true(m.turn() >= 2);
m.close();
print("section multi ok");

print("FULL_DRIVER_SURFACE_OK");
"FULL_DRIVER_SURFACE_OK"
