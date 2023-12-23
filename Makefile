FILES=$(shell find examples/ -name "*.tiny" -print)

all:
	make format
	make run
	make test

build:
	cargo build

run:
	@cargo run -p tiny-cli -- $(FILES)

run-clip:
	wl-paste | DUMP=1 cargo run -p tiny-cli

inter:
	cp history-sample.txt history.txt
	DUMP=1 cargo run -p tiny-cli

test:
	cargo test --all

format:
	cargo fmt

clean:
	cargo clean

.PHONY: all build run run-clip test format
