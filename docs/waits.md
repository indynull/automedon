# Waiting on the stream

Agent streams are asynchronous. Asserting only on final text races tools and multi-step turns. **Wait** and **expect** block until a condition matches or the timeout expires.

## Practical habits

Prefer **specific** waits (`tool("bash")`, `text("DONE")`) over sleeping. Call **`await_turn()`** before the next `prompt` while a turn may still be streaming. Set **`timeout_ms`** high enough for real model latency (often 60–180s on product CLIs). For **process-per-turn** adapters, the session id must be captured before turn 2 (stdout JSON, or Copilot’s stderr Resume footer).

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
