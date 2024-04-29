"use strict";(self.webpackChunkstaticdocs_starter=self.webpackChunkstaticdocs_starter||[]).push([[214],{42361:(e,n,t)=>{t.r(n),t.d(n,{assets:()=>o,contentTitle:()=>c,default:()=>h,frontMatter:()=>r,metadata:()=>d,toc:()=>l});var i=t(85893),s=t(11151);const r={sidebar_position:3},c="Specifics",d={id:"advanced/specifics",title:"Specifics",description:"This chapter describes behaviors of Scrut that should be known by the user to prevent surprises in the wrong moment.",source:"@site/docs/advanced/specifics.md",sourceDirName:"advanced",slug:"/advanced/specifics",permalink:"/scrut/docs/advanced/specifics",draft:!1,unlisted:!1,editUrl:"https://www.internalfb.com/code/fbsource/fbcode/clifoundation/scrut/website/docs/advanced/specifics.md",tags:[],version:"current",sidebarPosition:3,frontMatter:{sidebar_position:3},sidebar:"tutorialSidebar",previous:{title:"Expectations",permalink:"/scrut/docs/advanced/expectations"},next:{title:"Development",permalink:"/scrut/docs/advanced/development"}},o={},l=[{value:"Test output",id:"test-output",level:2},{value:"Pretty Renderer (default)",id:"pretty-renderer-default",level:3},{value:"Diff renderer",id:"diff-renderer",level:3},{value:"JSON and YAML renderer",id:"json-and-yaml-renderer",level:3},{value:"Test environment variables",id:"test-environment-variables",level:2},{value:"Scrut specific environment variables",id:"scrut-specific-environment-variables",level:3},{value:"Common (linux) environment variables",id:"common-linux-environment-variables",level:3},{value:"(Optional) Cram environment variables",id:"optional-cram-environment-variables",level:3},{value:"Test work directory",id:"test-work-directory",level:2},{value:"Test execution",id:"test-execution",level:2},{value:"Execution within a custom shell",id:"execution-within-a-custom-shell",level:3},{value:"STDOUT and STDERR",id:"stdout-and-stderr",level:2},{value:"Exit Codes",id:"exit-codes",level:2},{value:"Skip Tests with Exit Code 80",id:"skip-tests-with-exit-code-80",level:3},{value:"Scrut Exit Code",id:"scrut-exit-code",level:3},{value:"Newline handling",id:"newline-handling",level:2},{value:"Execution Environment",id:"execution-environment",level:2}];function a(e){const n={a:"a",blockquote:"blockquote",code:"code",em:"em",h1:"h1",h2:"h2",h3:"h3",li:"li",ol:"ol",p:"p",pre:"pre",strong:"strong",ul:"ul",...(0,s.a)(),...e.components};return(0,i.jsxs)(i.Fragment,{children:[(0,i.jsx)(n.h1,{id:"specifics",children:"Specifics"}),"\n",(0,i.jsx)(n.p,{children:"This chapter describes behaviors of Scrut that should be known by the user to prevent surprises in the wrong moment."}),"\n",(0,i.jsx)(n.h2,{id:"test-output",children:"Test output"}),"\n",(0,i.jsx)(n.p,{children:"Executing a test with Scrut results either in success (when all expectations in the test match) or failure (when at least one expectation in the test does not match)."}),"\n",(0,i.jsxs)(n.p,{children:["Scrut supports multiple ",(0,i.jsx)(n.em,{children:"output renderers"}),", which yield a different representation of the test results."]}),"\n",(0,i.jsx)(n.h3,{id:"pretty-renderer-default",children:"Pretty Renderer (default)"}),"\n",(0,i.jsx)(n.p,{children:"Scrut will always tell you what it did:"}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-bash",children:"$ scrut test selftest/cases/regex.md\nResult: 1 file(s) with 8 test(s): 8 succeeded, 0 failed and 0 skipped\n"})}),"\n",(0,i.jsxs)(n.p,{children:["In case of failure the ",(0,i.jsx)(n.code,{children:"pretty"})," default renderer will provide a human-readable output that points you to the problem with the output:"]}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-bash",children:"$ scrut test a-failing-test.md\n// =============================================================================\n// @ /path/to/a-failing-test.md:10\n// -----------------------------------------------------------------------------\n// # One conjunct expression\n// -----------------------------------------------------------------------------\n// $ echo Foo && \\\n//   echo Bar\n// =============================================================================\n\n1  1  |   Foo\n   2  | - BAR\n2     | + Bar\n3     | + Baz\n"})}),"\n",(0,i.jsx)(n.p,{children:"The failure output consists of two components:"}),"\n",(0,i.jsxs)(n.ol,{children:["\n",(0,i.jsxs)(n.li,{children:["The failure header, which consists of all initial lines that start with ",(0,i.jsx)(n.code,{children:"//"}),", indicates the position"]}),"\n",(0,i.jsx)(n.li,{children:"The failure body, which consists of all the following lines, indicates the problem"}),"\n"]}),"\n",(0,i.jsx)(n.p,{children:(0,i.jsx)(n.strong,{children:"Header"})}),"\n",(0,i.jsx)(n.p,{children:"The header contains three relevant information. Given the above output:"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"@ /path/to/a-failing-test.md:4"}),", tells you that the test that failed is in the provided file ",(0,i.jsx)(n.code,{children:"/path/to/a-failing-test.md"})," and that the shell expression (that failed the test) starts in line four of that file."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"# <test title>"}),", gives you the optional title of the test in the file. See ",(0,i.jsx)(n.a,{href:"/scrut/docs/advanced/file-formats",children:"File Formats"}),") to learn more. ",(0,i.jsx)(n.em,{children:"If the test does not have a title, this line is omitted."})]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"$ <test command>"}),", is the shell expectation from the test file that is tested and that has failed. Again, see ",(0,i.jsx)(n.a,{href:"/scrut/docs/advanced/file-formats",children:"File Formats"}),") for more information."]}),"\n"]}),"\n",(0,i.jsx)(n.p,{children:(0,i.jsx)(n.strong,{children:"Body"})}),"\n",(0,i.jsxs)(n.p,{children:["There are two possible variants that the ",(0,i.jsx)(n.code,{children:"diff"})," renderer may return:"]}),"\n",(0,i.jsxs)(n.ol,{children:["\n",(0,i.jsxs)(n.li,{children:["Failed ",(0,i.jsx)(n.a,{href:"/scrut/docs/advanced/expectations",children:"output expectations"})]}),"\n",(0,i.jsxs)(n.li,{children:["Failed ",(0,i.jsx)(n.a,{href:"#exit-codes",children:"exit code expectation"})]}),"\n"]}),"\n",(0,i.jsx)(n.p,{children:"The above output is a failed output expectations and you can read it as following:"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"1  1  |   Foo"}),": This line was printed as expected. The left hand ",(0,i.jsx)(n.code,{children:"1"})," is the number of the output line and the right hand ",(0,i.jsx)(n.code,{children:"1"})," is the number of the expectation."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"   2  | - BAR"}),": This line was expected, but not printed. The left hand omitted number indicates that it was not found in output. The right hand number tells that this is the second expectation. The ",(0,i.jsx)(n.code,{children:"-"})," before the line ",(0,i.jsx)(n.code,{children:"Bar"})," emphasizes that this is a missed expectation."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"2     | + Bar"}),": This line was printed and expected. The left hand ",(0,i.jsx)(n.code,{children:"2"})," is the number of the output line and the right hand ",(0,i.jsx)(n.code,{children:"3"})," is the number of the expectation."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"3     | + Baz"}),": This line was printed unexpectedly. The left hand ",(0,i.jsx)(n.code,{children:"3"})," is the number of the output line the omitted right hand number implies there is no expectation that covers it. The ",(0,i.jsx)(n.code,{children:"+"})," before the line ",(0,i.jsx)(n.code,{children:"Zoing"}),' emphasizes that this is a "surplus" line.']}),"\n"]}),"\n",(0,i.jsxs)(n.blockquote,{children:["\n",(0,i.jsxs)(n.p,{children:[(0,i.jsx)(n.strong,{children:"Note"}),": If you work with test files that contain a large amount of tests, then you may want to use the ",(0,i.jsx)(n.code,{children:"--absolute-line-numbers"})," flag on the command line: instead of printing the relative line number for each test, as described above, it prints absolute line numbers from within the test file. Assuming the ",(0,i.jsx)(n.code,{children:"Foo"})," expectation from above is in line 10 of a file, it would read ",(0,i.jsx)(n.code,{children:"13  13  |   Foo"})," - and all subsequent output liens with respective aligned line numbers."]}),"\n"]}),"\n",(0,i.jsxs)(n.p,{children:["An example for the body of an ",(0,i.jsx)(n.em,{children:"exit code expectation"}),":"]}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{children:"unexpected exit code\n  expected: 2\n  actual:   0\n\n## STDOUT\n#> Foo\n## STDERR\n"})}),"\n",(0,i.jsx)(n.p,{children:"This should be mostly self-explanatory. Scrut does not provide any output expectation failures, because it assumes that when the exit code is different, then it is highly likely that the output is very different - and even if not, it would not matter, as it failed anyway."}),"\n",(0,i.jsxs)(n.p,{children:["The tailing ",(0,i.jsx)(n.code,{children:"## STDOUT"})," and ",(0,i.jsx)(n.code,{children:"## STDERR"})," contain the output lines (prefixed with ",(0,i.jsx)(n.code,{children:"#> "}),") that were printed out from the failed execution."]}),"\n",(0,i.jsx)(n.h3,{id:"diff-renderer",children:"Diff renderer"}),"\n",(0,i.jsxs)(n.p,{children:["The ",(0,i.jsx)(n.code,{children:"diff"})," renderer, that can be enabled with ",(0,i.jsx)(n.code,{children:"--renderer diff"})," (or ",(0,i.jsx)(n.code,{children:"-r diff"}),"), prints a diff in the ",(0,i.jsx)(n.a,{href:"https://en.wikipedia.org/wiki/Diff#Unified_format",children:"unified format"}),"."]}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-bash",children:"$ scrut test -r diff a-failing-test.md\n--- /path/to/a-failing-test.md\n+++ /path/to/a-failing-test.md.new\n@@ -14 +14,2 @@ malformed output: One conjunct expression\n-BAR\n+Bar\n+Baz\n"})}),"\n",(0,i.jsxs)(n.blockquote,{children:["\n",(0,i.jsxs)(n.p,{children:[(0,i.jsx)(n.strong,{children:"Note"}),": The created diff is compatible with the ",(0,i.jsx)(n.code,{children:"patch"})," command line tool (e.g. ",(0,i.jsx)(n.code,{children:"patch -p0 < <(scrut test -r diff a-failing-test.md)"}),")."]}),"\n"]}),"\n",(0,i.jsx)(n.h3,{id:"json-and-yaml-renderer",children:"JSON and YAML renderer"}),"\n",(0,i.jsxs)(n.p,{children:["These renderer are primarily intended for automation and are to be considererd experimental.\nYou can explore them using ",(0,i.jsx)(n.code,{children:"--renderer yaml"})," or respective ",(0,i.jsx)(n.code,{children:"--renderer json"}),"."]}),"\n",(0,i.jsx)(n.h2,{id:"test-environment-variables",children:"Test environment variables"}),"\n",(0,i.jsxs)(n.p,{children:["Scrut sets a list of environment variables for the execution. These are set ",(0,i.jsx)(n.em,{children:"in addition to and overwriting"})," any environment variables that are set when ",(0,i.jsx)(n.code,{children:"scrut"})," is being executed."]}),"\n",(0,i.jsxs)(n.blockquote,{children:["\n",(0,i.jsxs)(n.p,{children:[(0,i.jsx)(n.strong,{children:"Note"}),": If you need an empty environment, consider executing using ",(0,i.jsx)(n.a,{href:"https://man7.org/linux/man-pages/man1/env.1.html",children:(0,i.jsx)(n.code,{children:"env"})}),", like ",(0,i.jsx)(n.code,{children:"env -i scrut test .."})," instead"]}),"\n"]}),"\n",(0,i.jsx)(n.h3,{id:"scrut-specific-environment-variables",children:"Scrut specific environment variables"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"TESTDIR"}),": contains the absolute path of the directory where the file that contains the test that is currently being executed is in"]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"TESTFILE"}),": contains the name of the file that contains the test that is currently being executed"]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"TESTSHELL"}),": contains the shell that in which the test is being executed in (default ",(0,i.jsx)(n.code,{children:"/bin/bash"}),", see ",(0,i.jsx)(n.code,{children:"--shell"})," flag on commands)"]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"TMPDIR"}),": contains the absolute path to a temporary directory that will be cleaned up after the test is executed. This directory is shared in between all executed tests across all test files."]}),"\n"]}),"\n",(0,i.jsx)(n.h3,{id:"common-linux-environment-variables",children:"Common (linux) environment variables"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"CDPATH"}),": empty"]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"COLUMNS"}),": ",(0,i.jsx)(n.code,{children:"80"})]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"GREP_OPTIONS"}),": empty"]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"LANG"}),": ",(0,i.jsx)(n.code,{children:"C"})]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"LANGUAGE"}),": ",(0,i.jsx)(n.code,{children:"C"})]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"LC_ALL"}),": ",(0,i.jsx)(n.code,{children:"C"})]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"SHELL"}),": Same as ",(0,i.jsx)(n.code,{children:"TESTSHELL"}),", see above"]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"TZ"}),": ",(0,i.jsx)(n.code,{children:"GMT"})]}),"\n"]}),"\n",(0,i.jsx)(n.h3,{id:"optional-cram-environment-variables",children:"(Optional) Cram environment variables"}),"\n",(0,i.jsxs)(n.p,{children:["When using the ",(0,i.jsx)(n.code,{children:"--cram-compat"})," flag, or when a Cram ",(0,i.jsx)(n.code,{children:".t"})," test file is being executed, the following additional environment variables will be exposed for compatibility:"]}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"CRAMTMP"}),": if no specific work directory was provided (default), then it contains the absolute path to the temporary directory in which per-test-file directories will be created in which those test files are then executed in (",(0,i.jsx)(n.code,{children:'CRAMTMP=$(realpath "$(pwd)/..")'}),"); otherwise the path to the provided work directory"]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"TMP"}),": same as ",(0,i.jsx)(n.code,{children:"TMPDIR"})]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"TEMP"}),": same as ",(0,i.jsx)(n.code,{children:"TMPDIR"})]}),"\n"]}),"\n",(0,i.jsx)(n.h2,{id:"test-work-directory",children:"Test work directory"}),"\n",(0,i.jsxs)(n.p,{children:["By default ",(0,i.jsx)(n.code,{children:"scrut"})," executes all tests in a dedicated directory ",(0,i.jsx)(n.em,{children:"per test file"}),". This means ",(0,i.jsx)(n.em,{children:"all tests within one file are being executed in the same directory"}),". The directory is created within the system temporary directory. It will be removed (including all the files or directories that the tests may have created) after all tests in the file are executed - or if the execution of the file fails for any reason."]}),"\n",(0,i.jsx)(n.p,{children:"This means something like the following can be safely done and will be cleaned up by Scrut after the test finished (however it finishes):"}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-markdown",children:'# Some test that creates a file\n\n```scrut\n$ date > file\n```\n\nThe `file` lives in the current directory\n\n```scrut\n$ test -f "$(pwd)/file"\n```\n'})}),"\n",(0,i.jsxs)(n.p,{children:["The directory within which tests are being executed can be explicitly set using the ",(0,i.jsx)(n.code,{children:"--work-directory"})," parameter for the ",(0,i.jsx)(n.code,{children:"test"})," and ",(0,i.jsx)(n.code,{children:"update"})," commands. If that parameter is set then ",(0,i.jsx)(n.em,{children:"all tests"})," from ",(0,i.jsx)(n.em,{children:"all test files"})," are executed run within that directory, and the directory is ",(0,i.jsx)(n.em,{children:"not removed"})," afterwards."]}),"\n",(0,i.jsxs)(n.blockquote,{children:["\n",(0,i.jsxs)(n.p,{children:[(0,i.jsx)(n.strong,{children:"Note"}),": In addition to the work directory Scrut also creates and cleans up a temporary directory, that is accessible via ",(0,i.jsx)(n.code,{children:"$TMPDIR"}),". Tools like ",(0,i.jsx)(n.code,{children:"mktemp"})," automatically use it (from said environment variable)."]}),"\n"]}),"\n",(0,i.jsx)(n.h2,{id:"test-execution",children:"Test execution"}),"\n",(0,i.jsxs)(n.p,{children:["As Scrut is primarily intended as an integration testing framework for CLI applications, it is tightly integrated with the shell.\nEach Scrut test must define a ",(0,i.jsx)(n.a,{href:"/scrut/docs/advanced/file-formats#test-case-anatomy",children:"shell expression"}),' (called an "execution").\nEach of those executions is then run within an actual shell (bash) process, as they would be when a human or automation would execute the expression manually on the shell.']}),"\n",(0,i.jsx)(n.p,{children:"With that in mind:"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:["Each execution from the same test file is executed in an individual shell process.","\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:["Scrut currently only supports ",(0,i.jsx)(n.code,{children:"bash"})," as shell process."]}),"\n",(0,i.jsxs)(n.li,{children:["Each subsequent execution within the same file inherits the state of the previous execution: environment variables, shell variables, functions, settings (",(0,i.jsx)(n.code,{children:"set"})," and ",(0,i.jsx)(n.code,{children:"shopt"}),")."]}),"\n"]}),"\n"]}),"\n",(0,i.jsx)(n.li,{children:"Tests within the same file are executed in sequential order."}),"\n",(0,i.jsxs)(n.li,{children:["Executions happen in a ",(0,i.jsx)(n.a,{href:"#test-work-directory",children:"temporary work directory"}),", that is initially empty and will be cleaned up after the last executions of the test file has run (or when executions are ",(0,i.jsx)(n.a,{href:"#skip-tests-with-exit-cod",children:"skipped"}),")."]}),"\n",(0,i.jsxs)(n.li,{children:["Executions may be detached, but Scrut will not clean up (kill) or wait for detached child processes","\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:["If you want to run your process in the background or detach, see the ",(0,i.jsx)(n.code,{children:"detached"})," setting in the ",(0,i.jsx)(n.a,{href:"/scrut/docs/advanced/file-formats#testcase-configuration",children:"testcase configuration"})," page."]}),"\n"]}),"\n"]}),"\n"]}),"\n",(0,i.jsx)(n.h3,{id:"execution-within-a-custom-shell",children:"Execution within a custom shell"}),"\n",(0,i.jsxs)(n.p,{children:["While Scrut currently only supports ",(0,i.jsx)(n.code,{children:"bash"})," (>= 3.2) a custom shell can be provided with the ",(0,i.jsx)(n.code,{children:"--shell"})," command line parameter.\nTo understand how that works consider the following:"]}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-bash",children:'$ echo "echo Hello" | /bin/bash -\nHello\n'})}),"\n",(0,i.jsxs)(n.p,{children:["What the above does is piping the string ",(0,i.jsx)(n.code,{children:"echo Hello"})," into the ",(0,i.jsx)(n.code,{children:"STDIN"})," of the process that was started with ",(0,i.jsx)(n.code,{children:"/bin/bash -"}),".\nScrut pretty much does the same with each shell expressions within a test file."]}),"\n",(0,i.jsxs)(n.p,{children:["So why provide a custom ",(0,i.jsx)(n.code,{children:"--shell"})," then?\nThis becomes useful in two scenarios:"]}),"\n",(0,i.jsxs)(n.ol,{children:["\n",(0,i.jsx)(n.li,{children:"You need to execute the same code before Scrut runs each individual expression"}),"\n",(0,i.jsx)(n.li,{children:"You need Scrut to execute each expression in some isolated environment"}),"\n"]}),"\n",(0,i.jsx)(n.p,{children:"For (1) consider the following code:"}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-bash",children:"#!/bin/bash\n\n# do something in this wrapper script\nsource /my/custom/setup.sh\nrun_my_custom_setup\n\n# consume and run STDIN\nsource /dev/stdin\n"})}),"\n",(0,i.jsx)(n.p,{children:"For (2) consider the following:"}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-bash",children:"#!/bin/bash\n\n# do something in this wrapper script\nsource /my/custom/setup.sh\nrun_my_custom_setup\n\n# end in a bash process that will receive STDIN\nexec ssh username@acme.tld /bin/bash\n"})}),"\n",(0,i.jsx)(n.p,{children:"Instead of SSHing into a machine, consider also running a bash process in docker container."}),"\n",(0,i.jsx)(n.h2,{id:"stdout-and-stderr",children:"STDOUT and STDERR"}),"\n",(0,i.jsxs)(n.p,{children:["Commands-line applications can generate output on to two streams: ",(0,i.jsx)(n.code,{children:"STDOUT"})," and ",(0,i.jsx)(n.code,{children:"STDERR"}),". There is no general agreement on which stream is supposed to contain what kind of data, but commonly ",(0,i.jsx)(n.code,{children:"STDOUT"})," contains the primary output and ",(0,i.jsx)(n.code,{children:"STDERR"})," contains logs, debug messages, etc. This is also the recommendation of the ",(0,i.jsx)(n.a,{href:"https://clig.dev/#:~:text=primary%20output%20for%20your%20command",children:"CLI guidelines"}),"."]}),"\n",(0,i.jsxs)(n.p,{children:["Scrut validates CLI output via ",(0,i.jsx)(n.a,{href:"/scrut/docs/advanced/expectations",children:"Expectations"}),". Which output that entails can be configured via the ",(0,i.jsxs)(n.a,{href:"/scrut/docs/advanced/file-formats#testcase-configuration",children:[(0,i.jsx)(n.code,{children:"output_stream"})," configuration directive"]})," (and the ",(0,i.jsx)(n.code,{children:"--(no-)combine-output"})," command-line parameters)."]}),"\n",(0,i.jsxs)(n.p,{children:[(0,i.jsx)(n.strong,{children:"Note:"})," While you can configure which output streams Scrut considers when evaluating output expecations, you can also steer this by using stream control bash primitives like ",(0,i.jsx)(n.code,{children:"some-command 2>&1"}),"."]}),"\n",(0,i.jsx)(n.h2,{id:"exit-codes",children:"Exit Codes"}),"\n",(0,i.jsx)(n.p,{children:"You can denote the expected exit code of a shell expression in a testcase. For example:"}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{children:"The command is expected to end with exit code 2\n\n```scrut\n$ some-command --foo\nan expected line of output\n[2]\n```\n"})}),"\n",(0,i.jsxs)(n.p,{children:["Unless otherwise specified an exit code of 0 (zero) is assumed. You can explicitly denote it with ",(0,i.jsx)(n.code,{children:"[0]"})," if you prefer."]}),"\n",(0,i.jsxs)(n.blockquote,{children:["\n",(0,i.jsxs)(n.p,{children:[(0,i.jsx)(n.strong,{children:"Note"}),": Exit code evaluation happens before output expectations are evaluated."]}),"\n"]}),"\n",(0,i.jsx)(n.h3,{id:"skip-tests-with-exit-code-80",children:"Skip Tests with Exit Code 80"}),"\n",(0,i.jsxs)(n.p,{children:["If any testcase in a test file exist with exit code ",(0,i.jsx)(n.code,{children:"80"}),", then all testcases in that file are skipped."]}),"\n",(0,i.jsx)(n.p,{children:"This is especially helpful for OS specific tests etc. Imagine:"}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{children:'Run tests in this file only on Mac\n\n```scrut\n$ [[ "$(uname)" == "Darwin" ]] || exit 80\n```\n'})}),"\n",(0,i.jsxs)(n.p,{children:[(0,i.jsx)(n.strong,{children:"Note:"})," The code that Scrut accepts to skip a whole file can be modified with the ",(0,i.jsxs)(n.a,{href:"/scrut/docs/advanced/file-formats#testcase-configuration",children:[(0,i.jsx)(n.code,{children:"skip_document_code"})," configuration directive"]}),"."]}),"\n",(0,i.jsx)(n.h3,{id:"scrut-exit-code",children:"Scrut Exit Code"}),"\n",(0,i.jsx)(n.p,{children:"Scrut itself communicates the outcome of executions with exit codes. Currently three possible exit codes are supported:"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"0"}),": Command succeeded, all is good (",(0,i.jsx)(n.code,{children:"scrut test"}),", ",(0,i.jsx)(n.code,{children:"scrut create"}),", ",(0,i.jsx)(n.code,{children:"scrut update"}),")"]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"1"}),": Command failed with error (",(0,i.jsx)(n.code,{children:"scrut test"}),", ",(0,i.jsx)(n.code,{children:"scrut create"}),", ",(0,i.jsx)(n.code,{children:"scrut update"}),")"]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"50"}),": Validation failed (",(0,i.jsx)(n.code,{children:"scrut test"})," only)"]}),"\n"]}),"\n",(0,i.jsx)(n.h2,{id:"newline-handling",children:"Newline handling"}),"\n",(0,i.jsxs)(n.p,{children:[(0,i.jsx)(n.a,{href:"https://en.wikipedia.org/wiki/Newline",children:"Newline"})," endings is a sad story in computer history. In Unix / MacOS ( / *BSD / Amiga / ..) the standard line ending is the line feed (LF) character ",(0,i.jsx)(n.code,{children:"\\n"}),". Windows (also Palm OS and OS/2?) infamously attempted to make a combination of carriage return (CR) and line feed the standard: CRLF (",(0,i.jsx)(n.code,{children:"\\r\\n"}),"). Everybody got mad and still is."]}),"\n",(0,i.jsxs)(n.p,{children:["See the ",(0,i.jsxs)(n.a,{href:"/scrut/docs/advanced/file-formats#testcase-configuration",children:[(0,i.jsx)(n.code,{children:"keep_crlf"})," configuration directive"]})," to understand how Scrut handles LF and CRLF and how you can modify the default behavior."]}),"\n",(0,i.jsx)(n.h2,{id:"execution-environment",children:"Execution Environment"}),"\n",(0,i.jsxs)(n.p,{children:["A ",(0,i.jsx)(n.a,{href:"/scrut/docs/advanced/file-formats",children:"Scrut test file"})," can contain arbitrary amounts of tests. Scrut provides a shared execution environment for all tests within a single file, which comes with certain behaviors and side-effects that should be known:"]}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.strong,{children:"Shared Shell Environment"}),": Each subsequent testcase in the same file inherits the shell environment of the previous testcase. This means: All environment variables, shell variables, aliases, functions, etc that have are set in test are available to the immediate following test.","\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.em,{children:"Exception"}),": Environments from ",(0,i.jsx)(n.a,{href:"/scrut/docs/advanced/file-formats#testcase-configuration",children:(0,i.jsx)(n.code,{children:"detached"})})," testcases are not passed along"]}),"\n"]}),"\n"]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.strong,{children:"Shared Ephemeral Directories"}),": Each testcase in the same test file executes in the the same work directory and is provided with the same temporary directory (",(0,i.jsx)(n.a,{href:"#test-environment-variables",children:(0,i.jsx)(n.code,{children:"$TEMPDIR"})}),"). Both directories will be removed (cleaned up) after test execution - independent of whether the test execution succeeds or fails.","\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.em,{children:"Exception"}),": If the ",(0,i.jsx)(n.code,{children:"--work-directory"})," command-line parameter is provided, then this directory will not be cleaned up (deleted) after execution. A temporary directory, that will be removed after execution, will be created within the working directory."]}),"\n"]}),"\n"]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.strong,{children:"Process Isolation"}),": Scrut starts individual ",(0,i.jsx)(n.code,{children:"bash"})," processes for executing each testcase of the same test file. Each shell expression. The environment of the previous execution is pulled in through a shared ",(0,i.jsx)(n.code,{children:"state"})," file, that contains all environment variables, shell variables, aliases, functions and settings as they were set when the the previous testcase execution ended.","\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.em,{children:"Exception"}),": All testcases in cram files are currently executed within the same ",(0,i.jsx)(n.code,{children:"bash"})," process - this is likely to change in the future."]}),"\n"]}),"\n"]}),"\n"]})]})}function h(e={}){const{wrapper:n}={...(0,s.a)(),...e.components};return n?(0,i.jsx)(n,{...e,children:(0,i.jsx)(a,{...e})}):a(e)}},11151:(e,n,t)=>{t.d(n,{Z:()=>d,a:()=>c});var i=t(67294);const s={},r=i.createContext(s);function c(e){const n=i.useContext(r);return i.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function d(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(s):e.components||s:c(e.components),i.createElement(r.Provider,{value:n},e.children)}}}]);