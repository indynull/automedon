# Introduction

**Automedon** is a production driver for **local AI coding-agent CLIs**. Teams use it to automate multi-turn checks against the real product binary — the same path a customer script would use — without inventing a bespoke shell harness per vendor.

| Piece | What |
|-------|------|
| Library | Rust crate `automedon` |
| CLI | **`medon`** |
| Scripts | [Rhai scripts](rhai.md) (`.rhai`) or the [Rust API](rust-api.md) |
| Examples | [Ready-made scripts](examples.md) under `examples/mock/` and `examples/harnesses/` |

It spawns the product CLI, normalizes its stream into events, and blocks until waits/expects match. It does **not** reimplement agents, score answers with an LLM judge, or perform product login for you.

## Who this is for

| Audience | Start here |
|----------|------------|
| Vendor QA / harness engineers | [Testing your harness (QA)](qa-playbook.md) |
| First install | [Getting started](getting-started.md) |
| Writing automation | [Write a script](first-script.md) |
| Embedding in a test binary | [Rust API](rust-api.md) |
| Per-product flags | [Adapters](adapters/index.md) |

## Supported product adapters

| Adapter | Docs | Daily multi-turn smoke |
|---------|------|------------------------|
| `claude` | [Claude Code](adapters/claude.md) | [`examples/harnesses/claude.rhai`](examples.md) |
| `codex` | [OpenAI Codex](adapters/codex.md) | [`examples/harnesses/codex.rhai`](examples.md) |
| `gemini` | [Gemini CLI](adapters/gemini.md) | [`examples/harnesses/gemini.rhai`](examples.md) |
| `opencode` | [OpenCode](adapters/opencode.md) | [`examples/harnesses/opencode.rhai`](examples.md) |
| `grok` | [Grok Build](adapters/grok.md) | [`examples/harnesses/grok.rhai`](examples.md) |
| `cursor` | [Cursor agent](adapters/cursor.md) | [`examples/harnesses/cursor.rhai`](examples.md) |
| `aider` | [Aider](adapters/aider.md) | [`examples/harnesses/aider.rhai`](examples.md) |
| `pi` | [Pi](adapters/pi.md) | [`examples/harnesses/pi.rhai`](examples.md) |
| `copilot` | [GitHub Copilot CLI](adapters/copilot.md) | [`examples/harnesses/copilot.rhai`](examples.md) |

Also: offline [mock](examples.md) and escape-hatch `generic`. Capability detail: [matrix](matrix.md).

## Paths through the handbook

| Goal | Page |
|------|------|
| Install and offline smoke | [Getting started](getting-started.md) |
| Daily product regression | [Testing your harness (QA)](qa-playbook.md) |
| Sessions, events, multi-turn | [How it works](concepts.md) |
| Full CLI | [Command line](cli.md) |
| Full script surface | [Rhai scripts](rhai.md) |
| Ready-made scripts | [Examples](examples.md) |
