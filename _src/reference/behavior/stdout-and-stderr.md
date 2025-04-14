# STDOUT and STDERR

Commands-line applications can generate output on to two streams: `STDOUT` and `STDERR`. There is no general agreement on which stream is supposed to contain what kind of data, but commonly `STDOUT` contains the primary output and `STDERR` contains logs, debug messages, etc. This is also the recommendation of the [CLI guidelines](https://clig.dev/#:~:text=primary%20output%20for%20your%20command).

**Scrut, by default, only considers `STDOUT` when validating output.**

You can modify this behavior by using the [`output_stream` configuration directive](/docs/reference/fundamentals/inline-configuration/) or the `--(no-)combine-output` command-line parameters.

:::tip

While you can configure which output streams Scrut considers when evaluating output expecations, you can also steer this by using stream control bash primitives like `some-command 2>&1`.

:::

:::note

The above is true for Markdown [test documents](/docs/reference/fundamentals/test-document/). However Cram [test documents](/docs/reference/fundamentals/test-document/) default to combining `STDOUT` and `STDERR`.

:::
