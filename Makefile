RUST_FILES := $(shell find . -type f -name '*.rs')

.PHONY: all

all: target/debug target/release

target/debug/%: $(RUST_FILES)
	cargo build -j 8 --bin $*

target/release/%: $(RUST_FILES)
	cargo build -j 8 --release --bin $*

target/debug: target/debug/wip

target/release: target/release/wip

