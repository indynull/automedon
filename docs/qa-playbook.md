# Smoke checklist (product CLIs)

How to check that Automedon can still drive a coding-agent CLI you care about.
Automedon is **alpha**: treat failures as possible adapter bugs, not only product bugs.

Automedon does not replace the product CLI or log you in. It spawns that binary,
normalizes the stream, and lets a script wait and assert multi-turn behavior.

## First install (about 15 minutes)

```bash
# 1. Install
git clone https://github.com/indynull/automedon.git
cd automedon
cargo install --path crates/automedon-cli   # binary: automedon
export PATH="$HOME/.cargo/bin:$PATH"
automedon adapters

# 2. Product you already use (after that CLI alone accepts a prompt)
which pi && automedon run examples/harnesses/pi_workspace.rhai --print
which grok && automedon run examples/harnesses/grok_workspace.rhai --print
# or:
# automedon run examples/harnesses/claude.rhai --print
```

If step 2 fails, check product CLI, auth, model, and adapter parse first -- then Automedon.

## Multi-turn pattern

Same shape for every harness script:

1. **One-turn text marker** -- launch, parse, turn end
2. **Two-turn markers** (`AUTOMEDON_T1` then `AUTOMEDON_T2`) -- multi-turn / session resume
3. **Optional tools** -- when the stream exposes tools (see adapter page and harness scripts)

```bash
automedon run examples/harnesses/<adapter>.rhai --print
```

| Adapter | Script | What it checks |
|---------|--------|----------------|
| `pi` | **`pi_workspace.rhai`** | multi-turn workspace tools (live entry) |
| `grok` | **`grok_workspace.rhai`** | multi-turn coding + tools (live entry) |
| `claude` | `claude.rhai` | multi-turn resume |
| `codex` | `codex.rhai` | exec --json + resume |
| `copilot` | `copilot.rhai` | JSONL + resume id |
| `cursor` | `cursor.rhai` | stream-json + resume |
| `gemini` | `gemini.rhai` | stream-json + resume |
| `grok` | `grok.rhai` | streaming-json + resume markers |
| `opencode` | `opencode.rhai` | json + session |
| `pi` | `pi.rhai` | json multi-turn markers |
| `pi` | `pi_tools.rhai` | tools + hooks |
| `aider` | `aider.rhai` | history multi-turn |
| `grok` | `grok_acp.rhai` | ACP long-lived path |

## What to assert

| Check | How |
|-------|-----|
| Launch | Script starts without "binary not found" |
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

For slow models, raise both. For `automedon shot`:

```bash
automedon shot claude "say hi only" --yolo --timeout-ms 120000
```

## Debugging a failed run

```bash
# Full Automedon + child logging
RUST_LOG=automedon=debug automedon run examples/harnesses/<name>.rhai --print

# Confirm the product alone still works
# (use that product's own one-shot flags)
```

| Failure | Likely cause |
|---------|----------------|
| Binary not found | CLI not on `PATH`; set `bin` in launch map |
| Auth / not logged in | Complete **product** login; Automedon never injects tokens |
| Timeout on text | Model paraphrased; marker too strict; raise timeout |
| Empty `session_id` turn 2 | Resume frame not parsed or drained; check adapter notes |
| Capability error | Call needs a feature the adapter does not implement |

## Continuous integration notes

Default suite:

```bash
cargo test --workspace
# or: make check
```

Live product runs on a runner (optional; needs secrets + product CLI):

```bash
automedon run examples/harnesses/<your-adapter>.rhai --print
# or: AUTOMEDON_LIVE_<ADAPTER>=1 cargo test -p automedon --test live_harness -- --ignored
```

A live multi-turn script is the bar for "this CLI still drives." Unit tests may use an
internal mock adapter; that is not product proof.

## Where to read next

| Need | Page |
|------|------|
| Install detail | [Getting started](getting-started.md) |
| Adapter flags | [Adapters](adapters/index.md) |
| Script language | [Rhai scripts](rhai.md) |
| Wait patterns | [Waiting on the stream](waits.md) |
| Failure table | [Troubleshooting](troubleshooting.md) |
