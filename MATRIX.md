# Adapter capability matrix

What each specialized driver **implements** in Automedon (prepare / parse / control),
not a log of who had which login on which machine.

**Rule:** if the product supports multi-turn, the adapter must implement multi-turn.

| Value | Meaning |
|-------|---------|
| **yes** | Driver implements this surface |
| **no** | Product or driver does not expose this path (call fails closed with a clear error) |
| **optional** | Available when you set the launch extra (e.g. `acp: true`) |

## Multi-turn mechanism

| Adapter | Multi-turn mechanism | Sessions |
|---------|----------------------|----------|
| `grok` | `--resume <sessionId>`; ACP long-lived process | yes |
| `pi` | `--session-id` / `--continue` | yes |
| `aider` | `--chat-history-file` + `--restore-chat-history` | yes (history path) |
| `copilot` | `--resume <id>` from Resume footer → SessionInfo | yes |
| `claude` | `--resume` / `--continue` (stream-json) | yes |
| `codex` | `exec resume <thread_id>` + `--json` | yes |
| `opencode` | `--session` / `--continue` | yes |
| `cursor` | `--resume` / `--continue` | yes |
| `gemini` | `-r` / resume latest | yes |

## Feature surface

| Adapter | Launch | Multi-turn | Stream tools | Wait hooks | Perm preflight | Sessions | ACP | Notes |
|---------|--------|------------|--------------|------------|----------------|----------|-----|-------|
| `grok` | yes | yes | yes (ACP path) | partial (ACP) | yes | yes | optional | Headless streaming-json; interactive mid-flight permission/plan not implemented |
| `pi` | yes | yes | yes | yes | yes | yes | no | Tool lifecycle → Pre/PostToolUse; `provider` / `model` / `extension` extras |
| `aider` | yes | yes | no | no | yes | yes | no | Message path has no agent tool stream |
| `copilot` | yes | yes | no | no | yes | yes | optional | Resume footer → SessionInfo |
| `claude` | yes | yes | yes | yes | yes | yes | no | stream-json; hooks/tools parse |
| `codex` | yes | yes | yes | no | yes | yes | optional | `exec --json`; thread/item parse |
| `opencode` | yes | yes | yes | no | yes | yes | optional | `run --format json` |
| `cursor` | yes | yes | yes | no | yes | yes | no | agent stream-json |
| `gemini` | yes | yes | yes | no | yes | yes | optional | Prefers `agy` when present |

Interactive permission / plan / goals mid-flight: **mock** only unless listed above.

## Operator requirements

Every product adapter needs:

1. That product’s CLI on `PATH` (or `bin` / `binary` override)
2. That product’s normal authentication / provider configuration

Optional knobs (`provider`, `model`, `acp`, …) are launch options — see each adapter page in the handbook.

## Infrastructure

| Name | Role |
|------|------|
| `mock` | Unit tests and offline examples only |
| `generic` | Escape hatch: arbitrary `opts.bin` |

## Live tests (developers)

Optional ignored integration tests gate on env vars when you want to exercise a real binary:

```bash
AUTOMEDON_LIVE_GROK=1 cargo test -p automedon --test live_harness -- --ignored --nocapture
```

These are not product status cells. See `crates/automedon/tests/live_harness.rs`.
