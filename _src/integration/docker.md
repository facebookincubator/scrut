import OssNote from '../fb/components/_oss-note.md';

# Scrut in Docker Container

<FbInternalOnly><OssNote /></FbInternalOnly>

Scrut can be run in a Docker container. This is useful when integrating into CI/CD or if no local Rust development environment is available.

## Get Scrut Docker Image

There are two ways:

### Pre-Built Image from GHCR

Here is how you can [work with theGitHub Container Registry](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry).

The image is then available as:

```
ghcr.io/facebookexternal/scrut:<VERSION>
```

### Build Locally

Check out the [Scrut git repository on GitHub](https://github.com/facebookincubator/scrut) locally. It comes with a `Dockerfile` in the root directory.

Now build the image:

```bash
$ docker build -t scrut:latest .
```

:::note

The build requires [Docker BuildKit](https://docs.docker.com/build/buildkit/).

:::


:::tip

The container build automatically runs both unit and integrating tests. This makes it a good, isolated development environment if you are interested in contributing to Scrut.

If you want to skip the tests, resulting in a faster build, you can set `--build-arg SKIP_TESTS=yes` when executing `docker build`.

:::

## Run Scrut in Docker Container

Once you have the image available make sure to mount the directory containing the test suite as a volume into the container under `/app`.

Following an example with a small Rust CLI:

```bash
$ cd my-cli
$ tree
.
├── Cargo.toml
├── dist
│   └── my-cli
├── src
│   ├── command_something_else.rs
│   ├── command_user_list.rs
│   ├── command_user_login.rs
│   └── main.rs
└── tests
    ├── smoke.md
    ├── something-else.md
    ├── user-listing.md
    └── user-login.md
```

Now you would run Scrut like this:

```bash title="Terminal"
$ docker run --rm -ti -v $(pwd):/app scrut:latest test --verbose tests/
🔎 Found 4 test document(s)
✅ tests/user-login.md: passed 3 testcases
✅ tests/smoke.md: passed 5 testcases
✅ tests/user-listing.md: passed 1 testcase
✅ tests/something-else.md: passed 13 testcases

Result: 4 document(s) with 22 testcase(s): 22 succeeded, 0 failed and 0 skipped
```

:::tip

Running tests inside a container can change the path location of the CLI binary. Consider using the `--prepend-test-file-paths` parameter to inject a [test document](/docs/reference/fundamentals/test-document/) that extends the `PATH` environment variable as needed. Here an example:

````markdown title="docker-prepend.md"
# Add `/app/dist` to `PATH`

```scrut
$ export PATH="/app/dist:$PATH"
```
````

And then all calls to `my-cli` in the [test documents](/docs/reference/fundamentals/test-document/) will be resolved to `/app/dist/my-cli`:

```bash title="Terminal"
$ docker run --rm -ti -v $(pwd):/app scrut:latest \
    test --verbose --prepend-test-file-paths=./docker-prepend.md tests/
🔎 Found 4 test document(s)
✅ tests/user-login.md: passed 4 testcases
✅ tests/smoke.md: passed 6 testcases
✅ tests/user-listing.md: passed 2 testcase
✅ tests/something-else.md: passed 14 testcases

Result: 4 document(s) with 26 testcase(s): 26 succeeded, 0 failed and 0 skipped
```

:::
