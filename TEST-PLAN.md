# Moon Cram Regression Test Plan

General-purpose verification steps for validating changes to Moon Cram. This file
is Moon Cram-compliant and can be executed directly:

```bash
moon-cram test fbcode/clifoundation/mooncram/TEST-PLAN.md --work-directory ~/fbsource
```

## Prerequisites

If the `moon-cram` CLI is not installed, you can get it in two ways:

1. **DotSlash (no install needed):** Use the DotSlash file checked into the repo:

   ```bash
   fbcode/clifoundation/mooncram/bin/moon-cram test fbcode/clifoundation/mooncram/TEST-PLAN.md --work-directory ~/fbsource
   ```

2. **Feature install on devserver:**

   ```bash
   feature install moon-cram
   ```

## Testing CLI changes locally

If you modify the CLI itself (e.g. add a flag, change output formatting), use
`buck2 run` to build and run the modified binary:

```bash
buck2 run fbcode//clifoundation/mooncram:moon-cram -- <command>
```

For example, to test a new `--verbose` flag:

```bash
buck2 run fbcode//clifoundation/mooncram:moon-cram -- test --verbose path/to/test.md
```

## 1. Format & Lint

```mooncram
$ arc lint --never-apply-patches
* (glob*)
```

## 2. Build the binary

```mooncram
$ buck2 build fbcode//clifoundation/mooncram:moon-cram 2>&1
* (glob+)
```

## 3. Run binary unit tests

```mooncram
$ buck2 test fbcode//clifoundation/mooncram:moon-cram-unittest 2>&1
* (glob+)
```

## 4. Run library unit tests

```mooncram
$ buck2 test fbcode//clifoundation/mooncram:tests 2>&1
* (glob+)
```

## 5. Run integration tests

```mooncram
$ buck2 test fbcode//clifoundation/mooncram:integration-tests 2>&1
* (glob+)
```
