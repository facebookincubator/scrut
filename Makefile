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
	$(SCRUT_BIN) test --cram-compat --keep-output-crlf --combine-output \
		$$(find selftest -type f \( -name "*.t" \) -not -name "*fail*")

selftest_markdown: $(SCRUT_BIN)
	$(SCRUT_BIN) test \
		$$(find selftest -type f \( -name "*.md" \) -not -name "*fail*")

selftest: selftest_markdown selftest_cram

cargotest:
	cargo test --features volatile_tests

doctest: $(SCRUT_BIN)
	export PATH="$$(dirname "$(SCRUT_BIN)"):$$PATH"; \
	$(SCRUT_BIN) test \
		--work-directory="$$(pwd)" --markdown-languages=sh $$(find docs -type f -not -name "*Meta*" -and -not -name "Tutorial.md")

pytest: $(SCRUT_BIN)
	make -C py test

test: cargotest selftest doctest pytest


update-tests: $(SCRUT_BIN)
	export PATH="$$(dirname "$(SCRUT_BIN)"):$$PATH"; \
	$(SCRUT_BIN) update --replace \
		$$(find selftest -type f \( -name "*.md" \) -not -name "*fail*"); \
	$(SCRUT_BIN) update --replace \
		$$(find selftest -type f \( -name "*.t" \) --combine-output --keep-output-crlf -not -name "*fail*"); \
	$(SCRUT_BIN) update \
		--replace --work-directory="$$(pwd)" --markdown-languages=sh \
			$$(find docs -type f -not -name "*Meta*" -and -not -name "Tutorial.md")
