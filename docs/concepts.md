# Concepts

## Session

A `Session` is one Automedon conversation with a harness. It owns:

- The adapter (how to spawn and parse)
- The event transcript
- Multi-turn context (session id, turn number)
- Process supervision (kill-on-drop)

## Events

Adapters normalize product streams into shared events, including:

| Event | Meaning |
|-------|---------|
| `TextDelta` / `ThinkingDelta` | Assistant text / thinking |
| `ToolCall` / `ToolResult` | Model tool use |
| `HookStarted` / `HookFinished` | Lifecycle hooks (e.g. PreToolUse) |
| `TurnComplete` | Current **turn** finished |
| `ProcessExit` | Child process exited |
| `SessionInfo` | Session / resume id from the harness |
| `Done` | **Session** finished (do not use for per-turn end on product adapters) |
| `Error` | Harness or adapter error |

## Multi-turn

Many CLIs are **process-per-turn**: each `prompt` spawns a new process with resume/history flags. Others keep one long-lived process (e.g. Grok ACP).

- Continuity uses `SessionInfo` / session id on the next prepare (resume, session-id, chat-history path, …).
- `TurnComplete` or `ProcessExit` ends a turn; `Done` closes the whole session.

## Wait and expect

Both match predicates on the event stream. Prefer waits for tools/hooks; expect for text/markers. Share the same timeout defaults from launch options when configured.

## Capabilities

Product adapters advertise features the driver implements. Calling `approve` / `approve_plan` without the interactive bit fails with a clear error.

List current bits:

```bash
medon adapters
```

Status table: [matrix.md](matrix.md).

## Adapters

One module per product: discover binary, build argv (or ACP spawn), parse lines to events, optional encode for mid-flight control. Quirks stay in the adapter and `LaunchOptions.extra` — not new Session methods per product.

See [Architecture](architecture.md).
