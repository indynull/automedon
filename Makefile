.PHONY: check test fmt clippy build coverage book book-serve

# Greenfield bar: drive line coverage of crates/automedon to complete.
# Floor ratchets up; do not lower it to hide gaps—add tests.
COVERAGE_FAIL_UNDER ?= 96

# book is separate: install mdbook for `make book` / CI pages job
check: fmt clippy test coverage

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

# Line coverage of crates/automedon only (CLI covered by integration tests separately).
coverage:
	cargo llvm-cov --package automedon --fail-under-lines $(COVERAGE_FAIL_UNDER) --summary-only

build:
	cargo build --workspace --release

# Handbook (mdBook). Install: cargo install mdbook --version 0.4.40
book:
	mdbook build

book-serve:
	mdbook serve --open
