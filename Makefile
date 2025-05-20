.DEFAULT_GOAL := selftest
.PHONY: build selftest cargotest doctest pytest tests update-tests
export SHELL = bash

ifeq ($(SCRUT_BIN),)
SCRUT_BIN = $(shell pwd)/target/debug/scrut
endif

export

$(SCRUT_BIN): $(shell find src/ -type f -name "*.rs")
	test -x "$$SCRUT_BIN" || cargo build --bin scrut

build: $(SCRUT_BIN)

check:
	cargo check --bin scrut

selftest/cases/crlf-encoded.md:
	@echo -e '# Here is a CRLF encoded file\r\n' \
		'\r\n' \
		'```scrut\r\n' \
		'$ echo foo\r\n' \
		'foo\r\n' \
		'```\r\n\r\n' > "$@"

# TODO remove this once T136897640 closes
selftest_cram: $(SCRUT_BIN)
	$(SCRUT_BIN) test \
		--verbose --cram-compat --keep-output-crlf --combine-output \
		$$(find selftest -type f -name "*.t" -not -name "*fail*")

selftest_markdown: $(SCRUT_BIN)
	$(SCRUT_BIN) test \
		--verbose \
		$$(find selftest -type f -name "*.md" -not -name "*fail*")

selftest: selftest_markdown selftest_cram

cargotest:
	cargo test --features volatile_tests

test: cargotest selftest

update_tests_markdown: $(SCRUT_BIN)
	export PATH="$$(dirname "$(SCRUT_BIN)"):$$PATH"; \
	$(SCRUT_BIN) update --replace \
		$$(find selftest -type f -name "*.md" -not -name "*fail*");

update_tests_cram: $(SCRUT_BIN)
	export PATH="$$(dirname "$(SCRUT_BIN)"):$$PATH"; \
	$(SCRUT_BIN) update --replace --combine-output --keep-output-crlf \
		$$(find selftest -type f -name "*.t" -not -name "*fail*");

update_tests: update_tests_markdown update_tests_cram
