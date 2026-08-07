# Automedon

Drive local AI coding-agent CLIs through one event model: specialized adapters, a Rust library, and a Rhai scripting DSL.

| | |
|--|--|
| CLI | **`medon`** |
| Docs | Handbook under `docs/` ([GitHub Pages](https://indynull.github.io/automedon/)) |
| Capabilities | [MATRIX.md](MATRIX.md) |
| Goal | [GOAL.md](GOAL.md) |
| Architecture | [docs/architecture.md](docs/architecture.md) |

**1.0 goal:** specialized drivers for the agreed harness set; general drive/assert API. Mock is offline test infrastructure only.

## What it is

Coding-agent CLIs each have their own flags and JSON streams. Scripts need one way to spawn, prompt, wait for tools, assert text, approve permissions, and keep multi-turn continuity — without shell soup or a bespoke parser per product.

Automedon:

- Uses a **shared event stream** (`TextDelta`, `ToolCall`, `TurnComplete`, plan/goal/permission, `Done`, …)
- Supports **multi-turn** on one `Session` (resume, session id, chat history, …)
- Ships **adapters** for product-specific flags and JSON shapes (plus `generic` and in-process `mock`)
- Exposes a **Rust API** and a **Rhai DSL**
- Runs on **Tokio** with bounded channels and kill-on-drop process supervision

## Quick start

```bash
cargo build -p automedon-cli
cargo install --path crates/automedon-cli   # installs `medon` on PATH

# Offline (mock — no product CLI)
medon run examples/mock/smoke.rhai --print
medon run examples/mock/multi_turn.rhai --print
medon shot mock "hello" --scenario echo

# Product harnesses (need that CLI + auth)
medon run examples/harnesses/grok.rhai --print
medon run examples/harnesses/pi.rhai --print
# see examples/README.md and examples/harnesses/README.md
```

### Multi-turn (Rhai)

```rust
let s = launch("grok", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
s.prompt("Reply with exactly: AUTOMEDON_T1 and nothing else");
s.expect(timeout_ms(text("AUTOMEDON_T1"), 120_000));
s.await_turn();
s.prompt("Reply with exactly: AUTOMEDON_T2 and nothing else");
s.expect(timeout_ms(text("AUTOMEDON_T2"), 120_000));
s.close();
```

### Waits (Rhai)

```rust
let s = launch("pi", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
s.prompt("Run a shell tool once: echo hi. End with DONE.");
s.wait(timeout_ms(wait_hook_started("PreToolUse"), 120_000));
s.wait(timeout_ms(wait_tool_any(), 120_000));
s.wait(timeout_ms(wait_text("DONE"), 120_000));
```

Rust: `s.wait(Wait::hook("PreToolUse")).await?` or `Wait::any([Wait::permission(), Wait::text("DONE")])`.

### Rust API

```rust
use automedon::{Expect, Session};

#[tokio::main]
async fn main() -> automedon::Result<()> {
    let mut s = Session::builder("grok")
        .yolo(true)
        .timeout(std::time::Duration::from_secs(180))
        .extra("multi_turn", serde_json::json!(true))
        .build()?;
    s.prompt("Reply with exactly: AUTOMEDON_T1 and nothing else").await?;
    s.expect(Expect::text("AUTOMEDON_T1").timeout(std::time::Duration::from_secs(120)))
        .await?;
    s.await_turn().await?;
    s.close().await?;
    Ok(())
}
```

## Adapters

One specialized module per product harness. What each driver implements: **[MATRIX.md](MATRIX.md)**.

### Tier A

| Id | Binary | Notes |
|----|--------|--------|
| `claude` | `claude` | `-p` + stream-json; `--resume` / `--continue` |
| `codex` | `codex` | `exec --json`; optional ACP via `extra.acp` |
| `gemini` | `gemini` / `agy` | stream-json; `-r` resume; optional ACP |
| `opencode` | `opencode` | `run --format json`; session / continue |
| `grok` | `grok` | streaming-json + `--resume`; ACP via `extra.acp` |
| `cursor` | `agent` / `cursor-agent` / `cursor` | stream-json; resume / continue |

### Tier B

| Id | Binary | Notes |
|----|--------|--------|
| `aider` | `aider` | chat-history multi-turn; set `model` for your backend |
| `pi` | `pi` | `--mode json`; multi-turn; optional `provider` / `model` |
| `copilot` | `copilot` | non-interactive path; resume from footer |

### Infrastructure

| Id | Role |
|----|------|
| `mock` | In-process scenarios for tests / offline examples |
| `generic` | Escape hatch: arbitrary `opts.bin` |

```bash
medon adapters
```

## Architecture

```
Script (Rhai) ──► Session ──► Adapter.prepare()
                     │              │
                     │              ├─ process (stdout NDJSON)
                     │              └─ synthetic events (mock)
                     ▼
              Expect engine ◄── normalized Event stream
                     │
                     ▼
                 Transcript (text, tools, usage)
```

**Non-goals:** reimplementing each harness TUI; LLM-as-judge scoring; remote cloud-only agents. Structured headless streams and ACP stdio are the fast path.

## CLI

```
medon run <script.rhai> [--print]
medon eval 'let s = launch("mock"); s.run("hi")'
medon shot <harness> <prompt> [--yolo] [--model M] [--scenario echo]
medon adapters
```

## Develop

```bash
make check        # fmt + clippy + test + coverage (≥96% on crate automedon)
make book         # handbook → book/ (needs mdbook)
make book-serve   # local preview
```

Workflows: continuous integration, Pages, release — see [docs/ci-and-releases.md](docs/ci-and-releases.md).

## License

MIT
