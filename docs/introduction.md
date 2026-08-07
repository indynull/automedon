# Introduction

**Automedon** is a production driver for **local AI coding-agent CLIs**. Teams use it to automate multi-turn checks against the real product binary — the same path a customer script would use — without inventing a bespoke shell harness per vendor.

| Piece | Name |
|-------|------|
| Library | Rust crate `automedon` |
| CLI | **`medon`** |
| Scripts | Rhai (`.rhai`) or the Rust API |

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

`claude` · `codex` · `gemini` · `opencode` · `grok` · `cursor` · `aider` · `pi` · `copilot`

Plus offline `mock` and escape-hatch `generic`. Capability detail: [matrix](matrix.md).

## Paths through the handbook

| Goal | Page |
|------|------|
| Install and offline smoke | [Getting started](getting-started.md) |
| Daily product regression | [Testing your harness (QA)](qa-playbook.md) |
| Sessions, events, multi-turn | [How it works](concepts.md) |
| Full CLI | [Command line](cli.md) |
| Full script surface | [Rhai scripts](rhai.md) |
| Ready-made scripts | [Examples](examples.md) |
