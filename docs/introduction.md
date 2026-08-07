# Introduction

**Automedon** is a library and CLI that drives **local** AI coding-agent programs (Grok Build, Claude Code, Codex, Pi, and others) through one session model.

You write a short script or a few lines of Rust. Automedon spawns the real product CLI, normalizes its stream into events, and lets you wait and assert — multi-turn, tools, hooks, permissions — without a bespoke shell pipeline per vendor.

| Piece | Name |
|-------|------|
| Library | crate `automedon` |
| CLI | **`medon`** |
| Scripts | Rhai (`.rhai`) or the Rust API |

It does **not** reimplement agents, score answers with an LLM judge, or replace the product’s own login.

## Paths through this handbook

| If you want to… | Start here |
|-----------------|------------|
| Install and run something offline | [Getting started](getting-started.md) |
| Learn the script pattern | [Write a script](first-script.md) |
| Understand sessions, events, multi-turn | [How it works](concepts.md) |
| Drive a specific product CLI | [Adapters](adapters/index.md) |
| See what each driver supports | [Capability matrix](matrix.md) |
| Copy a ready-made example | [Examples](examples.md) |

Internals (architecture, continuous integration) live under **Internals** in the sidebar — optional unless you are changing the tree.
