# Adapters

An **adapter** is the only place that knows a product’s binary name, flags, and stream shape. Everything else in Automedon speaks in shared events and waits.

## Choosing a harness

Install the product CLI, complete **its** login, then point `launch("…")` at the adapter id.

| Id | CLI on `PATH` | Multi-turn | First script |
|----|---------------|------------|--------------|
| `grok` | `grok` | `--resume`; optional ACP | [Grok](grok.md) · `examples/harnesses/grok.rhai` |
| `pi` | `pi` | session id / continue | [Pi](pi.md) · `examples/harnesses/pi.rhai` |
| `aider` | `aider` | chat-history restore | [Aider](aider.md) |
| `copilot` | `copilot` | resume footer | [Copilot](copilot.md) |
| `claude` | `claude` | resume / continue | [Claude](claude.md) |
| `codex` | `codex` | exec resume | [Codex](codex.md) |
| `opencode` | `opencode` | session / continue | [OpenCode](opencode.md) |
| `cursor` | `agent` / `cursor-agent` / `cursor` | resume / continue | [Cursor](cursor.md) |
| `gemini` | `gemini` or `agy` | `-r` resume | [Gemini](gemini.md) |
| `mock` | (none) | scenarios | offline only — `examples/mock/` |
| `generic` | your `bin` | process exit | escape hatch |

Expand a product page in the sidebar for launch options. Runtime flags: `medon adapters`. Full matrix: [Capability matrix](../matrix.md).

## Launch extras (shared map)

Pass product knobs in the Rhai map / `LaunchOptions.extra`:

| Key | Typical use |
|-----|-------------|
| `acp` | Long-lived stdio agent path (Grok, Codex, OpenCode, Copilot, Gemini when supported) |
| `provider` | Pi provider id |
| `model` | Model id (`LaunchOptions.model` or extra) |
| `chat_history_file` | Aider history path |
| `extension` / `extensions` | Pi extensions |
| `binary` | Override binary for Cursor / Gemini |
| `yolo` | Map to the product’s allow-all / skip-permission flags when the adapter knows them |

## Minimal multi-turn pattern

```rust
let s = launch("grok", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
s.prompt("Reply with exactly: AUTOMEDON_T1");
s.expect(timeout_ms(text("AUTOMEDON_T1"), 120_000));
s.await_turn();
s.prompt("Reply with exactly: AUTOMEDON_T2");
s.expect(timeout_ms(text("AUTOMEDON_T2"), 120_000));
s.close();
```

Every product script under `examples/harnesses/` follows that shape. See [Examples](../examples.md).
