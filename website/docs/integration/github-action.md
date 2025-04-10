import OssNote from '../fb/components/_oss-note.md';

# Scrut in GitHub Action

<FbInternalOnly><OssNote /></FbInternalOnly>

Currently there is no official Scrut GitHub Action, but you can manually run Scrut. Following an example how that can look like:
- Runs on push and on PRs
- Checks out the current code
- Installs [DotSlash](/docs/integration/dotslash/)
- Downloads a pinned version of Scrut
- Runs all test documents in the `my-test-folder/` directory with Scrut

```yaml title=".github/workflows/scrut-tests.yml" showLineNumbers
name: Scrut Tests
on: [push, pull_request]

jobs:
  scrut-tests:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout Code
        uses: actions/checkout@v3
      - name: Setup DotSlash
        uses: facebook/install-dotslash@latest
      - name: Download Scrut DotSlash Wrapper
        env:
          SCRUT_VERSION: v0.3.0
        run: |
          curl --proto '=https' --tlsv1.2 -LsSf \
            "https://github.com/facebookincubator/scrut/releases/download/$SCRUT_VERSION/scrut" \
            > scrut
          chmod +x scrut
      - name: Run Scrut Tests
        run: ./scrut test my-test-folder/
```

:::note

- If you wish to use the latest version of Scrut, instead of a pinned one, use the URL `https://github.com/facebookincubator/scrut/releases/latest/download/scrut` instead.
- You could, of course, download and unpack the Scrut binary directly, but then you have to consider the OS (DotSlash does this for you) and it will be slightly more boilerplate code.

:::
