# Architecture

## Premise

Coding agent CLIs differ in flags and stream formats. They share the same job: run an agent loop over a workspace. Automedon controls those CLIs from the outside; it does not reimplement the agents.

- **General drive/assert API** — one `Session`, `Wait`, `Expect`, normalized `Event` for concepts all harnesses share (when present).
- **Specialized adapters** — one module per product; only place for binary discovery, argv, frame parse, encode, and quirks.
- **Capabilities** — product adapters advertise features the driver implements; calls that need a missing feature fail with a clear error.

## Abstract concepts (general API)

These map into `Event` + `Session` / `Wait` / `Expect` / Rhai. Adapters **must** normalize into these when the harness supports them.

| Concept | Drive | Assert / wait | Normalized events |
|---------|-------|---------------|-------------------|
| **Launch** | `Session::builder` / `launch` | process starts | `Spawned` |
| **Multi-turn** | repeated `prompt` on one session | continuity | `TurnStart`, session id |
| **Text / thinking** | (observe) | `Wait::text`, thinking | `TextDelta`, `ThinkingDelta` |
| **Tools** | tool filter via opts when supported | `Wait::tool`, `tool_result` | `ToolCall`, `ToolResult` |
| **Turn boundary** | `await_turn` | `Wait::turn_complete` | `TurnComplete`, `Done` |
| **Sessions** | resume via adapter | `session_info` | `SessionInfo` |
| **Permissions** | preflight `yolo`; mid-flight `approve`/`deny` | `Wait::permission` | `PermissionRequest`, `PermissionResolved` |
| **Plan** | `approve_plan` / `reject_plan` | `Wait::plan` | `PlanPresented`, `PlanResolved` |
| **Goals** | (observe / drive when supported) | goal waits | `GoalStarted` / progress / complete |
| **Hooks** | configure only when harness uses files/flags | `Wait::hook*` | `HookStarted`, `HookFinished` |
| **ACP** | long-lived stdio client when `extra.acp` | same waits on mapped events | via adapter + session ACP path |
| **Errors** | — | expect error / exit | `Error`, `ProcessExit` |

**Hooks (abstract):** any lifecycle interception point around tools or turns (pre-tool, post-tool, session start/stop, …). Harnesses name these differently (Claude `PreToolUse`, Pi extension `tool_call`, Grok ACP hook_execution). Adapters **normalize** to `HookStarted`/`HookFinished` with:

- `name` — stable product name when possible (`PreToolUse` / `PostToolUse`) or clear harness name  
- `phase` — optional native phase/id  
- `detail` — tool name, input, raw payload  

General scripts wait with `Wait::hook("PreToolUse")` or `Wait::tool("bash")` (tools and hooks are related but distinct: tool = model tool use; hook = policy/lifecycle around it).

## Specialized only (not general API)

Keep out of `Session` surface unless almost universal. Prefer `LaunchOptions.extra` / Rhai map, or adapter-local helpers:

| Example | Where |
|---------|--------|
| Aider `chat_history_file` | `extra.chat_history_file` |
| Pi `--provider` / `-e` extensions | `extra.provider`, `extra.extension` / `extra.extensions` |
| Grok ACP | `extra.acp` + Session ACP path (general *if* capabilities.acp) |
| Codex dangerous-bypass flag | adapter yolo mapping |
| OpenCode `--auto` | adapter yolo mapping |
| Model id string | `LaunchOptions.model` (general) |

## Adapter contract

```
prepare(prompt, opts, turn_ctx) → argv / ACP spawn / synthetic events + capabilities
parse_line(line) → Vec<Event>   // only normalized events (or Raw)
encode_permission / encode_plan  // optional mid-flight control
```

No harness-specific types leak into `Session`.

## Capability honesty

Product adapters advertise bits for control paths they implement. Mock may advertise a full matrix for offline tests. Scripts that call `approve` / `approve_plan` without the bit get a clear error: `capability not supported on <harness>: permissions_interactive`.

## Non-goals

- Reimplementing each TUI  
- LLM-as-judge scoring  
- Treating mock success as product delivery  

Product goal checklist (maintainers): repository root [`GOAL.md`](https://github.com/indynull/automedon/blob/main/GOAL.md).
