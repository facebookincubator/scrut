# Test Scrut's multiline inline configuration syntax is supported

Below tests validate that multi-line configuration syntax is interpreted, by running a test with CRLF normalizing enabled and disabled.
They can only both succeed if the crlf_keep option is delegated.


## Disable CRLF option

```scrut
% keep_crlf: false
$ echo -e "word\r"
word
```

This test can only succeed if a) either keep_crlf is disabled per default or the multi-line syntax disables configuration the multi-line syntax enables configuraiton and b) keep_crlf works as expected.

## Enable CRLF option

```scrut
% keep_crlf: true
$ echo -e "word\r"
word\r (escaped)
```

This test can only succeed if a) either keep_crlf is enabled per default or the multi-line syntax enables configuration and b) CR characters are indeed kept
