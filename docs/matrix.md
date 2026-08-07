# Adapter capability matrix

What each specialized driver **implements** in Automedon (prepare / parse / control).

**Rule:** if the product supports multi-turn, the adapter must implement multi-turn.

| Value | Meaning |
|-------|---------|
| **yes** | Driver implements this surface |
| **no** | Product or driver does not expose this path (fails closed) |
| **optional** | Available when you set a launch extra (e.g. `acp: true`) |

## Multi-turn mechanism

| Adapter | Multi-turn mechanism | Sessions |
|---------|----------------------|----------|
| `grok` | `--resume <id>` / `--continue`; ACP long-lived process | yes |
| `pi` | `--session-id` / `--continue` | yes |
| `aider` | `--chat-history-file` + `--restore-chat-history` | yes (history path) |
| `copilot` | `--resume=<id>` / `--continue`; JSONL `result.sessionId` | yes |
| `claude` | `--resume` / `--continue` (stream-json) | yes |
| `codex` | `exec resume <id\|--last> --json` | yes |
| `opencode` | `--session` / `--continue` | yes |
| `cursor` | `--resume` / `--continue` (stream-json) | yes |
| `gemini` | `-r` / resume latest | yes |

## Feature surface

| Adapter | Launch | Multi-turn | Stream tools | Wait hooks | Perm preflight | Sessions | ACP | Notes |
|---------|--------|------------|--------------|------------|----------------|----------|-----|-------|
| `grok` | yes | yes | yes | partial | yes | yes | optional | Headless `streaming-json`; `--continue` when no id; ACP via `grok agent stdio` |
| `pi` | yes | yes | yes | yes | yes | yes | no | `--mode json`; tool lifecycle -> Pre/PostToolUse |
| `aider` | yes | yes | no | no | yes | yes | no | Message path has no agent tool stream |
| `copilot` | yes | yes | yes | no | yes | yes | optional | Default `--output-format json`; text/tools/session from JSONL |
| `claude` | yes | yes | yes | yes | yes | yes | no | `--include-hook-events` with stream-json |
| `codex` | yes | yes | yes | no | yes | yes | optional | `exec --json` / `exec resume`; thread/item frames |
| `opencode` | yes | yes | yes | no | yes | yes | optional | `run --format json`; sessionID on frames |
| `cursor` | yes | yes | yes | no | yes | yes | no | `cursor-agent`/`agent` print + stream-json |
| `gemini` | yes | yes | yes | no | yes | yes | optional | `-o stream-json`; prefers `agy` when present |

Interactive mid-flight permission / plan encode: **mock** only (no product control channel implemented).

## Operator requirements

1. Product CLI on `PATH` (or `bin` / `binary` override)
2. That product's normal authentication

Optional knobs: see handbook adapter pages.

## Infrastructure

| Name | Role |
|------|------|
| `mock` | Internal test fixture only (not a public operator path) |
| `generic` | Escape hatch |

## Live tests (developers)

```bash
AUTOMEDON_LIVE_GROK=1 cargo test -p automedon --test live_harness -- --ignored --nocapture
```
