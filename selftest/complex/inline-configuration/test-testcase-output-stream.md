# Validate per-testcase output_stream configuration

Tests in this file validate that the `output_stream` determines which output stream(s) scrut validates


```scrut
$ function print_streams {
>    echo "Word on STDOUT"
>    >&2 echo "Word on STDERR"
> }
```

## Per default only STDOUT is validated

```scrut
$ print_streams
Word on STDOUT
```

TODO: default must change to Combined

## Selected STDOUT is exclusively validated

```scrut {output_stream: stdout}
$ print_streams
Word on STDOUT
```

## Selected STDERR is exclusively validated

```scrut {output_stream: stderr}
$ print_streams
Word on STDERR
```

## If enabled default both STDOUT and STDERR are validated

```scrut {output_stream: combined}
$ print_streams
Word on STDOUT
Word on STDERR
```
