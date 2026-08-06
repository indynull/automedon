# Adapter × capability matrix (1.0)

Statuses (GOAL contract only): **done** · **unsupported** · **blocked-by-vendor**

Rule: if the harness supports multi-turn, the specialized adapter **must** implement multi-turn. Advertise `capabilities.*` only when **live-proven**.

Live tests: `AUTOMEDON_LIVE_<ADAPTER>=1 cargo test -p automedon --test live_harness -- --ignored --nocapture`.

Evidence on implementer host (2026-08-06): inventory + probes under goal scratch; live logs for grok/pi/aider/copilot/acp.

## Multi-turn mechanism (by harness)

| Adapter | Multi-turn mechanism (driver) | Live-proven |
|---------|-------------------------------|-------------|
| `grok` | `--resume <sessionId>`; ACP long-lived process | **yes** (headless + ACP) |
| `pi` | `--session-id` / `--continue` | **yes** (xAI multi-turn) |
| `aider` | `--chat-history-file` + `--restore-chat-history` | **yes** (xAI multi-turn) |
| `copilot` | `--resume <id>` from **stderr** Resume footer → SessionInfo; ACP prepare | **yes** (headless multi-turn; session id live) |
| `claude` | `--resume` / `--continue` (stream-json) | **blocked-by-vendor** live; prepare+parse ready (system init → SessionInfo) |
| `codex` | `exec resume <thread_id>` + `--json` | **blocked-by-vendor** live (401); prepare+parse ready (`thread.started`) |
| `opencode` | `--session` / `--continue` | **blocked-by-vendor** live; prepare+parse ready (`step_start` sessionID) |
| `cursor` | `--resume` / `--continue` (`agent`/`cursor-agent`) | **blocked-by-vendor** live; prepare+parse ready |
| `gemini` | `-r` / resume latest | **blocked-by-vendor** live (IneligibleTier); prepare+parse ready |

## Feature matrix (live)

| Adapter | Launch | Multi-turn | Stream tools | Wait tools | Perm preflight | Sessions | ACP | Notes |
|---------|--------|------------|--------------|------------|----------------|----------|-----|-------|
| `grok` | **done** | **done** | **done** (ACP) | **done** (ACP) | **done** | **done** | **done** | Native xAI; interactive mid-flight permission/plan not live |
| `pi` | **done** | **done** (xAI) | **done** (json stream) | **done** | **done** | **done** | unsupported | hooks: Pre/PostToolUse from tool lifecycle; `extra.extension` |
| `aider` | **done** | **done** (xAI history) | unsupported | unsupported | **done** | **done** (history path) | unsupported | no agent tool stream on message path |
| `copilot` | **done** | **done** | unsupported (not proven) | unsupported | **done** | **done** | prepare only | stderr Resume footer → SessionInfo → turn-2 `--resume`; ACP not live-proven |
| `claude` | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | unsupported | live auth fail; driver: stream-json, resume/continue, SessionInfo/hooks/tools parse |
| `codex` | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | live 401; driver: exec --json, resume, thread/item parse, ACP prepare |
| `opencode` | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | live provider gap; driver: run --format json, session/continue, step_start parse, ACP |
| `cursor` | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | unsupported | blocked-by-vendor | unsupported | live login required; driver: agent -p stream-json, resume/continue |
| `gemini` | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | blocked-by-vendor | IneligibleTier; driver: -o stream-json, -r, agy preferred when present |

Interactive permission / plan / goals: **mock-proven**; live mid-flight control incomplete on product harnesses (honest **unsupported** until control channel proven).

**Hooks wait:** general API complete; **live** on Pi (tool lifecycle → PreToolUse/PostToolUse); partial on Grok ACP; mock full.

## Runtime capabilities (honest bits)

| Adapter | Advertised true only when live-proven |
|---------|----------------------------------------|
| `grok` | launch, multi_turn, stream_tools, sessions, acp, yolo/preflight |
| `pi` | launch, multi_turn, stream_tools, wait_hooks, sessions, yolo/preflight |
| `aider` | launch, multi_turn, sessions, yolo/preflight |
| `copilot` | launch, multi_turn, sessions, yolo/preflight |
| `claude` / `codex` / `opencode` / `cursor` / `gemini` | **all false** until live |

## xAI-friendly host path

| Harness | How | Status |
|---------|-----|--------|
| Grok | native | **done** multi-turn + ACP tools |
| Pi | `provider=xai` model `grok-4.5` | **done** multi-turn + tools/hooks |
| Aider | `xai/grok-*` + key/bearer | **done** multi-turn |
| OpenCode | connect xAI provider | not logged in / blocked |

## Infrastructure

| Name | Role |
|------|------|
| `mock` | Unit tests only |
| `generic` | Escape hatch |

## Probe reasons (one-liners)

| Adapter | Reason |
|---------|--------|
| claude | authentication_failed — Not logged in · Please run /login |
| codex | OpenAI API 401 Unauthorized (missing bearer) |
| opencode | live probe timeout EXIT 124; no working provider session |
| cursor | Authentication required — `agent login` or `CURSOR_API_KEY` |
| gemini | IneligibleTierError — free tier / client no longer supported |
| copilot | (not blocked) — live launch + multi-turn green |
| grok / pi / aider | (not blocked) — live green under xAI |
