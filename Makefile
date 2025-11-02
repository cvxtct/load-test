all: test

test-core:
	cargo test -p rstress-core

test-core-verbose:
	cargo test -p rstress-core -- --nocapture

test-cli:
	cargo test -p rstress

bench-core:
	cargo bench -p rstress-core

network-test:
	cargo test -p rstress-core --features net-tests -- --ignored --nocapture

linting:
	cargo clippy


clean:
	cargo clean

test: test-core test-cli

.PHONY: all test test-core test-core-verbose test-cli bench-core clean