# Dirstack recovery

This test validates that the `DIRSTACK` (in bash) is consistent in between
test executions.

## Initialize directories and directory stack

```scrut
$ mkdir dir1 dir2 dir3
> cd "dir1"
> pushd "../dir2" >/dev/null
> pushd "../dir3" >/dev/null
> pwd
> dirs -l -p
*/dir3 (glob)
*/dir3 (glob)
*/dir2 (glob)
*/dir1 (glob)
```

## Verify that the directory stack is the same

```scrut
$ pwd
> dirs -l -p
*/dir3 (glob)
*/dir3 (glob)
*/dir2 (glob)
*/dir1 (glob)
```
