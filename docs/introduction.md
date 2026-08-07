# Introduction

Automedon is a **driver** for local AI coding-agent CLIs. It does not replace Claude Code, Grok Build, Codex, or Pi — it spawns those tools, normalizes their streams, and gives you one API to wait, assert, and multi-turn.

Use it when you want **scripts or tests** that talk to a real agent the same way every time: launch, prompt, wait for a tool or a marker, resume the next turn, close.

| Piece | Name |
|-------|------|
| Library | Rust crate `automedon` |
| CLI | **`medon`** |
| Scripts | Rhai (`.rhai`) or the Rust API |

## What you get

- **One session model** — `prompt` → events → `expect` / `wait` → `await_turn` → next turn  
- **Specialized adapters** — product-specific argv and JSON parsing live behind one interface  
- **Offline mock** — full multi-turn / tools / hooks without a product CLI  
- **Honest capabilities** — if interactive approve is not implemented, the call fails clearly  

## What you still own

- Installing and authenticating each product CLI  
- Choosing models and provider settings for that product  
- Deciding what “done” means in your script (text markers, tools, exit codes)

## Paths through this handbook

| Goal | Page |
|------|------|
| Install `medon` and run offline | [Getting started](getting-started.md) |
| Learn the script pattern | [Write a script](first-script.md) |
| Understand sessions and events | [How it works](concepts.md) |
| Full CLI reference | [Command line](cli.md) |
| Full scripting surface | [Rhai scripts](rhai.md) |
| Embed from Rust | [Rust API](rust-api.md) |
| Pick a product adapter | [Adapters](adapters/index.md) |
| See feature coverage | [Capability matrix](matrix.md) |
| Copy ready-made scripts | [Examples](examples.md) |

Internals (architecture, continuous integration) are under **Internals** in the sidebar.
