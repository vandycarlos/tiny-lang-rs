
FILES=$(shell find examples/ -name "*.tiny" -print)

all: run format check test

build:
	cargo build

run:
	@cargo run --quiet -p tiny-cli -- $(FILES)

run-clip:
	wl-paste | DUMP=1 cargo run -p tiny-cli

inter:
	cp history-sample.txt history.txt
	DUMP=1 cargo run -p tiny-cli

test:
	cargo test --workspace

format:
	cargo fmt --all --check

check:
	cargo clippy --workspace -- -D warnings
	cargo check --all

clean:
	cargo clean

.PHONY: all build run run-clip inter test format check clean
