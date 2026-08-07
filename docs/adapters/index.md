# Adapters

Each product harness has a specialized adapter: binary discovery, argv (or ACP), parse, optional encode.

| Id | Binary | Multi-turn | Page |
|----|--------|------------|------|
| `grok` | `grok` | `--resume`; optional ACP | [grok](grok.md) |
| `pi` | `pi` | session id / continue | [pi](pi.md) |
| `aider` | `aider` | chat-history restore | [aider](aider.md) |
| `copilot` | `copilot` | resume footer | [copilot](copilot.md) |
| `claude` | `claude` | resume / continue | [claude](claude.md) |
| `codex` | `codex` | exec resume | [codex](codex.md) |
| `opencode` | `opencode` | session / continue | [opencode](opencode.md) |
| `cursor` | `agent` / `cursor-agent` / `cursor` | resume / continue | [cursor](cursor.md) |
| `gemini` | `gemini` / `agy` | `-r` resume | [gemini](gemini.md) |
| `mock` | (in-process) | scenarios | offline examples |
| `generic` | `opts.bin` | process exit | escape hatch |

Feature surface: [matrix.md](../matrix.md). Examples: `examples/harnesses/`.

## Launch extras (common)

| Key | Used by |
|-----|---------|
| `acp` | grok, codex, opencode, copilot, gemini |
| `provider` | pi |
| `model` | any adapter that maps `LaunchOptions.model` |
| `chat_history_file` | aider |
| `extension` / `extensions` | pi |
| `binary` | cursor, gemini (override) |

## Pattern

```rust
let s = launch("grok", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
s.prompt("Reply with exactly: AUTOMEDON_T1");
s.expect(timeout_ms(text("AUTOMEDON_T1"), 120_000));
s.await_turn();
s.prompt("Reply with exactly: AUTOMEDON_T2");
s.expect(timeout_ms(text("AUTOMEDON_T2"), 120_000));
s.close();
```
