// Offline multi-turn + permission / plan / goal (mock only).
// medon run examples/mock/multi_turn.rhai --print

// --- multi-turn continuity ---
let s = launch("mock", #{ scenario: "multi", timeout_ms: 10_000 });
s.prompt("alpha");
s.expect(text("T1:alpha"));
s.await_turn();
print(`turn1 ok session=${s.session_id()}`);

s.prompt("beta");
s.expect(text("T2:beta"));
s.expect(text("prior=T1:alpha"));
s.await_turn();
assert_contains(s.text(), "T1:alpha");
assert_contains(s.text(), "prior=T1:alpha");
print(`turn2 ok text=${s.text()}`);
s.close();

// --- permission approve ---
let p = launch("mock", #{ scenario: "permission" });
p.prompt("payload");
p.expect(permission());
p.approve();
p.expect(text("ran:payload"));
p.expect(turn_complete());
print("permission ok");

// --- plan ---
let pl = launch("mock", #{ scenario: "plan" });
pl.prompt("refactor");
pl.expect(plan_summary("refactor"));
pl.approve_plan();
pl.expect(plan_resolved(true));
pl.expect(text("executing plan"));
print("plan ok");

// --- goal ---
let g = launch("mock", #{ scenario: "goal" });
g.prompt("ship");
g.expect(goal_title("ship"));
g.expect(goal_progress());
g.expect(goal_completed(true));
g.expect(text("goal_ok:ship"));
print("goal ok");

"MULTI_TURN_OK"
