set quiet

default:
  just --list

[no-quiet]
setup:
	echo "TODO(setup)"

[no-quiet]
clean:
	cargo clean

build:
	cargo build

run_server: build
	RUST_LOG=debug ./target/debug/tuitter --server

run_client USER: build
	RUST_LOG=debug ./target/debug/tuitter --user {{USER}}

test:
	cargo test

lint:
	cargo clippy

lint_fix:
	cargo clippy --fix

fmt:
	cargo fmt --check

fmt_fix:
	cargo fmt
