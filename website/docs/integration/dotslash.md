# Dotslash and Version Pinning

On the [GitHub release page of Scrut](https://github.com/facebookincubator/scrut/releases/latest) you will find a file  named `scrut`. This is a [Dotslash](https://dotslash-cli.com/docs/) file. Dotslash is a command line tool that is designed to fetch, verify and execute arbitrary other command line tools.

Using the Dotslash allows you to pin a specific version of Scrut in your automation (i.e. CI/CD pipeline) without storing the Scrut binary itself. It also automatically checks the hash of the downloaded file to ensure that the file is not corrupted.

Assuming `dotslash` is installed in your system, then you can:

```bash
# decide on the version
$ export SCRUT_VERSION=v0.3.0

# Download the latest (or specific version) of the Scrut Dotslash file
$ curl -L https://github.com/facebookincubator/scrut/releases/download/${SCRUT_VERSION}/scrut > scrut
$ chmod +x scrut

# Execute Scrut via Dotslash
$ ./scrut test some/file.md
```

:::tip

DotSlash files provide download instructions for multiple operating systems. The Scrut Dotslash file is configured to work on Mac (ARM64 and x86_64), Linux (ARM64 and x86_64) and Windows (x86_64). That makes it very easy to run Scrut tests in multi-platform scenarios.

:::
