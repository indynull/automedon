# Testing your harness (vendor QA)

This page is for engineers who ship a coding-agent CLI (Claude Code, Codex, Grok Build, Copilot, Cursor, Gemini, Pi, Aider, OpenCode, …) and want a **repeatable daily check** that Automedon can still drive it.

Automedon does **not** replace your CLI or log you in. It spawns **your** binary, normalizes the stream, and lets scripts wait/assert multi-turn behavior the same way every morning.

## 15-minute first day

```bash
# 1. Install
git clone https://github.com/indynull/automedon.git
cd automedon
cargo install --path crates/automedon-cli   # binary: medon
export PATH="$HOME/.cargo/bin:$PATH"

# 2. Offline only (no product CLI)
medon adapters
medon run examples/mock/smoke.rhai --print
medon run examples/mock/multi_turn.rhai --print

# 3. Your product (example: claude)
which claude && claude -p "hi" --output-format text   # prove login outside Automedon
medon run examples/harnesses/claude.rhai --print
```

If step 2 fails, the problem is Automedon or Rust. If step 2 works and step 3 fails, the problem is almost always product CLI, auth, or model — not Rhai syntax.

## Daily regression pattern

Use the same script shape for every harness:

1. **One-turn text marker** — proves launch + parse + turn end  
2. **Two-turn markers** (`AUTOMEDON_T1` → `AUTOMEDON_T2`) — proves multi-turn / session resume  
3. **Optional tools** — when your stream exposes tools (see MATRIX)

Ready-made scripts:

```bash
medon run examples/harnesses/<adapter>.rhai --print
```

| Adapter | Script | What it proves |
|---------|--------|----------------|
| `claude` | `claude.rhai` | multi-turn resume |
| `codex` | `codex.rhai` | exec --json + resume |
| `copilot` | `copilot.rhai` | JSONL + resume id |
| `cursor` | `cursor.rhai` | stream-json + resume |
| `gemini` | `gemini.rhai` | stream-json + resume |
| `grok` | `grok.rhai` | streaming-json + resume |
| `opencode` | `opencode.rhai` | json + session |
| `pi` | `pi.rhai` | json multi-turn |
| `pi` | `pi_tools.rhai` | tools + hooks |
| `aider` | `aider.rhai` | history multi-turn |
| `grok` | `grok_acp.rhai` | ACP long-lived path |

## What to assert

| Check | How |
|-------|-----|
| Launch | Script starts without “binary not found” |
| Text stream | `expect(text("AUTOMEDON_T1"))` |
| Turn boundary | `await_turn()` returns |
| Session id | `print(s.session_id())` after turn 1 (non-empty preferred) |
| Multi-turn | Turn 2 sees `AUTOMEDON_T2`; transcript still has `AUTOMEDON_T1` |
| Tools (if any) | `wait(wait_tool_any())` or named tool |
| Hooks (if any) | `wait_hook_started("PreToolUse")` |

## Timeouts

Product model latency is not instant. Examples use:

- launch default `timeout_ms: 180_000` (3 minutes)
- expects at `120_000` for markers

For slow models, raise both. For `medon shot`:

```bash
medon shot claude "say hi only" --yolo --timeout-ms 120000
```

## Debugging a failed morning run

```bash
# Full Automedon + child logging
RUST_LOG=automedon=debug medon run examples/harnesses/<name>.rhai --print

# Confirm the product alone still works
# (use that product's own one-shot flags)
```

| Failure | Likely cause |
|---------|----------------|
| Binary not found | CLI not on `PATH`; set `bin` in launch map |
| Auth / not logged in | Complete **product** login; Automedon never injects tokens |
| Timeout on text | Model paraphrased; marker too strict; raise timeout |
| Empty `session_id` turn 2 | Resume frame not parsed or drained; check adapter notes |
| Capability error | Call requires a surface the driver does not implement |

## Embedding in your CI

Offline (always green without secrets):

```bash
cargo test --workspace
medon run examples/mock/multi_turn.rhai --print
```

Live (optional, needs secrets + product CLI on the runner):

```bash
medon run examples/harnesses/<your-adapter>.rhai --print
# or: AUTOMEDON_LIVE_<ADAPTER>=1 cargo test -p automedon --test live_harness -- --ignored
```

Do not treat mock success as product delivery. Treat live multi-turn scripts as the bar for “our CLI still drives cleanly.”

## Where to read next

| Need | Page |
|------|------|
| Install detail | [Getting started](getting-started.md) |
| Your adapter flags | [Adapters](adapters/index.md) → product page |
| Capability columns | [Capability matrix](matrix.md) |
| Script language | [Rhai scripts](rhai.md) |
| Wait patterns | [Waiting on the stream](waits.md) |
| Failure table | [Troubleshooting](troubleshooting.md) |
