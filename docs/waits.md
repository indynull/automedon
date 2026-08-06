# Waits and flakes

## Why wait

Agent streams are asynchronous. Asserting only on final text often races tools and multi-step turns. Wait/expect block until a condition matches or the timeout expires.

## Rules of thumb

1. Prefer **specific** waits (`tool("bash")`, `text("DONE")`) over sleeping.
2. Call **`await_turn()`** before the next `prompt` when the turn may still be streaming.
3. Set **`timeout_ms`** (or Rust `timeout`) high enough for real model latency (often 60–180s live).
4. **Process-per-turn:** session id must be captured before turn 2 (stdout JSON, or stderr footer for Copilot).

## Common flake causes

| Symptom | Likely cause |
|---------|----------------|
| Timeout on text | Marker never emitted; wrong wait; model paraphrased |
| `session_id` empty turn 2 | Resume footer/session frame not parsed or drained |
| Hang | Child waiting on stdin; adapter should use null stdin for one-shot |
| Capability error | `approve` / plan without interactive capability |
| Session finished early | Adapter emitted session `Done` on turn end (bug) |

## Turn end vs session end

- **Turn end:** `TurnComplete` and/or `ProcessExit` under `multi_turn`
- **Session end:** `Done` or `close()`

Product multi-turn adapters must not map per-turn result frames to session `Done`.
