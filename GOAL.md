# GOAL: 100% specialized implementation for agreed harnesses

**Product:** Automedon drives local AI coding-agent CLIs through a shared session and event API. It is not an eval scorer and not a mock demo.

**Architecture (locked):** [docs/architecture.md](docs/architecture.md)

- **General** drive/assert API: `Session`, `Wait`, `Expect`, normalized `Event`, Rhai.
- **Specialized** adapters: one module per product harness.
- Harness-specific knobs only via `LaunchOptions.extra` / Rhai maps — never one-off Session methods per product.
- **Capabilities** advertise only **live-proven** bits on product adapters; unsupported calls fail closed.

**This goal is complete when every agreed harness is fully specialized for everything that harness actually exposes, with live evidence, honest capability bits, and MATRIX cells that are never silent `in-progress`.**

---

## Agreed harness set (nine product adapters)

Same selection as before: 2026 adoption, driveable CLI, agent-class product. All are first-class agent harnesses for Automedon: **multi-turn is required whenever the product supports continuity**.

| Adapter id | Product / binary | Control path (specialized) | Multi-turn mechanism (required if product has it) |
|------------|------------------|----------------------------|---------------------------------------------------|
| `claude` | Claude Code (`claude`) | `-p` + `stream-json`; hooks config + stream | `--resume <id>` (+ Agent SDK / ACP when used) |
| `codex` | OpenAI Codex CLI (`codex`) | `exec --json`; ACP when available | `codex exec resume <id>` (or documented equivalent) |
| `opencode` | OpenCode (`opencode`) | `run --format json`; `opencode acp` | `--session` / `--continue` |
| `grok` | Grok Build (`grok`) | `streaming-json` + `--resume`; **ACP** `grok agent stdio` | `--resume <sessionId>`; ACP `session/prompt` on one process |
| `cursor` | Cursor agent CLI (`cursor-agent` / `agent`) | `-p` + `stream-json` | `--resume` / `--continue` |
| `gemini` | Gemini CLI and/or Antigravity (`gemini` / `agy`) | headless stream-json / successor binary | `-r` / `--resume` as live binary documents |
| `aider` | Aider (`aider`) | `--message` (+ model/provider flags) | `--chat-history-file` + `--restore-chat-history` |
| `pi` | Pi (`pi`) | `--mode json`; extensions; provider/model | `--session-id` / `--continue` |
| `copilot` | GitHub Copilot CLI (`copilot`) | `-p` non-interactive; ACP preferred | `--resume` / `--continue`; prefer ACP long-lived |

### Explicitly not product surface

| Name | Role |
|------|------|
| `mock` | Test infrastructure only — never in “supported harnesses” |
| `generic` | Escape hatch (`opts.bin`) only |
| Tier C (Cline, Goose, Windsurf, …) | After this goal is green |

---

## What “100% implementation” means

**Not** “scaffolding exists.” **Not** “prepare builds argv.” **Not** “unit tests with fixtures.”

For **each** of the nine adapters, 100% means:

### 1. Specialized driver complete

| Piece | Done means |
|-------|------------|
| Discover | Binary resolution (PATH / documented locations / aliases) works |
| Prepare | Correct argv or ACP spawn for first turn **and** multi-turn when product multi-turns |
| Parse | Real frames from the live harness normalize to `Event` (text, tools, turn end, hooks, permissions, plan, goals as emitted) |
| Encode | Mid-flight control (`approve` / `deny` / plan) only when harness has a control channel; else capability false |
| Capabilities | Runtime bits true **only** for live-proven cells |

### 2. Every abstract capability the harness has is classified

| Status | Meaning |
|--------|---------|
| **done** | Live wait/drive works; evidence log recorded; capability bit true; MATRIX cell `done` |
| **unsupported** | Live attempt + docs show no surface (or no side channel); bit false; one-line MATRIX reason |
| **blocked-by-vendor** | Binary missing / auth / vendor kill-switch **after recorded live attempt**; bit false; probe log path |
| **in-progress** | Temporary only while actively implementing; **forbidden at goal freeze** |

No silent gaps. No “we’ll parse it later.” No mock success as product delivery.

### 3. General API only for abstract concepts

Scripts use the same `Session` / `Wait` / `Expect` everywhere. Quirks stay in the adapter + `extra`.

### 4. MATRIX.md is the public truth

Statuses only: `done` | `unsupported` | `blocked-by-vendor`. Zero `in-progress` at tag.

---

## Abstract concepts → general API (must normalize when harness exposes them)

| Concept | Drive | Assert / wait | Events |
|---------|-------|---------------|--------|
| Launch | builder / `launch` | process up | `Spawned` |
| Multi-turn | repeated `prompt` | continuity on turn 2+ | `TurnStart`, `SessionInfo` |
| Text / thinking | — | `Wait::text`, thinking | `TextDelta`, `ThinkingDelta` |
| Tools | allow/deny flags when native | `Wait::tool*` | `ToolCall`, `ToolResult` |
| Turn end | `await_turn` | turn complete / done | `TurnComplete`, `Done`, `ProcessExit` |
| Sessions | resume/fork via adapter | session id known | `SessionInfo` |
| Permissions | preflight yolo; mid-flight approve/deny | `Wait::permission` | `PermissionRequest` / resolved |
| Plan | `approve_plan` / `reject_plan` | `Wait::plan*` | `PlanPresented` / resolved |
| Goals | start/observe when native | goal waits | Goal* events |
| Hooks | configure via `extra` when harness needs files/extensions | `Wait::hook*` | `HookStarted`, `HookFinished` |
| ACP | long-lived stdio when `extra.acp` + cap | same waits on mapped events | Session ACP + adapter parse |
| Errors | — | structured failure | `Error`, non-zero exit |

### Multi-turn rule (non-negotiable)

- If the **product** supports multi-turn / session continuity, the **adapter must implement it**.
- Do **not** invent multi-turn for a product with no continuity mechanism.
- Aider is an agent harness: multi-turn via chat-history restore is **required**, not optional.

### Hooks rule (abstract)

Hooks = lifecycle interception (pre/post tool, session start/stop, extension events). Vendors name them differently; adapters normalize to `HookStarted` / `HookFinished` (and tool events when tools are on the stream).

| Harness examples | Native shape | Normalize to |
|------------------|--------------|--------------|
| Claude Code | PreToolUse / PostToolUse / … | Hook* + Tool* |
| Pi | Extensions + JSON tool lifecycle (`toolcall_*`, …) | Hook* + Tool* |
| Grok | ACP / streaming hook notifications when emitted | Hook* |
| Others | whatever the binary documents | same or honest `unsupported` |

Harness-only config (extension paths, hook files) stays in **`extra`**.

### Specialized-only knobs (`extra`, not Session methods)

| Knob | Via |
|------|-----|
| Pi provider / model / extensions | `extra.provider`, `model`, `extra.extension(s)` |
| Aider history file | `extra.chat_history_file` |
| ACP mode | `extra.acp` |
| yolo / allow-all | `LaunchOptions.yolo` → adapter-native flags |
| tool allowlists | `extra.tools` / adapter-native flags |

---

## Capability matrix every adapter must fill

For each adapter, every row is **done**, **unsupported**, or **blocked-by-vendor** (with evidence). Track in [MATRIX.md](MATRIX.md).

| Capability | Done requires (live) |
|------------|----------------------|
| Launch | Spawn headless / structured stream; process supervised |
| Multi-turn | Second prompt sees continuity (session id, resume, ACP, or history restore) |
| Stream text | `Wait::text` (or thinking) against real deltas |
| Stream tools | `Wait::tool` / tool result against real tool frames (if product emits tools) |
| Wait tools | Same, via general Wait API |
| Hooks | `Wait::hook*` when lifecycle is on the wire or normalized from tool lifecycle |
| Permission preflight | yolo / allow-all maps to native flags when product has them |
| Permission interactive | mid-flight approve/deny only if product has a control channel |
| Plan / goals | drive/observe only if product exposes them |
| Sessions | session id / resume path works |
| ACP | full multi-turn (+ tools, + permissions if exposed) on long-lived stdio when product has ACP |

**Minimum bar for “adapter done”:** Launch + Multi-turn (if product multi-turns) + Stream text + Turn end, all live-proven, **plus** every other row classified with evidence. Tools/hooks/ACP/permissions classified for that product’s real surface — not left blank.

---

## Acceptance criteria (goal complete when all true)

1. **Nine specialized modules** registered (`claude`, `codex`, `opencode`, `grok`, `cursor`, `gemini`, `aider`, `pi`, `copilot`); no mega-adapter; mock/generic not product.
2. **Multi-turn implemented** for every product that multi-turns; live multi-turn **done** when binary+auth available, else **blocked-by-vendor** with probe log (never silent skip).
3. **Parse completeness:** live frames for text, tools, turn end, hooks, permissions, plan/goals as the harness emits them — unit fixtures for parse only after real frames captured.
4. **Stream + waits live** where the harness emits the corresponding events.
5. **Permissions / plan / goals:** live drive or observe when exposed; otherwise **unsupported** with reason — never silent mock success on product adapters.
6. **ACP:** every product with an ACP surface has a specialized ACP path; at least one of Claude / Grok / OpenCode / Copilot fully live multi-turn + tools (+ permissions if exposed).
7. **Capability honesty:** runtime bits match MATRIX; `approve` / plan APIs fail closed without the bit.
8. **Live tests:** `AUTOMEDON_LIVE_<ADAPTER>=1` (and variants `_XAI`, `_ACP`, tools) per product adapter; skip cleanly if binary/auth missing — skip is not **done**.
9. **Docs:** this GOAL + ARCHITECTURE + MATRIX + README product table (mock excluded).
10. **Quality:** `make check` with library line coverage fail-under ≥ **96%** (ratchet toward 100%; never lower fail-under to hide gaps).
11. **MATRIX freeze:** no `in-progress` cells; every cell has status + one-line reason when not `done`.

When all of the above hold → tag **1.0**.

---

## Evidence rules

| Counts as **done** | Does **not** count |
|--------------------|--------------------|
| Live test log with multi-turn / tool / hook / ACP wait | Mock scenario only |
| Recorded real NDJSON/ACP frames used to unit-test **parse** | Parse fixture alone for product “supported” |
| Probe log: binary missing, auth required, or vendor error | Silent skip claimed as success |

Env gates (extend as needed):

`AUTOMEDON_LIVE_GROK`, `AUTOMEDON_LIVE_GROK_ACP`, `AUTOMEDON_LIVE_PI`, `AUTOMEDON_LIVE_PI_XAI`, `AUTOMEDON_LIVE_PI_XAI_TOOLS`, `AUTOMEDON_LIVE_AIDER_XAI`, `AUTOMEDON_LIVE_CLAUDE`, `AUTOMEDON_LIVE_CODEX`, `AUTOMEDON_LIVE_OPENCODE`, `AUTOMEDON_LIVE_CURSOR`, `AUTOMEDON_LIVE_COPILOT`, `AUTOMEDON_LIVE_GEMINI`, …

---

## Definition of blocked-by-vendor

Only after a **recorded live attempt**:

1. No headless / stream / ACP control for that feature, **and**
2. No documented file / env / config side channel, **and**
3. Capability stays false with a one-line MATRIX reason + probe artifact.

“We didn’t reverse-engineer the stream” is **not** blocked-by-vendor.  
“Host has no Anthropic/OpenAI/GitHub/Cursor login” **is** blocked-by-vendor for **live** cells after a failed probe — prepare/parse work must still be finished from captured frames or public docs so the driver is ready when auth appears.

---

## Delivery order (execution plan)

Phases may parallelize when binaries and credentials exist.

| Phase | Outcome | Done check |
|-------|---------|------------|
| **P0** | Inventory all nine: binary on PATH? multi-turn mechanism? hook surface? auth? MATRIX skeleton rows | inventory notes + MATRIX rows exist |
| **P1** | Architecture freeze already in ARCHITECTURE; capability fail-closed on Session | unit tests for require_cap / fail-closed |
| **P2** | **Grok** full matrix (reference driver): multi-turn, tools, ACP, hooks/plan/permission as binary emits | live logs + capability bits |
| **P3** | **Pi** + **Aider** full matrix for what they expose (xAI path allowed) | live multi-turn (+ Pi tools/hooks) |
| **P4** | **Claude** + **Codex** full specialized matrix (auth-dependent live; prepare/parse complete either way) | live or blocked-by-vendor + parse tests from real frames |
| **P5** | **OpenCode** + **Cursor** + **Copilot** + **Gemini**/Antigravity full matrix | same bar as P4 |
| **P6** | MATRIX freeze, coverage ≥96%, examples/live packaging, **1.0 tag** | `make check` green; MATRIX no `in-progress` |

**Proof order on an xAI-only machine:** Grok → Pi → Aider first (live **done**). Remaining six stay **blocked-by-vendor** for live cells only after probes; implementation (prepare/parse/encode/live test scaffolding) still finishes to 100% readiness.

---

## Per-adapter 100% checklist

Copy into MATRIX or work notes; each box is binary.

### Shared for every product adapter

- [ ] Specialized `prepare` first turn
- [ ] Specialized multi-turn prepare (if product multi-turns)
- [ ] `parse_line` covers real text / tools / turn / hooks / permissions as emitted
- [ ] `capabilities()` matches live MATRIX only
- [ ] Live test env gate + clean skip without binary/auth
- [ ] Live launch + text wait **or** blocked-by-vendor probe
- [ ] Live multi-turn **or** blocked-by-vendor probe (if product multi-turns)
- [ ] Live tools/hooks/ACP/permissions rows classified with evidence

### Adapter-specific musts

| Adapter | Musts beyond shared |
|---------|---------------------|
| `grok` | Headless multi-turn + ACP multi-turn + tools; hooks/plan/permission as emitted |
| `pi` | session-id/continue multi-turn; json tool lifecycle → Tool* + Hook* (Pre/PostToolUse); `extra.extension` |
| `aider` | chat-history restore multi-turn; honest **unsupported** for tool stream if product has none |
| `claude` | stream-json parse; resume multi-turn; hooks normalization when on wire |
| `codex` | exec json parse; resume; ACP path when available |
| `opencode` | run json; session/continue; ACP path |
| `cursor` | stream-json; resume/continue |
| `copilot` | non-interactive + resume; ACP preferred for multi-turn |
| `gemini` | stream/resume on live binary or Antigravity alias; vendor blocks recorded |

---

## Non-negotiables for implementers

1. Live harness or it did not ship as **done**.
2. Specialized adapter per product — no shared mega-adapter for product ids.
3. Abstract concepts → general API; quirks → adapter + `extra`.
4. Multi-turn if the product multi-turns (including Aider history restore).
5. Hooks normalized when the product has lifecycle interception.
6. Mock never in product “supported” lists or MATRIX product rows.
7. Prefer ACP / structured streams over TUI scraping.
8. Fail closed on missing capabilities.
9. Do not lower coverage fail-under to hide gaps; add tests.

---

## Out of scope for this goal

- Cloning each harness TUI
- Cloud SaaS agents without a local driveable CLI
- LLM-as-judge / eval scoring product
- Tier C harnesses
- Browser automation of agent web UIs

---

## Status tracking

- **Contract (this file):** what 100% means; does not drift with partial progress.
- **Progress:** [MATRIX.md](MATRIX.md) only — update cells as work lands.
- **Design detail:** [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

When MATRIX has no `in-progress`, every acceptance criterion holds, and `make check` is green → **1.0**.
