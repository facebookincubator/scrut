# Validate per-testcase output_stream configuration

Tests in this file validate that the `output_stream` determines which output stream(s) mooncram validates


```mooncram
$ function print_streams {
>    echo "Word on STDOUT"
>    >&2 echo "Word on STDERR"
> }
```

## Per default only STDOUT is validated

```mooncram
$ print_streams
Word on STDOUT
```

TODO: default must change to Combined

## Selected STDOUT is exclusively validated

```mooncram {output_stream: stdout}
$ print_streams
Word on STDOUT
```

## Selected STDERR is exclusively validated

```mooncram {output_stream: stderr}
$ print_streams
Word on STDERR
```

## If enabled default both STDOUT and STDERR are validated

```mooncram {output_stream: combined}
$ print_streams
Word on STDOUT
Word on STDERR
```
