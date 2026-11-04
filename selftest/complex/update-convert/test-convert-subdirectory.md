# Converted documents are written next to their source

## Create Cram tests in subdirectories, with the same file name

```scrut
$ mkdir -p sub/a sub/b
```

```scrut
$ "${SCRUT_BIN}" create --format cram "echo A" > sub/a/x.t
```

```scrut
$ "${SCRUT_BIN}" create --format cram "echo B" > sub/b/x.t
```

```scrut
$ "${SCRUT_BIN}" create --format cram "echo C" > sub/a/y.z.t
```

## Convert all of them to Markdown

```scrut
$ "${SCRUT_BIN}" update --assume-yes --convert markdown sub
Result: 3 document(s) of which 3 updated, 0 skipped and 0 unchanged
```

Each converted document is next to its source, keeping its full name:

```scrut
$ find . -type f | sort
./sub/a/x.md
./sub/a/x.t
./sub/a/y.z.md
./sub/a/y.z.t
./sub/b/x.md
./sub/b/x.t
```

```scrut
$ grep '^\$' sub/a/x.md sub/b/x.md sub/a/y.z.md
sub/a/x.md:$ echo A
sub/b/x.md:$ echo B
sub/a/y.z.md:$ echo C
```
