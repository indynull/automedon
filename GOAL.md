# Automedon 1.0 product goal

Drive local AI coding-agent CLIs through one event model: specialized adapters, a Rust library, and a Rhai scripting surface. CLI binary: **`medon`**.

## What “done” means

1. **Specialized driver** per agreed product harness (argv, parse, encode, multi-turn).
2. **Shared API:** `Session`, `Wait`, `Expect`, normalized `Event` — no harness types in Session.
3. **Capabilities** describe what the driver implements; missing features fail closed with a clear error.
4. **Examples** under `examples/mock/` (offline) and `examples/harnesses/` (product CLIs).
5. **Docs** tell operators how to install, launch, and multi-turn — not implementer probe diaries.

## Product harness set

### Tier A

| Id | Binary | Multi-turn |
|----|--------|------------|
| `claude` | `claude` | `--resume` / `--continue` |
| `codex` | `codex` | `exec resume` + `--json` |
| `gemini` | `gemini` / `agy` | `-r` resume |
| `opencode` | `opencode` | `--session` / `--continue` |
| `grok` | `grok` | `--resume`; ACP optional |
| `cursor` | `agent` / `cursor-agent` / `cursor` | `--resume` / `--continue` |

### Tier B

| Id | Binary | Multi-turn |
|----|--------|------------|
| `aider` | `aider` | chat-history restore |
| `pi` | `pi` | `--session-id` / `--continue` |
| `copilot` | `copilot` | `--resume` from footer |

### Not product delivery

| Id | Role |
|----|------|
| `mock` | Offline tests / examples |
| `generic` | Escape hatch |

## Architecture contract

See [docs/architecture.md](docs/architecture.md). Per-adapter operator notes: [docs/adapters/](docs/adapters/).

- General drive/assert API for concepts all harnesses share when present.
- Specialized adapters only for binary discovery, argv, parse, encode, quirks.
- Harness-specific knobs via `LaunchOptions.extra` / Rhai maps.
- Multi-turn: if the product multi-turns, the adapter **must** implement it.

## Quality bar

- `make check` — fmt, clippy `-D warnings`, tests, coverage floor on crate `automedon`
- Offline examples and CLI smoke pass without product CLIs
- Live multi-turn against a real CLI when the operator has auth (optional ignored tests)

## Non-goals

- Reimplementing each harness TUI
- LLM-as-judge scoring
- Treating mock success as product delivery
- Remote cloud-only agents with no local CLI
