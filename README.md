# Automedon

Library and CLI (`automedon`) to spawn local AI coding-agent CLIs, normalize their streams into events, and wait until expects match.

| | |
|--|--|
| CLI | **`automedon`** |
| Docs | [Handbook](https://indynull.github.io/automedon/) / [Smoke checklist](docs/qa-playbook.md) |
| Architecture | [docs/architecture.md](docs/architecture.md) |

## Install

```bash
git clone https://github.com/indynull/automedon.git
cd automedon
cargo install --path crates/automedon-cli   # needs Rust 1.85+
export PATH="$HOME/.cargo/bin:$PATH"
automedon adapters
```

## Quick start

Pick a product CLI you already use, confirm it works alone, then run a harness script:

```bash
# After `pi` / `grok` / `claude` / ... accepts a one-shot prompt on its own
automedon run examples/harnesses/pi_workspace.rhai --print
automedon run examples/harnesses/grok_workspace.rhai --print
automedon run examples/harnesses/claude.rhai --print

automedon shot claude "say hi only" --yolo --timeout-ms 120000
```

More adapters: [examples/harnesses/](examples/harnesses/). Multi-turn pattern: [docs/qa-playbook.md](docs/qa-playbook.md).

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
| `cursor` | `cursor-agent` | stream-json + resume |
| `aider` | `aider` | chat-history restore |
| `pi` | `pi` | json mode + session id |
| `copilot` | `copilot` | JSONL + `--resume=` |
| `generic` | `opts.bin` | escape hatch |

```bash
automedon adapters
```

Per-adapter notes: [docs/adapters/](docs/adapters/). Smoke scripts: [examples/harnesses/](examples/harnesses/).

## Develop

```bash
make check        # fmt + clippy + test + coverage
make book         # handbook
make book-serve
```

In-tree continuous integration uses a private `mock` adapter and fixtures under
`examples/mock/` -- those are not the public getting-started path.

## License

MIT
