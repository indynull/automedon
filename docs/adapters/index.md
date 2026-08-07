# Adapters

An **adapter** is the only place that knows a product’s binary name, flags, and stream shape. Everything else speaks shared events and waits.

List what this build implements:

```bash
medon adapters
```

## Product catalog

| Id | Default binary | Multi-turn | Daily smoke |
|----|----------------|------------|-------------|
| `claude` | `claude` | `--resume` / `--continue` | [Claude](claude.md) · `examples/harnesses/claude.rhai` |
| `codex` | `codex` | `exec resume` | [Codex](codex.md) · `examples/harnesses/codex.rhai` |
| `gemini` | `gemini` / `agy` | `-r` resume | [Gemini](gemini.md) · `examples/harnesses/gemini.rhai` |
| `opencode` | `opencode` | `--session` / `--continue` | [OpenCode](opencode.md) · `examples/harnesses/opencode.rhai` |
| `grok` | `grok` | `--resume` / `--continue` | [Grok](grok.md) · `examples/harnesses/grok.rhai` |
| `cursor` | `agent` / `cursor-agent` / `cursor` | `--resume` / `--continue` | [Cursor](cursor.md) · `examples/harnesses/cursor.rhai` |
| `aider` | `aider` | chat-history restore | [Aider](aider.md) · `examples/harnesses/aider.rhai` |
| `pi` | `pi` | `--session-id` / `--continue` | [Pi](pi.md) · `examples/harnesses/pi.rhai` |
| `copilot` | `copilot` | `--resume=` / `--continue` | [Copilot](copilot.md) · `examples/harnesses/copilot.rhai` |
| `mock` | (in-process) | scenarios | `examples/mock/` |
| `generic` | `opts.bin` | process exit | escape hatch |

Feature columns: [Capability matrix](../matrix.md). Vendor QA workflow: [Testing your harness (QA)](../qa-playbook.md).

## Launch extras

| Key | Typical use |
|-----|-------------|
| `yolo` | Map to product allow-all / skip-permission flags |
| `model` | Model id |
| `timeout_ms` | Default wait/expect timeout (ms) |
| `cwd` | Child working directory |
| `bin` / `binary` | Override binary path |
| `acp` | Long-lived ACP / agent stdio path when the product supports it |
| `provider` | Pi provider id |
| `chat_history_file` | Aider history path |
| `extension` / `extensions` | Pi extensions |
| `scenario` | Mock only |

## Minimal multi-turn pattern

```rust
let s = launch("claude", #{ yolo: true, multi_turn: true, timeout_ms: 180_000 });
s.prompt("Reply with exactly: AUTOMEDON_T1 and nothing else");
s.expect(timeout_ms(text("AUTOMEDON_T1"), 120_000));
s.await_turn();
print(s.session_id());
s.prompt("Reply with exactly: AUTOMEDON_T2 and nothing else");
s.expect(timeout_ms(text("AUTOMEDON_T2"), 120_000));
s.await_turn();
s.close();
```

Every `examples/harnesses/*.rhai` multi-turn smoke follows this shape.
