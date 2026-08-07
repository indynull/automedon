# Rust API

Crate: **`automedon`**. Async on Tokio.

Add to a binary crate:

```toml
automedon = { path = "..." }  # or crates.io when published
tokio = { version = "1", features = ["full"] }
```

## Multi-turn session

Needs a product CLI (example: Grok Build on `PATH` + login):

```rust
use automedon::{Expect, Session};
use std::time::Duration;

#[tokio::main]
async fn main() -> automedon::Result<()> {
    let mut s = Session::builder("grok")
        .yolo(true)
        .timeout(Duration::from_secs(180))
        .build()?;

    s.prompt("Reply with exactly: AUTOMEDON_T1 and nothing else")
        .await?;
    s.expect(Expect::text("AUTOMEDON_T1").timeout(Duration::from_secs(120)))
        .await?;
    s.await_turn().await?;

    s.prompt("Reply with exactly: AUTOMEDON_T2 and nothing else")
        .await?;
    s.expect(Expect::text("AUTOMEDON_T2").timeout(Duration::from_secs(120)))
        .await?;
    s.close().await?;
    Ok(())
}
```

## Builder options

```rust
Session::builder("grok")
    .yolo(true)
    .model("...")
    .cwd("/path/to/workspace")
    .bin("/custom/path/to/binary")
    .extra("acp", serde_json::json!(true))
    .extra("provider", serde_json::json!("..."))
    .timeout(std::time::Duration::from_secs(180))
    .build()?;
```

| Method | Maps to |
|--------|---------|
| `.yolo(bool)` | Preflight allow-all flags |
| `.model(s)` | Model id |
| `.cwd(path)` | Child working directory |
| `.bin(path)` | Binary override |
| `.timeout(Duration)` | Default wait/expect timeout |
| `.extra(key, Value)` | Adapter-specific knobs |
| `.opts(LaunchOptions)` | Replace the whole options struct |

## Session methods

| Method | Role |
|--------|------|
| `prompt` / `run` | User turn; `run` also awaits the turn and returns text |
| `expect` / `wait` / `wait_for` | Stream predicates |
| `await_turn` | Drain until turn end |
| `approve` / `deny` | Interactive permission (capability-gated) |
| `approve_plan` / `reject_plan` | Plan control (capability-gated) |
| `drain_until_done` / `close` | Session end |
| `text` / `turn_text` / `thinking` | Transcript slices |
| `session_id` / `harness` / `turn` / `is_finished` | State |
| `transcript` / `capabilities` | Inspection |

## Waits and content asserts (stream only)

```rust
use automedon::{Expect, Wait};
use std::time::Duration;

s.wait(Wait::hook("PreToolUse")).await?;
s.wait(Wait::tool("bash")).await?;
// Tool write payload (code path / body) and tool result text from the harness stream:
s.expect(Expect::tool_input("write_file", "fn main")).await?;
s.expect(Expect::tool_result_contains("bash", "PASS")).await?;
s.wait(Wait::any([Wait::permission(), Wait::text("DONE")])).await?;
s.wait(Wait::text("ok").timeout(Duration::from_secs(120))).await?;
s.await_turn().await?;
```

Also: `Expect::tool_result_error`, `Expect::text_regex`, `Wait::tool_input`,
`Wait::tool_result_contains`. On-disk file checks stay outside Automedon.

## One-shot helper

```rust
let result = automedon::run("grok", "say hi only", opts).await?;
println!("{}", result.text);
```

## Resolve adapters

```rust
let adapter = automedon::resolve("grok")?;
let caps = adapter.capabilities();
```

Same capability rules as Rhai. See [How it works](concepts.md) and [Capability matrix](matrix.md).
