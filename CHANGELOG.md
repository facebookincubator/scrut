# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2026-10-01

### Added

- Interactive mode (`mode: interactive`) to test programs through a pseudo-terminal, using `WAIT`, `WRITE`, `SEND_KEYS` and `ASSERT` directives (short forms `@`, `|`, `^`, `!`), per-directive timeouts and conditional key sending. See [Interactive Mode](https://facebookincubator.github.io/scrut/docs/reference/fundamentals/interactive-mode/).
- JSON Schema validation mode (`mode: jsonschema`) to validate a command's JSON output against a schema written inline as YAML. See [Validation Modes](https://facebookincubator.github.io/scrut/docs/reference/fundamentals/validation-modes/).
- New testcase configuration option `interpolated`: if set to `true`, `$VAR` and `${VAR}` in output expectations are replaced with environment variables captured from the test's shell (`$$` for a literal `$`). `scrut update` does not preserve the variable references. See [Output Expectations](https://facebookincubator.github.io/scrut/docs/reference/fundamentals/output-expectations/).
- Multi-line testcase configuration: per-testcase configuration can be written as `% key: value` lines at the top of the code block instead of the single-line `{...}` form. See [Inline Configuration](https://facebookincubator.github.io/scrut/docs/reference/fundamentals/inline-configuration/).
- JUnit XML renderer (`--renderer junit`) for CI test result collectors (see [Test Output](https://facebookincubator.github.io/scrut/docs/reference/fundamentals/test-output/#junit-renderer)):
  - File paths relative to the working directory, and `line` pointing at the first failing expectation.
  - `package` and `timestamp` on test suites, and a `<properties>` block recording the document format.
  - A message on skipped test cases.
- Per-testcase execution duration (`duration_ms`) in `json` and `yaml` renderer output, where known (not for Cram / `--cram-compat` documents).
- Shell completions for Bash, Zsh, Fish, PowerShell and Elvish via the `_SCRUT_COMPLETE` environment variable, by [@cboone] in [#48](https://github.com/facebookincubator/scrut/pull/48). See [Installation](https://facebookincubator.github.io/scrut/docs/getting-started/installation/).
- New crates `scrutty` and `scrutty_macros` (0.1.1): a PTY-based end-to-end test framework for CLI applications, published alongside Scrut and used by interactive mode.
- GitHub Action that verifies on every push to `main` that all crates resolve and build.

### Changed

- **Breaking (library API):** `TestCase.expectations` is replaced by `TestCase.body` (`ValidationBody`), with an `expectations()` accessor, and `TestCaseError::MalformedOutput` is replaced by `TestCaseError::ValidationFailed(ValidationFailure)`. `executors::executor::DEFAULT_TOTAL_TIMEOUT` is now a plain `Duration` (use it without `*`), and `executors::DEFAULT_SHELL` is now a `LazyLock<&Path>`. CLI output, including the `json` and `yaml` renderers, is unchanged.
- Minimum supported Rust version raised from 1.85 to 1.91.
- Release workflow publishes `scrutty_macros`, `scrutty` and `scrut` to crates.io in dependency order.
- GitHub releases use the matching section of this changelog as their description.
- Website dependencies updated, including security fixes.

### Fixed

- `scrut --version` reported a Unix timestamp instead of the version when built outside a git checkout, e.g. with `cargo install scrut`.
- Installation script downloaded releases from a personal fork instead of `facebookincubator/scrut` ([#53](https://github.com/facebookincubator/scrut/issues/53)).

## [0.4.3] - 2026-01-28

### Added

- New testcase configuration option `fail_fast`: if set to `true`, a failing testcase in the test document skips execution of all subsequent tests.

## [0.4.2] - 2025-08-19

Not published to crates.io; these changes first shipped there in 0.4.3.

### Added

- `.scrut` as an additional default suffix for Markdown-formatted test documents.

### Changed

- In multiline commands, subsequent lines starting with `>` no longer require a following whitespace, to reduce friction with IDEs that remove it if the line is otherwise empty.

## [0.4.1] - 2025-06-06

### Added

- Parameter `--max-multiline-matched-lines` for `test` and `update` command with default `100`.
- Automatic testing on PRs in GitHub.
- Environment variable `SCRUT_DEFAULT_SHELL` to override detected `bash` binary at runtime.

### Changed

- Rendering of matches of multiline expectations in failed test results: output lines that are covered by the multiline expectations are now printed, limited to parameter-defined amount.

### Removed

- Obsolete test targets from `Makefile`.

### Fixed

- Some integration tests on Windows.
- Cleanup of temporary directories in unit tests on Windows.
- Docker image building GitHub Action.

## [0.4.0] - 2025-05-13

### Added

- Detached process handling (Linux/MacOS): new per-testcase inline configuration option `detached_kill_signal` that accepts common *nix signal notations (`SIGTERM`, `term`, `15`) and can be disabled (`disabled`).
- Default kill signal `SIGTERM` on Linux/MacOS.
- Windows support for the new configuration: accepted and validated, but not acted on.
- Documentation of the new behavior.

## [0.3.0] - 2025-04-11

### Added

- `--verbose` flag to also print "info level" messages.
- `--log-level` parameter, defaults to `warn`:
  - When the log level is `info` or higher, progress printing is disabled and `info` and `warn` level log messages are printed instead.
- Per-testcase `skip_ansi_escaping` configuration option that allows for writing tests ignoring color encoding etc.
- Getting started section with installation guide.
- curl-style installation script.
- Dockerfile.
- GitHub Action instructions.
- GitHub Action to build and publish the Docker image.
- Prism language support for bash / markdown "scrut style".

### Changed

- `create`, `test` and `update` commands now provide progress and better insight into what is being processed.
- Tutorial refactored into multiple pages and largely rewritten / updated.
- Catch-all pages separated into a structured Reference section.

### Removed

- `colored` crate: through using `indicatif` for progress the `console` crate is already available, and it provides the same function.

### Fixed

- Small fixes in `Makefile`.

## [0.2.3] - 2025-03-24

### Changed

- `Cargo.toml` is now auto-generated and contains up-to-date dependencies.

## [0.2.2] - 2025-03-20

No release notes. Not published to crates.io.

## [0.2.1] - 2025-01-17

No release notes.

[0.5.0]: https://github.com/facebookincubator/scrut/compare/v0.4.3...v0.5.0
[0.4.3]: https://github.com/facebookincubator/scrut/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/facebookincubator/scrut/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/facebookincubator/scrut/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/facebookincubator/scrut/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/facebookincubator/scrut/compare/v0.2.3...v0.3.0
[0.2.3]: https://github.com/facebookincubator/scrut/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/facebookincubator/scrut/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/facebookincubator/scrut/releases/tag/v0.2.1
[@cboone]: https://github.com/cboone
