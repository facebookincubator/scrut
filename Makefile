.DEFAULT_GOAL := selftest
.PHONY: build selftest cargotest doctest pytest tests update-tests
export SHELL = bash

ifeq ($(MOON_CRAM_BIN),)
MOON_CRAM_BIN = $(shell pwd)/target/debug/moon-cram
endif

export

$(MOON_CRAM_BIN): $(shell find src/ -type f -name "*.rs")
	test -x "$$MOON_CRAM_BIN" || cargo build --bin moon-cram

build: $(MOON_CRAM_BIN)

check:
	cargo check --bin moon-cram

selftest/cases/crlf-encoded.md:
	@echo -e '# Here is a CRLF encoded file\r\n' \
		'\r\n' \
		'```mooncram\r\n' \
		'$ echo foo\r\n' \
		'foo\r\n' \
		'```\r\n\r\n' > "$@"

# TODO remove this once T136897640 closes
selftest_cram: $(MOON_CRAM_BIN)
	$(MOON_CRAM_BIN) test \
		--verbose --cram-compat --keep-output-crlf --combine-output \
		$$(find selftest -type f -name "*.t" -not -name "*fail*")

selftest_markdown: $(MOON_CRAM_BIN)
	$(MOON_CRAM_BIN) test \
		--verbose \
		$$(find selftest -type f \( -name "*.md" -o -name "*.mooncram" \) -not -name "*fail*")

selftest: selftest_markdown selftest_cram

cargotest:
	cargo test --features volatile_tests

test: cargotest selftest

update_tests_markdown: $(MOON_CRAM_BIN)
	export PATH="$$(dirname "$(MOON_CRAM_BIN)"):$$PATH"; \
	$(MOON_CRAM_BIN) update --replace \
		$$(find selftest -type f \( -name "*.md" -o -name "*.mooncram" \) -not -name "*fail*");

update_tests_cram: $(MOON_CRAM_BIN)
	export PATH="$$(dirname "$(MOON_CRAM_BIN)"):$$PATH"; \
	$(MOON_CRAM_BIN) update --replace --combine-output --keep-output-crlf \
		$$(find selftest -type f -name "*.t" -not -name "*fail*");

update_tests: update_tests_markdown update_tests_cram
