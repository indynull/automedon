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
| `scenario` | Internal mock adapter only (test suite) |
| other keys | Passed as `LaunchOptions.extra` (e.g. `provider`, `acp`, `chat_history_file`) |

`launch("name")` without a map uses defaults.

## Session methods

| Call | Role |
|------|------|
| `s.prompt(text)` | Start a user turn (may spawn a process) |
| `s.expect(pred)` | Block until an expect-predicate matches |
| `s.wait(w)` / `s.wait_for(w)` | Block until a wait-condition matches |
| `s.await_turn()` | Drain until the current turn ends |
| `s.run(text)` | `prompt` + `await_turn` + return that turn's text |
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
| `text("...")` | Text delta / accumulated text contains substring |
| `thinking("...")` | Thinking contains substring |
| `tool("name")` | Tool call by name |
| `tool_any()` | Any tool call |
| `tool_input("name", "frag")` | Tool call whose JSON input contains `frag` (code/path payloads) |
| `tool_result("name")` | Tool result for name |
| `tool_result_contains("name", "frag")` | Tool result whose output text contains `frag` |
| `tool_result_error("name", bool)` | Tool result with error flag |
| `text_regex("pattern")` | Turn text matches Rust regex |
| `permission()` | Permission request |
| `hook("name")` / `hook_any()` | Hook by name / any |
| `hook_started("...")` / `hook_finished("...")` | Hook lifecycle |
| `hook_phase(name, phase)` | Hook with phase |
| `plan()` / `plan_summary("...")` / `plan_resolved(bool)` | Plan events |
| `goal()` / `goal_title("...")` / `goal_progress()` / `goal_completed(bool)` | Goal events |
| `session_info()` | Session info frame |
| `turn_complete()` | Turn completed |
| `process_exit()` | Process exited |
| `done()` | Session done |
| `timeout_ms(pred, ms)` | Same predicate with custom timeout |

## Wait constructors

Preferred for tools and hooks. Pass to `s.wait(...)`:

| Constructor | Role |
|-------------|------|
| `wait_text("...")` | Text |
| `wait_tool("...")` / `wait_tool_any()` / `wait_tool_result("...")` | Tools |
| `wait_tool_input("name", "frag")` | Tool call input contains fragment |
| `wait_tool_result_contains("name", "frag")` | Tool result output contains fragment |
| `wait_tool_result_error("name", bool)` | Tool result error flag |
| `wait_text_regex("pattern")` | Text regex |
| `wait_hook("...")` / `wait_hook_any()` / `wait_hook_started` / `wait_hook_finished` / `wait_hook_phase` | Hooks |
| `wait_permission()` | Permission |
| `wait_plan()` / `wait_goal()` | Plan / goal |
| `wait_turn_complete()` / `wait_done()` | Boundaries |
| `timeout_ms(wait, ms)` or `wait_timeout_ms(wait, ms)` | Custom timeout |

## Assertions

| Call | Role |
|------|------|
| `assert_contains(hay, needle)` | Fail script if missing (transcript / string already in hand) |
| `assert_true(bool)` | Fail script if false |

### Code and diffs from a turn

There is no dedicated "diff" event and no filesystem helpers (use your test
runner or shell for on-disk checks). Harness-side patterns:

1. **Tool write payload** -- `expect(tool_input("write_file", "fn main"))` (product tool name).
2. **Tool result text** -- `expect(tool_result_contains("bash", "PASS"))`.
3. **Assistant text** -- `expect(text("DONE"))` / `text_regex(...)`.

## End-to-end sketch

Needs a product CLI (example: Grok Build):

```rust
let s = launch("grok", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
s.prompt("Reply with exactly: AUTOMEDON_T1 and nothing else");
s.expect(timeout_ms(text("AUTOMEDON_T1"), 120_000));
s.await_turn();
s.prompt("Reply with exactly: AUTOMEDON_T2 and nothing else");
s.expect(timeout_ms(text("AUTOMEDON_T2"), 120_000));
assert_contains(s.text(), "AUTOMEDON_T1");
s.close();
"ok"
```

## Examples

| Path | Role |
|------|------|
| `examples/harnesses/pi_workspace.rhai` | Pi workspace multi-turn |
| `examples/harnesses/grok_workspace.rhai` | Grok coding multi-turn |
| `examples/harnesses/*` | Other product adapters |
| `examples/DRIVER_SURFACE.md` | Contributor method map (suite / mock coverage) |

See [Examples](examples.md).
