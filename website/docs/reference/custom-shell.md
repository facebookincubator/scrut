# Custom Shell

While Scrut currently only supports `bash` (>= 3.2) a custom shell can be provided with the `--shell` command line parameter.
To understand how that works consider the following:

```bash title="Terminal"
$ echo "echo Hello" | /bin/bash -
Hello
```

What the above does is piping the string `echo Hello` into the `STDIN` of the process that was started with `/bin/bash -`.
Scrut pretty much does the same with each shell expressions within a test file.

So why provide a custom `--shell` then?
This becomes useful in at least two scenarios:
1. You need to execute the same code before Scrut runs each individual expression
2. You need Scrut to redirect the execution, for example an isolated environment

For (1) consider the following code:

```bash title="my_custom_setup.sh"
#!/bin/bash

# do something in this wrapper script
source /my/custom/setup.sh
run_my_custom_setup

# consume and run STDIN
source /dev/stdin
```

For (2) consider the following:

```bash title="my_remote_execution.sh"
#!/bin/bash

# do something in this wrapper script
source /my/custom/setup.sh
run_my_custom_setup

# end in a bash process that will receive STDIN
exec ssh username@acme.tld /bin/bash
```
