all: src/parser.c
	tree-sitter test

fixme: src/parser.c
	tree-sitter test -f Fixme

src/parser.c: grammar.js
	tree-sitter generate

build: src/parser.c size

size: src/parser.c
	du -b src/parser.c
	du -b src/parser.c > size.txt

# The scanner test and the fuzzer compile the C sources, including the
# tree-sitter runtime, with sanitizers and with the scanner's internal
# assertions enabled. rustc links without the C compiler's default libraries,
# which on macOS leaves out the sanitizer runtime, so it is named explicitly.
SANITIZE = -g -O1 -fsanitize=address,undefined -fno-sanitize-recover=undefined -DTYPST_SCANNER_DEBUG
ifeq ($(shell uname),Darwin)
ASAN_RUNTIME = $(shell $(CC) -print-file-name=libclang_rt.asan_osx_dynamic.dylib)
SANITIZE_LINK = -C link-arg=$(ASAN_RUNTIME) -C link-arg=-Wl,-rpath,$(dir $(ASAN_RUNTIME))
else
SANITIZE_LINK = -C link-arg=-fsanitize=address,undefined
endif
FUZZ_SECONDS ?= 60

# Instrumented objects get their own target directory so they never mix with a
# normal build.
scanner-test: src/parser.c
	CFLAGS="$(SANITIZE)" RUSTFLAGS="$(SANITIZE_LINK)" \
		cargo test --target-dir target/sanitize --test scanner_state

# Runs libFuzzer for FUZZ_SECONDS, seeded with every corpus input. Crashing and
# slow inputs are written to fuzz/artifacts. Inputs that once crashed are kept
# in fuzz/regressions and replayed on every run.
fuzz: src/parser.c
	mkdir -p fuzz/artifacts
	cargo run --manifest-path fuzz/Cargo.toml --bin seeds -- fuzz/corpus test/corpus/*.scm
	CFLAGS="$(SANITIZE) -fsanitize=fuzzer-no-link" RUSTFLAGS="$(SANITIZE_LINK)" \
	TYPST_FUZZ_SLOW_DIR=fuzz/artifacts \
		cargo run --manifest-path fuzz/Cargo.toml --target-dir fuzz/target/sanitize --bin parse -- \
		fuzz/corpus fuzz/regressions -dict=fuzz/typst.dict -artifact_prefix=fuzz/artifacts/ \
		-max_len=65536 -timeout=120 -max_total_time=$(FUZZ_SECONDS)

.PHONY: fixme test build all size scanner-test fuzz
