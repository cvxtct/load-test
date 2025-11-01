# Default target
all: test

# --- Test targets ---

test-core:
	cargo test -p rstress-core

test-core-verbose:
	cargo test -p rstress-core -- --nocapture

test-cli:
	cargo test -p rstress

# --- Benchmark target ---

bench-core:
	cargo bench -p rstress-core

network-test:
	cargo test -p rstress-core --features net-tests -- --ignored --nocapture

# --- Helpers ---

clean:
	cargo clean

# Run all tests at once
test: test-core test-cli

.PHONY: all test test-core test-core-verbose test-cli bench-core clean