# Automedon

**Production driver for local AI coding-agent CLIs.** One session model, specialized adapters, scripts or Rust — built for teams that need repeatable multi-turn harness checks (including vendor QA of Claude Code, Codex, Grok Build, Copilot, Cursor, Gemini, Pi, Aider, OpenCode, and more).

| | |
|--|--|
| CLI | **`medon`** |
| Docs | [Handbook](https://indynull.github.io/automedon/) · [QA playbook](docs/qa-playbook.md) |
| Capabilities | [MATRIX.md](MATRIX.md) |
| Architecture | [docs/architecture.md](docs/architecture.md) |

## Install

```bash
git clone https://github.com/indynull/automedon.git
cd automedon
cargo install --path crates/automedon-cli   # needs Rust 1.85+
export PATH="$HOME/.cargo/bin:$PATH"
medon adapters
```

## Quick start

```bash
# Offline (no product CLI)
medon run examples/mock/smoke.rhai --print
medon run examples/mock/multi_turn.rhai --print
medon shot mock "hello" --scenario echo

# Your product CLI (after its own login works)
medon run examples/harnesses/claude.rhai --print
medon run examples/harnesses/copilot.rhai --print
# … see examples/harnesses/README.md
```

**Vendor QA:** [docs/qa-playbook.md](docs/qa-playbook.md) — 15-minute first day and daily multi-turn pattern.

### Multi-turn script shape

```rust
let s = launch("claude", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
s.prompt("Reply with exactly: AUTOMEDON_T1 and nothing else");
s.expect(timeout_ms(text("AUTOMEDON_T1"), 120_000));
s.await_turn();
s.prompt("Reply with exactly: AUTOMEDON_T2 and nothing else");
s.expect(timeout_ms(text("AUTOMEDON_T2"), 120_000));
s.close();
```

### Rust API

```rust
use automedon::{Expect, Session};

#[tokio::main]
async fn main() -> automedon::Result<()> {
    let mut s = Session::builder("claude")
        .yolo(true)
        .timeout(std::time::Duration::from_secs(180))
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

| Id | Binary | Multi-turn |
|----|--------|------------|
| `claude` | `claude` | resume / continue, stream-json |
| `codex` | `codex` | `exec --json` + resume |
| `gemini` | `gemini` / `agy` | stream-json + `-r` |
| `opencode` | `opencode` | `run --format json` + session |
| `grok` | `grok` | streaming-json + resume; ACP optional |
| `cursor` | `agent` / `cursor-agent` | stream-json + resume |
| `aider` | `aider` | chat-history restore |
| `pi` | `pi` | json mode + session id |
| `copilot` | `copilot` | JSONL + `--resume=` |
| `mock` | (in-process) | offline only |
| `generic` | `opts.bin` | escape hatch |

```bash
medon adapters
```

Full matrix: [MATRIX.md](MATRIX.md).

## Develop

```bash
make check        # fmt + clippy + test + coverage
make book         # handbook
make book-serve
```

## License

MIT
