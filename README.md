# Automedon

Drive local AI coding-agent CLIs through one event model: specialized adapters, a Rust library, and a Rhai scripting DSL.

| | |
|--|--|
| CLI | **`medon`** |
| Docs | Handbook under `docs/` (GitHub Pages after deploy) |
| Status | [MATRIX.md](MATRIX.md) |
| Goal | [GOAL.md](GOAL.md) |
| Architecture | [docs/architecture.md](docs/architecture.md) |

**1.0 goal:** specialized drivers for the agreed harness set; general drive/assert API; live proof only. Mock is test infrastructure only.

## What it is

Coding-agent CLIs each have their own flags and JSON streams. Scripts need one way to spawn, prompt, wait for tools, assert text, approve permissions, and keep multi-turn continuity — without shell soup or a bespoke parser per product.

Automedon:

- Uses a **shared event stream** (`TextDelta`, `ToolCall`, `TurnComplete`, plan/goal/permission, `Done`, …)
- Supports **multi-turn** on one `Session` (Grok `--resume`, Pi `--session-id` / `--continue`, mock history, …)
- Ships **adapters** for product-specific flags and JSON shapes (Grok, Pi, Claude, … plus `generic` and in-process `mock`)
- Exposes a **Rust API** and a **Rhai DSL**
- Runs on **Tokio** with bounded channels and kill-on-drop process supervision

## Quick start

```bash
# Library + CLI (binary: medon)
cargo build -p automedon-cli
cargo install --path crates/automedon-cli   # installs `medon` on PATH

# Live product examples (need CLI + auth — Grok / Pi)
medon run examples/smoke.rhai --print          # grok
medon run examples/multi_turn.rhai --print     # grok multi-turn
medon run examples/wait_hooks.rhai --print     # pi tools + hooks
medon run examples/grok_hello.rhai --print     # grok: write fib + tests

# Per-product multi-turn smokes
medon run examples/live/grok.rhai --print
# also: pi.rhai, aider.rhai, copilot.rhai, …
# see examples/live/README.md

# Offline (mock only — no product binary)
medon run examples/mock/multi_turn.rhai --print
medon shot mock "hello" --scenario echo
```

### Multi-turn (Rhai, Grok)

```rhai
let s = launch("grok", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
s.prompt("Reply with exactly: AUTOMEDON_T1 and nothing else");
s.expect(timeout_ms(text("AUTOMEDON_T1"), 120_000));
s.await_turn();
s.prompt("Reply with exactly: AUTOMEDON_T2 and nothing else");
s.expect(timeout_ms(text("AUTOMEDON_T2"), 120_000));
s.close();
```

### Waits (Rhai, Pi tools/hooks)

```rhai
let s = launch("pi", #{ yolo: true, provider: "xai", model: "grok-4.5", timeout_ms: 180_000 });
s.prompt("Run a shell tool once: echo hi. End with DONE.");
s.wait(timeout_ms(wait_hook_started("PreToolUse"), 120_000));
s.wait(timeout_ms(wait_tool_any(), 120_000));
s.wait(timeout_ms(wait_text("DONE"), 120_000));
```

Rust: `s.wait(Wait::hook("PreToolUse")).await?` or `Wait::any([Wait::permission(), Wait::text("DONE")])`.
`expect(...)` still works and shares the same predicates.

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

One specialized module per product harness. Capability bits and live status: **[MATRIX.md](MATRIX.md)**.

### Tier A (required product drivers)

| Id | Binary | Notes |
|----|--------|--------|
| `claude` | `claude` | Claude Code: `-p` + `stream-json`; `--resume` |
| `codex` | `codex` | OpenAI Codex: `exec --json`; optional ACP via `extra.acp` |
| `gemini` | `gemini` | Gemini CLI: `-p` + `stream-json`; `-r` resume; `--acp`; aliases `antigravity` / `agy` |
| `opencode` | `opencode` | OpenCode: `run --format json`; session flag; ACP via npx path |
| `grok` | `grok` | Grok Build: `streaming-json` + `--resume`; ACP: `extra.acp` → `grok agent stdio` |
| `cursor` | `cursor-agent` / `cursor` | Cursor agent CLI when installed |

### Tier B

| Id | Binary | Notes |
|----|--------|--------|
| `aider` | `aider` | Multi-turn via `--chat-history-file` + `--restore-chat-history`; xAI: `model: "xai/grok-4.5"` + `XAI_API_KEY` / `extra.xai_api_key` |
| `pi` | `pi` | `--mode json`; multi-turn; xAI: `extra.provider: "xai"`, `model: "grok-4.5"` |
| `copilot` | `copilot` | GitHub Copilot CLI agent path when driveable |

### Infrastructure (not product “supported harnesses”)

| Id | Role |
|----|------|
| `mock` | In-process scenarios for unit tests / offline examples only |
| `generic` | Escape hatch: arbitrary `opts.bin` |

Harness-specific knobs go in `LaunchOptions.extra` / Rhai `#{ ... }` (e.g. `max_turns`, `tools`, `acp`, `scenario` for mock only).

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

**Non-goals:** reimplementing each harness TUI; LLM-as-judge scoring; remote cloud-only agents. Structured headless streams and ACP stdio are the fast path; PTY/TUI only as a last resort for a documented gap.

## CLI

Installed binary: **`medon`** (crate `automedon-cli`).

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

### CI, Pages, releases

Workflows in `.github/workflows/`:

- **ci** — fmt, clippy, test, coverage ≥96%, mdbook
- **pages** — publish handbook to GitHub Pages on `main`
- **release** — tag `v*` builds multi-platform `medon` assets

See [docs/ci-and-releases.md](docs/ci-and-releases.md).

### Publish the repo

```bash
# remote (repo: https://github.com/indynull/automedon)
git remote add origin git@github.com:indynull/automedon.git
git push -u origin main
# Settings → Pages → Source: GitHub Actions
# Optional: git tag v0.1.0 && git push origin v0.1.0
```


## License

MIT
