# How it works

## Mental model

Automedon sits **outside** the agent. Your script (or Rust code) drives a **session**. Each turn, an **adapter** turns your prompt into a process spawn (or synthetic events for mock), and maps stdout / ACP frames into a shared **event** stream. **Wait** and **expect** block on that stream until a condition matches.

```
Script / Rust
     │
     ▼
  Session ── prepare ──► child CLI (or mock)
     │                        │
     │◄──── events ───────────┘
     │
  Wait / Expect
     │
  Transcript + session id
```

## Session

A session holds:

- Which adapter is active  
- The event transcript (text, tools, hooks, …)  
- Multi-turn context (turn index, resume / session id)  
- Process supervision (kill-on-drop when a child is running)

You normally open one session per harness conversation, call `prompt` multiple times, then `close`.

## Events

| Event | Meaning |
|-------|---------|
| `TextDelta` / `ThinkingDelta` | Assistant text / thinking chunks |
| `ToolCall` / `ToolResult` | Model tool use |
| `HookStarted` / `HookFinished` | Lifecycle around tools or turns |
| `TurnComplete` | This **turn** finished |
| `ProcessExit` | Child process exited |
| `SessionInfo` | Resume / session id from the product |
| `PermissionRequest` | Mid-flight permission (when supported) |
| `PlanPresented` / plan resolved | Plan mode (when supported) |
| `Done` | **Session** finished |
| `Error` | Harness or adapter error |
| `Raw` | Unparsed line (still in the transcript) |

## Multi-turn shapes

**Process-per-turn** — common for headless CLIs. Each `prompt` may spawn a new process with resume or history flags. Continuity depends on capturing `SessionInfo` (or a history file path) before the next turn.

**Long-lived process** — e.g. Grok with `acp: true`. One process, many turns over stdin/stdout.

Turn end: `TurnComplete` and/or `ProcessExit`. Session end: `Done` or `close()`. Product multi-turn adapters must not map every turn result to session `Done`.

## Wait vs expect

Both match predicates on the event stream with a timeout.

- Prefer **wait** constructors for tools and hooks (`Wait::tool`, `wait_hook_started` in Rhai).  
- Prefer **expect** for text markers and simple assertions.  
- Call **`await_turn()`** before the next `prompt` if the turn may still be streaming.

See [Waiting on the stream](waits.md).

## Capabilities

Each adapter advertises what its **driver implements**. Missing features fail closed (for example interactive `approve` when there is no mid-flight control path).

```bash
medon adapters
```

Full table: [Capability matrix](matrix.md).

## Where product quirks live

Binary discovery, argv, frame parse, and encode stay in the **adapter**. Product-specific knobs go in `LaunchOptions.extra` / the Rhai launch map — not new `Session` methods per vendor.

Deeper design: [Architecture](architecture.md). Catalog: [Adapters](adapters/index.md).
