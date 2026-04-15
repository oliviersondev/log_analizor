.PHONY: help check fmt clippy test test-one run run-demo run-ui build-ui clean

help:
	@printf "Targets:\n"
	@printf "  make check      - cargo check\n"
	@printf "  make fmt        - cargo fmt --all\n"
	@printf "  make clippy     - cargo clippy --all-targets --all-features\n"
	@printf "  make test       - cargo test\n"
	@printf "  make test-one   - cargo test <name> -- --exact (use TEST=...)\n"
	@printf "  make run        - cargo run (or with LOG='...')\n"
	@printf "  make run-demo   - cargo run --bin demo\n"
	@printf "  make run-ui     - cargo run --bin ui --features ui\n"
	@printf "  make build-ui   - cargo build --bin ui --features ui\n"
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
	@if [ -n '$(LOG)' ]; then cargo run --bin log_analizor -- --log '$(LOG)'; else cargo run --bin log_analizor; fi

run-demo:
	cargo run --bin demo

run-ui:
	cargo run --bin ui --features ui

build-ui:
	cargo build --bin ui --features ui

clean:
	cargo clean
