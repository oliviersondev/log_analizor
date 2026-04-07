.PHONY: help check fmt clippy test test-one run clean

help:
	@printf "Targets:\n"
	@printf "  make check      - cargo check\n"
	@printf "  make fmt        - cargo fmt --all\n"
	@printf "  make clippy     - cargo clippy --all-targets --all-features\n"
	@printf "  make test       - cargo test\n"
	@printf "  make test-one   - cargo test <name> -- --exact (use TEST=...)\n"
	@printf "  make run        - cargo run\n"
	@printf "  make clean      - cargo clean\n"

check:
	cargo check

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets --all-features

test:
	cargo test

test-one:
	@test -n "$(TEST)" || (echo "Usage: make test-one TEST=test_name" && exit 1)
	cargo test $(TEST) -- --exact

run:
	cargo run

clean:
	cargo clean
