# Driver method coverage (Rhai)

Contributor map: every free function and session method registered in
`crates/automedon/src/dsl/engine.rs` is exercised somewhere for the test suite.
Interactive paths that product adapters do not implement (permission deny, plan
reject, goals, and so on) are covered by the internal **mock** adapter under
`examples/mock/` -- not a public operator path.

## Product scripts (what operators run)

| Job | Script | Markers |
|-----|--------|---------|
| Pi multi-turn workspace tools | [`harnesses/pi_workspace.rhai`](harnesses/pi_workspace.rhai) | `PI_WS_T1`, `PI_WS_OK` |
| Grok multi-turn coding | [`harnesses/grok_workspace.rhai`](harnesses/grok_workspace.rhai) | `DONE:fib`, `GROK_WS_OK` |

Shorter smokes: `harnesses/pi.rhai`, `harnesses/pi_tools.rhai`, `harnesses/grok.rhai`, `harnesses/grok_coding.rhai`.

## Session methods

| Method | Example |
|--------|---------|
| `launch` | `mock/full_driver_surface.rhai` |
| `prompt` | `mock/full_driver_surface.rhai` |
| `expect` | `mock/full_driver_surface.rhai` |
| `wait` | `mock/full_driver_surface.rhai` |
| `wait_for` | `mock/full_driver_surface.rhai` |
| `await_turn` | `mock/full_driver_surface.rhai` |
| `run` | `mock/full_driver_surface.rhai` |
| `approve` | `mock/full_driver_surface.rhai` |
| `deny` | `mock/full_driver_surface.rhai` |
| `approve_plan` | `mock/full_driver_surface.rhai` |
| `reject_plan` | `mock/full_driver_surface.rhai` |
| `drain` | `mock/full_driver_surface.rhai` |
| `close` | `mock/full_driver_surface.rhai` |
| `text` | `mock/full_driver_surface.rhai` |
| `turn_text` | `mock/full_driver_surface.rhai` |
| `thinking` | `mock/full_driver_surface.rhai` |
| `session_id` | `mock/full_driver_surface.rhai` |
| `harness` | `mock/full_driver_surface.rhai` |
| `finished` | `mock/full_driver_surface.rhai` |
| `turn` | `mock/full_driver_surface.rhai` |
| `tool_names` | `mock/full_driver_surface.rhai` |

## Assert helpers

| Call | Example |
|------|---------|
| `assert_contains` | `mock/full_driver_surface.rhai` |
| `assert_true` | `mock/full_driver_surface.rhai` |

## Expect constructors

| Constructor | Example |
|-------------|---------|
| `text` | `mock/full_driver_surface.rhai` |
| `thinking` | `mock/full_driver_surface.rhai` |
| `tool` | `mock/full_driver_surface.rhai` |
| `tool_any` | `mock/full_driver_surface.rhai` |
| `tool_input` | `mock/full_driver_surface.rhai` |
| `tool_result` | `mock/full_driver_surface.rhai` |
| `tool_result_contains` | `mock/full_driver_surface.rhai` |
| `tool_result_error` | `mock/full_driver_surface.rhai` |
| `permission` | `mock/full_driver_surface.rhai` |
| `hook` | `mock/full_driver_surface.rhai` |
| `hook_any` | `mock/full_driver_surface.rhai` |
| `hook_started` | `mock/full_driver_surface.rhai` |
| `hook_finished` | `mock/full_driver_surface.rhai` |
| `hook_phase` | `mock/full_driver_surface.rhai` |
| `plan` | `mock/full_driver_surface.rhai` |
| `plan_summary` | `mock/full_driver_surface.rhai` |
| `plan_resolved` | `mock/full_driver_surface.rhai` |
| `goal` | `mock/full_driver_surface.rhai` |
| `goal_title` | `mock/full_driver_surface.rhai` |
| `goal_progress` | `mock/full_driver_surface.rhai` |
| `goal_completed` | `mock/full_driver_surface.rhai` |
| `session_info` | `mock/full_driver_surface.rhai` |
| `turn_complete` | `mock/full_driver_surface.rhai` |
| `process_exit` | `mock/full_driver_surface.rhai` |
| `done` | `mock/full_driver_surface.rhai` |
| `timeout_ms` (Expect) | `mock/full_driver_surface.rhai` |

## Wait constructors

| Constructor | Example |
|-------------|---------|
| `wait_text` | `mock/full_driver_surface.rhai` |
| `wait_tool` | `mock/full_driver_surface.rhai` |
| `wait_tool_any` | `mock/full_driver_surface.rhai` |
| `wait_tool_result` | `mock/full_driver_surface.rhai` |
| `wait_tool_input` | `mock/full_driver_surface.rhai` |
| `wait_tool_result_contains` | `mock/full_driver_surface.rhai` |
| `wait_tool_result_error` | `mock/full_driver_surface.rhai` |
| `wait_permission` | `mock/full_driver_surface.rhai` |
| `wait_hook` | `mock/full_driver_surface.rhai` |
| `wait_hook_any` | `mock/full_driver_surface.rhai` |
| `wait_hook_started` | `mock/full_driver_surface.rhai` |
| `wait_hook_finished` | `mock/full_driver_surface.rhai` |
| `wait_hook_phase` | `mock/full_driver_surface.rhai` |
| `wait_plan` | `mock/full_driver_surface.rhai` |
| `wait_goal` | `mock/full_driver_surface.rhai` |
| `wait_turn_complete` | `mock/full_driver_surface.rhai` |
| `wait_done` | `mock/full_driver_surface.rhai` |
| `wait_timeout_ms` | `mock/full_driver_surface.rhai` |
| `timeout_ms` (Wait) | `mock/full_driver_surface.rhai` |

## Internal mock fixtures (suite only)

| Script | Scenario |
|--------|----------|
| `mock/smoke.rhai` | Tools smoke |
| `mock/multi_turn.rhai` | Multi-turn + permission approve + plan approve + goal |
| `mock/wait_hooks.rhai` | Wait API on hooks |
| `mock/full_driver_surface.rhai` | Full method tour for continuous integration |

```bash
# Contributors -- also covered by cargo test
medon run examples/mock/full_driver_surface.rhai --print
```
