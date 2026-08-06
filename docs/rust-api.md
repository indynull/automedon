# Rust API

Crate: `automedon`.

```rust
use automedon::{Expect, Session};

#[tokio::main]
async fn main() -> automedon::Result<()> {
    let mut s = Session::builder("mock")
        .extra("scenario", serde_json::json!("multi"))
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    s.prompt("alpha").await?;
    s.expect(Expect::text("T1:alpha")).await?;
    s.await_turn().await?;

    s.prompt("beta").await?;
    s.expect(Expect::text("prior=T1:alpha")).await?;
    s.close().await?;
    Ok(())
}
```

## Builder

```rust
Session::builder("grok")
    .yolo(true)
    .model("…")
    .cwd("/path")
    .extra("acp", serde_json::json!(true))
    .timeout(std::time::Duration::from_secs(180))
    .build()?;
```

## Waits

```rust
use automedon::Wait;

s.wait(Wait::hook("PreToolUse")).await?;
s.wait(Wait::tool("bash")).await?;
s.wait(Wait::any([Wait::permission(), Wait::text("DONE")])).await?;
```

## One-shot helper

```rust
let result = automedon::run("mock", "hello", opts).await?;
println!("{}", result.text);
```

Same event and capability rules as Rhai. See [Concepts](concepts.md).
