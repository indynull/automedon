# Troubleshooting

## Auth and vendor

| Message / symptom | Action |
|-------------------|--------|
| Claude “Not logged in” | Run product login / set Anthropic credentials |
| Codex 401 | OpenAI / Codex auth |
| Cursor authentication required | `agent login` or `CURSOR_API_KEY` |
| Gemini IneligibleTier | Supported client/tier or Antigravity (`agy`) |
| OpenCode hang / no session | Log in a provider; check `opencode` config |

## Automedon errors

| Error | Meaning |
|-------|---------|
| `script not found` | Bad path to `medon run` |
| `capability not supported on …` | Feature not advertised; do not call approve/plan |
| `SessionFinished` | Session already closed; do not prompt again |
| `ExpectTimeout` | Condition never matched before timeout |
| `HarnessNotFound` | Binary not on `PATH` (or wrong `bin` override) |

## Debug

```bash
RUST_LOG=automedon=debug medon run path/to/script.rhai --print
medon adapters
```

Capture a few lines of the child stream (or run the CLI by hand with the same flags the adapter builds) and compare to the adapter’s `parse_line` expectations.

## Still stuck

1. Confirm offline mock scripts pass.  
2. Confirm the product CLI works outside Automedon.  
3. Check [matrix.md](matrix.md) for known blocked cells.  
4. Open an issue with adapter name, command, and redacted logs.  
