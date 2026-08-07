# Adapters

Each product harness has a specialized adapter: binary discovery, argv (or ACP), parse, optional encode.

| Id | Live multi-turn (typical xAI/host) | Example |
|----|--------------------------------------|---------|
| `grok` | yes | [grok](grok.md) |
| `pi` | yes (xAI) | [pi](pi.md) |
| `aider` | yes (xAI) | [aider](aider.md) |
| `copilot` | yes | [copilot](copilot.md) |
| `claude` | needs Anthropic login | [claude](claude.md) |
| `codex` | needs OpenAI auth | [codex](codex.md) |
| `opencode` | needs provider login | [opencode](opencode.md) |
| `cursor` | needs Cursor login | [cursor](cursor.md) |
| `gemini` | often tier-blocked | [gemini](gemini.md) |
| `mock` | offline only | — |
| `generic` | escape hatch (`bin`) | — |

Full status: [matrix.md](../matrix.md).

## Launch extras (common)

Product-specific keys go in the Rhai launch map / `LaunchOptions.extra`:

| Key | Used by |
|-----|---------|
| `acp` | grok, codex, opencode, copilot, gemini (prepare) |
| `provider` | pi |
| `chat_history_file` | aider |
| `extension` / `extensions` | pi |
| `binary` | cursor, gemini (override) |

## Pattern

```rust
let s = launch("grok", #{ yolo: true, timeout_ms: 180_000 });
s.prompt("Reply with exactly: AUTOMEDON_T1");
s.expect(timeout_ms(text("AUTOMEDON_T1"), 120_000));
s.await_turn();
s.prompt("Reply with exactly: AUTOMEDON_T2");
s.expect(timeout_ms(text("AUTOMEDON_T2"), 120_000));
s.close();
```
