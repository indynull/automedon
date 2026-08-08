# Introduction

**Automedon** is a Rust library and CLI (`automedon`) that spawns local AI coding-agent CLIs, turns their stdout into events, and blocks until your expects and waits match.

| Piece | What |
|-------|------|
| Library | Rust crate `automedon` |
| CLI | **`automedon`** |
| Scripts | [Rhai scripts](rhai.md) (`.rhai`) or the [Rust API](rust-api.md) |
| Examples | [Product harness scripts](examples.md) under `examples/harnesses/` |

It does not reimplement agents, score answers with a model judge, or log you into product CLIs. You need the product binary on `PATH` and that product's own login.

## Start here

| Goal | Page |
|------|------|
| Install and first product run | [Getting started](getting-started.md) |
| Multi-turn checks against a product CLI | [Smoke checklist](qa-playbook.md) |
| Write a script | [Write a script](first-script.md) |
| Call from Rust | [Rust API](rust-api.md) |
| Per-product flags | [Adapters](adapters/index.md) |

## Product adapters

| Adapter | Docs | Example |
|---------|------|---------|
| `claude` | [Claude Code](adapters/claude.md) | [`examples/harnesses/claude.rhai`](examples.md) |
| `codex` | [OpenAI Codex](adapters/codex.md) | [`examples/harnesses/codex.rhai`](examples.md) |
| `gemini` | [Gemini CLI](adapters/gemini.md) | [`examples/harnesses/gemini.rhai`](examples.md) |
| `opencode` | [OpenCode](adapters/opencode.md) | [`examples/harnesses/opencode.rhai`](examples.md) |
| `grok` | [Grok Build](adapters/grok.md) | [`examples/harnesses/grok_workspace.rhai`](examples.md) |
| `cursor` | [Cursor agent](adapters/cursor.md) | [`examples/harnesses/cursor.rhai`](examples.md) |
| `aider` | [Aider](adapters/aider.md) | [`examples/harnesses/aider.rhai`](examples.md) |
| `pi` | [Pi](adapters/pi.md) | [`examples/harnesses/pi_workspace.rhai`](examples.md) |
| `copilot` | [GitHub Copilot CLI](adapters/copilot.md) | [`examples/harnesses/copilot.rhai`](examples.md) |

Escape hatch: `generic` (you supply the binary). What each adapter implements: [Adapters](adapters/index.md) and `examples/harnesses/`.

## Handbook map

| Goal | Page |
|------|------|
| Install and first product run | [Getting started](getting-started.md) |
| Product multi-turn checks | [Smoke checklist](qa-playbook.md) |
| Sessions, events, multi-turn | [How it works](concepts.md) |
| CLI flags | [Command line](cli.md) |
| Script language | [Rhai scripts](rhai.md) |
| Example index | [Examples](examples.md) |
