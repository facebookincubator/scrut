# Scrut Regression Test Plan

General-purpose verification steps for validating changes to scrut. This file
is scrut-compliant and can be executed directly:

```bash
scrut test fbcode/clifoundation/scrut/TEST-PLAN.md --work-directory ~/fbsource
```

## Prerequisites

If the `scrut` CLI is not installed, you can get it in two ways:

1. **DotSlash (no install needed):** Use the DotSlash file checked into the repo:

   ```bash
   fbcode/clifoundation/scrut/bin/scrut test fbcode/clifoundation/scrut/TEST-PLAN.md --work-directory ~/fbsource
   ```

2. **Feature install on devserver:**

   ```bash
   feature install scrut
   ```

## Testing CLI changes locally

If you modify the CLI itself (e.g. add a flag, change output formatting), use
`buck2 run` to build and run the modified binary:

```bash
buck2 run fbcode//clifoundation/scrut:scrut -- <command>
```

For example, to test a new `--verbose` flag:

```bash
buck2 run fbcode//clifoundation/scrut:scrut -- test --verbose path/to/test.md
```

## 1. Format & Lint

```scrut
$ arc lint --never-apply-patches
* (glob*)
```

## 2. Build the binary

```scrut
$ buck2 build fbcode//clifoundation/scrut:scrut 2>&1
* (glob+)
```

## 3. Run binary unit tests

```scrut
$ buck2 test fbcode//clifoundation/scrut:scrut-unittest 2>&1
* (glob+)
```

## 4. Run library unit tests

```scrut
$ buck2 test fbcode//clifoundation/scrut:tests 2>&1
* (glob+)
```

## 5. Run integration tests

```scrut
$ buck2 test fbcode//clifoundation/scrut:integration-tests 2>&1
* (glob+)
```
