GO_FILES := $(shell find . -type f -name '*.go')
GO_DIRS :=  $(shell find . -type f -name '*.go' | sed -r 's|/[^/]+$$||' | sort -u)
GO_CMDS := $(shell  find ./cmd -maxdepth 1 -mindepth 1 -type d)
GO_RUNS := $(patsubst ./cmd/%,run/%,$(GO_CMDS))
GO_BUILDS := $(patsubst ./cmd/%,build/%,$(GO_CMDS))
GO_TESTS := $(patsubst ./%,test/%,$(GO_DIRS))

build/%: $(GO_FILES)
	go build -o $@ ./cmd/$*/...

.PHONY: all $(GO_RUNS) test $(GO_TESTS) build clean

all: test build

build: $(GO_BUILDS)

$(GO_RUNS): # format run/<NAME OF COMMAND>
	go run ./cmd/$(patsubst run/%,%,$@)/... "$(ARGS)"

test: $(GO_FILES)
	go test ./...

$(GO_TESTS): # format test/<FILE PATH TO PACKAGE>
	go test ./$(patsubst test/%,%,$@)/...

clean:
	-rm $(GO_BUILDS)
	go clean -testcache
	go clean -cache