# Rhai scripts

Scripts are ordinary Rhai files (`.rhai`). Entry points: `medon run` and `medon eval`.

Handbook code samples use the **`rust`** fence language so highlight.js can style them. highlight.js has no Rhai grammar; the [Rhai book](https://rhai.rs/book/about/related.html) recommends Rust (or JavaScript when you need string interpolation).

## Launch

```rust
let s = launch("grok", #{
    yolo: true,
    model: "optional-model-id",
    multi_turn: true,
    timeout_ms: 120_000,
    cwd: "/path/to/workspace",
    acp: false
});
```

| Key | Meaning |
|-----|---------|
| `yolo` | Product allow-all / skip-permission flags when the adapter maps them |
| `model` | Model id |
| `timeout_ms` | Default wait / expect timeout (milliseconds) |
| `cwd` | Working directory for the child process |
| `bin` | Override binary path |
| `scenario` | **Mock only** — which synthetic stream to play |
| other keys | Passed as `LaunchOptions.extra` (e.g. `provider`, `acp`, `chat_history_file`) |

`launch("name")` without a map uses defaults.

## Session methods

| Call | Role |
|------|------|
| `s.prompt(text)` | Start a user turn (may spawn a process) |
| `s.expect(pred)` | Block until an expect-predicate matches |
| `s.wait(w)` / `s.wait_for(w)` | Block until a wait-condition matches |
| `s.await_turn()` | Drain until the current turn ends |
| `s.run(text)` | `prompt` + `await_turn` + return that turn’s text |
| `s.approve()` / `s.deny()` | Interactive permission (requires capability) |
| `s.approve_plan()` / `s.reject_plan()` | Plan control (requires capability) |
| `s.drain()` | Drain until session done |
| `s.close()` | Tear down the session |
| `s.text()` | Full transcript text so far |
| `s.turn_text()` | Current turn text |
| `s.thinking()` | Thinking / thought text |
| `s.session_id()` | Resume id if known |
| `s.harness()` | Adapter id |
| `s.turn()` | Turn index |
| `s.finished()` | Session finished? |
| `s.tool_names()` | Array of tool names seen |

## Expect constructors

Pass these to `s.expect(...)`:

| Constructor | Matches |
|-------------|---------|
| `text("…")` | Text delta / accumulated text contains substring |
| `thinking("…")` | Thinking contains substring |
| `tool("name")` | Tool call by name |
| `tool_any()` | Any tool call |
| `tool_result("name")` | Tool result for name |
| `permission()` | Permission request |
| `hook("name")` / `hook_any()` | Hook by name / any |
| `hook_started("…")` / `hook_finished("…")` | Hook lifecycle |
| `hook_phase(name, phase)` | Hook with phase |
| `plan()` / `plan_summary("…")` / `plan_resolved(bool)` | Plan events |
| `goal()` / `goal_title("…")` / `goal_progress()` / `goal_completed(bool)` | Goal events |
| `session_info()` | Session info frame |
| `turn_complete()` | Turn completed |
| `process_exit()` | Process exited |
| `done()` | Session done |
| `timeout_ms(pred, ms)` | Same predicate with custom timeout |

## Wait constructors

Preferred for tools and hooks. Pass to `s.wait(...)`:

| Constructor | Role |
|-------------|------|
| `wait_text("…")` | Text |
| `wait_tool("…")` / `wait_tool_any()` / `wait_tool_result("…")` | Tools |
| `wait_hook("…")` / `wait_hook_any()` / `wait_hook_started` / `wait_hook_finished` / `wait_hook_phase` | Hooks |
| `wait_permission()` | Permission |
| `wait_plan()` / `wait_goal()` | Plan / goal |
| `wait_turn_complete()` / `wait_done()` | Boundaries |
| `timeout_ms(wait, ms)` or `wait_timeout_ms(wait, ms)` | Custom timeout |

## Assertions

| Call | Role |
|------|------|
| `assert_contains(hay, needle)` | Fail script if missing |
| `assert_true(bool)` | Fail script if false |

## End-to-end sketch

```rust
let s = launch("mock", #{ scenario: "multi", timeout_ms: 10_000 });
s.prompt("alpha");
s.expect(text("T1:alpha"));
s.await_turn();
s.prompt("beta");
s.expect(text("prior=T1:alpha"));
assert_contains(s.text(), "T1:alpha");
s.close();
"ok"
```

## Examples

| Path | Role |
|------|------|
| `examples/mock/*` | Offline mock |
| `examples/harnesses/*` | Product adapters |

See [Examples](examples.md).
