# How it works

## Session

A **session** is one Automedon conversation with a harness. It owns the adapter, the event transcript, multi-turn context (session id, turn index), and process supervision (kill-on-drop).

```
Script  ──►  Session  ──►  Adapter.prepare()
                │                │
                │                ├─ child process (stdout / ACP)
                │                └─ synthetic events (mock)
                ▼
         Wait / Expect  ◄──  normalized Event stream
                │
                ▼
            Transcript
```

## Events

Adapters map product streams into a shared event set. The important ones:

| Event | Meaning |
|-------|---------|
| `TextDelta` / `ThinkingDelta` | Assistant text / thinking |
| `ToolCall` / `ToolResult` | Model tool use |
| `HookStarted` / `HookFinished` | Lifecycle around tools/turns |
| `TurnComplete` | This **turn** finished |
| `ProcessExit` | Child process exited |
| `SessionInfo` | Resume / session id from the product |
| `Done` | **Session** finished (not a stand-in for turn end) |
| `Error` | Harness or adapter error |

## Multi-turn

Two common shapes:

- **Process-per-turn** — each `prompt` may spawn a new process with resume/history flags. Continuity depends on capturing `SessionInfo` (or a history path) before the next turn.
- **Long-lived process** — e.g. Grok ACP (`extra.acp`): one process, many turns.

Turn end is `TurnComplete` and/or `ProcessExit`. Session end is `Done` or `close()`. Product multi-turn adapters must not treat every turn result as session `Done`.

## Wait and expect

Both match predicates on the event stream. Prefer **waits** for tools and hooks; **expect** for text markers. Timeouts come from launch options (`timeout_ms`) or per-predicate overrides.

See [Waiting on the stream](waits.md).

## Capabilities

Each adapter advertises what its **driver** implements (launch, multi-turn, tools, ACP, …). Calling something that is not implemented fails closed with a clear error — for example interactive `approve` when the adapter has no mid-flight control path.

```bash
medon adapters
```

Full table: [Capability matrix](matrix.md).

## Where product quirks live

Binary discovery, argv, frame parse, and encode stay in the **adapter**. Product-specific knobs go in `LaunchOptions.extra` / the Rhai launch map (`provider`, `acp`, `chat_history_file`, …) — not new `Session` methods per vendor.

Design detail: [Architecture](architecture.md). Catalog: [Adapters](adapters/index.md).
