For bash aliase expansion (aka "stateful aliases") must be enabled explicitly
  $ [[ "$SHELL" =~ "bash" ]] && shopt -s expand_aliases


Creates state
  $ export SOMETHINGFOO=ihavestate


Consumes state
  $ echo "STATE: ${SOMETHINGFOO:-missing}"
  STATE: ihavestate


Load state
  $ . ${TESTDIR}/state.sh


Consume more state
  $ echo "STATE: ${SOMEMORESTATE:-missing}"
  STATE: morestate


Load alais
  $ . ${TESTDIR}/alias.sh


Use alias
  $ some_alias
  I have been set
