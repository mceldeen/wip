GCC_OPTIONS?=-Wall -Wextra -Wpedantic -Werror -std=c17

.PHONY: all clean

all: build/debug build/release

clean:
	rm -rf build

build/release/%.o: %.c
	mkdir -p build/release
	gcc $(GCC_OPTIONS) -O2 -o $@ -c $<

build/debug/%.o: %.c
	mkdir -p build/debug
	gcc $(GCC_OPTIONS) -g -o $@ -c $<

build/debug: build/debug/wip

build/release: build/release/wip

build/debug/wip: build/debug/main.o build/debug/cJSON.o build/debug/iso8601.o build/debug/op.o build/debug/op_vector.o build/debug/buf_rdr.o
	mkdir -p build/debug
	gcc $(GCC_OPTIONS) -g -o build/debug/wip $^

build/release/wip: build/release/main.o build/release/cJSON.o build/release/iso8601.o build/release/op.o build/release/op_vector.o build/release/buf_rdr.o
	mkdir -p build/release
	gcc $(GCC_OPTIONS) -O2 -o build/release/wip $^
