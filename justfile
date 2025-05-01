set quiet

PROTO_IN := "./blackjack.proto"
PROTO_OUT_DIR := "./bjack-api/proto"
PROTO_OUT := "./bjack-api/proto/blackjack.pb.go ./bjack-api/proto/blackjack_grpc.pb.go"
API_DIR := "./bjack-api"
API_EXECUTABLE := "./bin/bjack-api"
UI_DIR := "./bjack-ui"
CI_IMAGE_TAG := "0.0.5"

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

run_server:
	RUST_LOG=debug ./target/debug/tuitter --server

run_client USER:
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
