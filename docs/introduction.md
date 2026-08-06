# Introduction

**Automedon** drives local AI coding-agent CLIs (Grok, Claude, Codex, Pi, and others) through one session and event model.

- **Library:** Rust crate `automedon`
- **CLI:** `medon` (from crate `automedon-cli`)
- **Scripts:** Rhai (`.rhai`) or the Rust API

It does not reimplement agents and is not an LLM-as-judge scorer. It spawns the real harness binary, normalizes its stream into events, and lets you wait and assert.

## What you need

| Use case | Requirements |
|----------|----------------|
| Offline tutorial | Rust toolchain; no API keys |
| Live harness | That product’s CLI on `PATH` + vendor login/credentials |

## Where to go next

1. [Getting started](getting-started.md) — install and first green run  
2. [First script](first-script.md) — multi-turn with the mock harness  
3. [Live harnesses](live.md) — real CLIs  
4. [Adapters](adapters/index.md) — per-product flags and status  

Contract and live status: [goal.md](goal.md), [matrix.md](matrix.md).
