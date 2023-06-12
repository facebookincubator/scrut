This should fail
  $ echo Hello
  World


This long multiline test should fail somewhere in the middle
  $ echo -ne "Lorem ipsum dolor sit amet, consectetur adipiscing elit,\n"\
  >          "sed do eiusmod tempor incididunt ut labore et dolore magna\n"\
  >          "aliqua. Ut enim ad minim veniam, quis nostrud exercitation\n"\
  >          "ullamco laboris nisi ut aliquip ex ea commodo consequat.\n"\
  >          "Duis aute irure dolor in FOO reprehenderit in voluptate velit\n"\
  >          "esse cillum dolore eu fugiat nulla pariatur. Excepteur sint\n"\
  >          "occaecat cupidatat non proident, sunt in culpa qui officia\n"\
  >          "deserunt mollit anim id est laborum"
  Lorem ipsum dolor sit amet, consectetur adipiscing elit,
   sed do eiusmod tempor incididunt ut labore et dolore magna
   aliqua. Ut enim ad minim veniam, quis nostrud exercitation
   ullamco laboris nisi ut aliquip ex ea commodo consequat.
   Duis aute irure dolor in reprehenderit in voluptate velit
   esse cillum dolore eu fugiat nulla pariatur. Excepteur sint
   occaecat cupidatat non proident, sunt in culpa qui officia
   deserunt mollit anim id est laborum (no-eol)


Failing with the wrong exit code
  $ (exit 2)
  Arbitrary expectation
  [3]
