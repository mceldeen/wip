RUST_FILES := $(shell find . -type f -name '*.rs')

target/debug/wip: $(RUST_FILES)
	cargo build -j 8

target/release/wip: $(RUST_FILES)
	cargo build -j 8 --release