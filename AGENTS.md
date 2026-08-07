# Automedon — agent notes

Library and CLI to drive local AI coding harness processes.

**1.0 product goal:** [GOAL.md](GOAL.md) — specialized drivers for Tier A/B harnesses; shared drive/assert API + per-harness adapters ([docs/architecture.md](docs/architecture.md)). Status surface: [MATRIX.md](MATRIX.md). Mock never counts as product delivery.

Handbook (mdBook): `docs/` + `book.toml` → `make book`. CI: `.github/workflows/`.

## Layout

- `crates/automedon` — library: events, expect, session, adapters, Rhai DSL
- `crates/automedon-cli` — `medon` binary (`run`, `eval`, `shot`, `adapters`)
- `docs/` — handbook source (GitHub Pages)
- `examples/mock/` — offline mock scripts
- `examples/harnesses/` — product CLI scripts
- `.github/workflows/` — ci, pages, release

## Gates

```bash
make check   # fmt + clippy -D warnings + test + llvm-cov fail-under on crate automedon
make book    # mdbook build (needs mdbook installed)
```

Greenfield bar: line coverage of `crates/automedon` (see `make coverage`). `COVERAGE_FAIL_UNDER` defaults to **96** and ratchets toward 100—do not lower it to hide gaps; add tests instead.

## Design rules

Full model: [docs/architecture.md](docs/architecture.md).

- **General drive/assert API** (`Session`, `Wait`, `Expect`, `Event`) for abstract concepts all agent harnesses share: launch, multi-turn, text/thinking, tools, turn end, sessions, permissions, plan, goals, hooks, errors, ACP when capable.
- **Specialized adapters only** for argv, parse, encode, and quirks (`adapter/{grok,pi,...}.rs`). No harness types in Session.
- **Harness-specific knobs** via `LaunchOptions.extra` / Rhai maps — not new Session methods per product.
- **Capabilities** list what the driver implements; missing features fail closed. Mock may advertise a full matrix offline.
- Multi-turn: if the harness supports it, the adapter **must** implement it.
- Prefer structured headless streams over PTY scraping.
- Docs describe operators and drivers — not implementer probe diaries or host-only auth stories.
