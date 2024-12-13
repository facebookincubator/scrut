"use strict";(self.webpackChunkstaticdocs_starter=self.webpackChunkstaticdocs_starter||[]).push([[5241],{90104:(e,n,t)=>{t.r(n),t.d(n,{assets:()=>l,contentTitle:()=>d,default:()=>h,frontMatter:()=>o,metadata:()=>s,toc:()=>a});const s=JSON.parse('{"id":"advanced/file-formats","title":"File Formats","description":"Scrut supports multiple test file formats. The recommended format is Markdown.","source":"@site/docs/advanced/file-formats.md","sourceDirName":"advanced","slug":"/advanced/file-formats","permalink":"/scrut/docs/advanced/file-formats","draft":false,"unlisted":false,"editUrl":"https://www.internalfb.com/code/fbsource/fbcode/clifoundation/scrut/website/docs/advanced/file-formats.md","tags":[],"version":"current","sidebarPosition":1,"frontMatter":{"sidebar_position":1},"sidebar":"tutorialSidebar","previous":{"title":"Tutorial","permalink":"/scrut/docs/tutorial"},"next":{"title":"Expectations","permalink":"/scrut/docs/advanced/expectations"}}');var i=t(74848),r=t(28453);const o={sidebar_position:1},d="File Formats",l={},a=[{value:"File Anatomy",id:"file-anatomy",level:2},{value:"Test Case Anatomy",id:"test-case-anatomy",level:3},{value:"Markdown Format",id:"markdown-format",level:2},{value:"Inline Configuration",id:"inline-configuration",level:3},{value:"Document Configuration",id:"document-configuration",level:4},{value:"TestCase Configuration",id:"testcase-configuration",level:4},{value:"Wait Configuration",id:"wait-configuration",level:3},{value:"Cram Format",id:"cram-format",level:2},{value:"Which format to chose?",id:"which-format-to-chose",level:2}];function c(e){const n={a:"a",blockquote:"blockquote",code:"code",em:"em",h1:"h1",h2:"h2",h3:"h3",h4:"h4",header:"header",li:"li",ol:"ol",p:"p",pre:"pre",strong:"strong",table:"table",tbody:"tbody",td:"td",th:"th",thead:"thead",tr:"tr",ul:"ul",...(0,r.R)(),...e.components};return(0,i.jsxs)(i.Fragment,{children:[(0,i.jsx)(n.header,{children:(0,i.jsx)(n.h1,{id:"file-formats",children:"File Formats"})}),"\n",(0,i.jsxs)(n.p,{children:["Scrut supports multiple test file formats. The recommended format is ",(0,i.jsx)(n.a,{href:"#markdown-format",children:"Markdown"}),"."]}),"\n",(0,i.jsx)(n.h2,{id:"file-anatomy",children:"File Anatomy"}),"\n",(0,i.jsx)(n.p,{children:"All test files contain one or more test cases. There are two common patterns to structure test files in Scrut:"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.strong,{children:"Coherent Test Suite"})," (recommended): One test file represents one use-case or behavior. This makes it easy to identify broken functionality."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.strong,{children:"List of Tests"}),": One test file contains a list of simple, not necessarily related tests."]}),"\n"]}),"\n",(0,i.jsxs)(n.p,{children:["Markdown files support ",(0,i.jsx)(n.a,{href:"#inline-configuration",children:"document wide configuration"}),' in the form of "YAML Frontmatter".']}),"\n",(0,i.jsx)(n.h3,{id:"test-case-anatomy",children:"Test Case Anatomy"}),"\n",(0,i.jsxs)(n.p,{children:["Each individual test that lives in a test file is called a ",(0,i.jsx)(n.em,{children:"Test Case"})," and consists of the following components:"]}),"\n",(0,i.jsxs)(n.ol,{children:["\n",(0,i.jsxs)(n.li,{children:["A ",(0,i.jsx)(n.strong,{children:"Title"}),", so that a human can understand what is being done"]}),"\n",(0,i.jsxs)(n.li,{children:["A ",(0,i.jsx)(n.strong,{children:"Shell Expression"}),", that can be anything from a single command to a multi-line, multi-piped expression"]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.strong,{children:(0,i.jsx)(n.a,{href:"/scrut/docs/advanced/expectations",children:"Expectations"})})," of the output that the Shell Expression will yield"]}),"\n",(0,i.jsxs)(n.li,{children:["Optionally the expected ",(0,i.jsx)(n.em,{children:"Exit Code"})," the Shell Expression must end in - if anything but successful execution (",(0,i.jsx)(n.code,{children:"0"}),") is expected"]}),"\n",(0,i.jsx)(n.li,{children:"Optionally per-test-case configuration (only supported by Markdown format)"}),"\n"]}),"\n",(0,i.jsx)(n.h2,{id:"markdown-format",children:"Markdown Format"}),"\n",(0,i.jsxs)(n.p,{children:[(0,i.jsx)(n.a,{href:"https://www.markdownguide.org/",children:"Markdown"})," is an amazingly simple, yet powerful language. To write ",(0,i.jsx)(n.em,{children:"Test Cases"})," in Markdown follow this guidance:"]}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.em,{children:"Shell Expressions"})," and ",(0,i.jsx)(n.em,{children:"Expectations"})," live in the same code-block, that must be annotated with the language ",(0,i.jsx)(n.code,{children:"scrut"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:["The first line of a ",(0,i.jsx)(n.em,{children:"Shell Expressions"})," must start with ",(0,i.jsx)(n.code,{children:"$ "})," (dollar, sign followed by a space), any subsequent with ",(0,i.jsx)(n.code,{children:"> "})," (closing angle bracket / chevron, followed by a space)"]}),"\n",(0,i.jsxs)(n.li,{children:["All other lines in the code block (including empty ones) that follow the ",(0,i.jsx)(n.em,{children:"Shell Expression"})," are considered ",(0,i.jsx)(n.em,{children:"Expectations"})]}),"\n",(0,i.jsxs)(n.li,{children:["Lines starting with ",(0,i.jsx)(n.code,{children:"#"})," that precede the shell expression are ignored (comments)"]}),"\n",(0,i.jsxs)(n.li,{children:["If an ",(0,i.jsx)(n.em,{children:"Exit Code"})," other than 0 is expected, it can be denoted in square brackets ",(0,i.jsx)(n.code,{children:"[123]"})," once per ",(0,i.jsx)(n.em,{children:"Test Case"})]}),"\n"]}),"\n"]}),"\n",(0,i.jsxs)(n.li,{children:["The first line before the code block that is either a paragraph or a header will be used as the ",(0,i.jsx)(n.em,{children:"Title"})," of the ",(0,i.jsx)(n.em,{children:"Test Case"})]}),"\n"]}),"\n",(0,i.jsx)(n.p,{children:"Here an example:"}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-markdown",children:"This is the title\n\n```scrut\n$ command | \\\n>   other-command\nexpected output line\nanother expected output line\n[123]\n```\n"})}),"\n",(0,i.jsxs)(n.p,{children:["The following ",(0,i.jsx)(n.strong,{children:"constraints"})," apply:"]}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsx)(n.li,{children:"A markdown file can contain as many Test Cases as needed (1..n)"}),"\n",(0,i.jsxs)(n.li,{children:["Each code block in a Test Case may only have ",(0,i.jsx)(n.em,{children:"one"})," (1) Shell Expression (each Test Case is considered atomic)"]}),"\n",(0,i.jsxs)(n.li,{children:["Code blocks that do not denote a language (or a different language than ",(0,i.jsx)(n.code,{children:"scrut"}),") will be ignored"]}),"\n"]}),"\n",(0,i.jsx)(n.p,{children:"With that in mind, consider the following markdown file that contains not only Test Cases but arbitrary other text and other code blocks. This is idiomatic Scrut markdown files that combines tests and documentation:"}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{children:'# This is just regular markdown\n\nIt contains both Scrut tests **and**  abitrary text, including code examples,\nthat are unrelated to Scrut.\n\n```python\nimport os\n\nprint("This code block ignored by Scrut")\n```\n\n## Here is a scrut test\n\n```scrut\n$ echo Hello\nHello\n```\n\n## Embedded with other documentation\n\nSo it\'s a mix of test and not tests.\n\nAny amount of tests are fine:\n\n```scrut\n$ echo World\nWorld\n```\n\nJust make sure to write only one Test Case per code-block.\n'})}),"\n",(0,i.jsxs)(n.blockquote,{children:["\n",(0,i.jsxs)(n.p,{children:[(0,i.jsx)(n.strong,{children:"Note"}),": If you are testing actual markdown output, be aware that you can embed code blocks in other code blocks, if the outer code block uses one more backtick (opening and closing!) than the embedded one(s). Just have a look at the source code of this file right above this text."]}),"\n"]}),"\n",(0,i.jsx)(n.h3,{id:"inline-configuration",children:"Inline Configuration"}),"\n",(0,i.jsx)(n.p,{children:"Scrut supports two kinds of inline configuration:"}),"\n",(0,i.jsxs)(n.ol,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.strong,{children:"Per Document"})," (document-wide) configuration, which can be defined at the start of the test file"]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.strong,{children:"Per Test Case"})," (test-case-wide) configuration, which can be defined with each individual Test Case"]}),"\n"]}),"\n",(0,i.jsx)(n.p,{children:(0,i.jsx)(n.strong,{children:"Example"})}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-markdown",children:"---\n# document-wide YAML configuration\ntotal_timeout: 30s\n---\n\n# The test document\n\nThe initial block that is initialized with `---` and terminated with `---` contains the configuration in YAML notation.\n\n## A simple test\n\n```scrut\n$ echo Hello One\nHello One\n```\n\nThe above test does not contain any per-test configuration\n\n## A test with configuration\n\n```scrut {timeout: 10s}\n$ echo Hello Two\nHello Two\n```\n\nThe above test contains per-test configuration\n"})}),"\n",(0,i.jsx)(n.p,{children:"Some inline-configuration attribute can overwritten by parameters provided on the command-line. The order of precedence is:"}),"\n",(0,i.jsxs)(n.ol,{children:["\n",(0,i.jsx)(n.li,{children:"Command-line parameter"}),"\n",(0,i.jsx)(n.li,{children:"Per-TestCase configuration"}),"\n",(0,i.jsx)(n.li,{children:"Per-Document configuration"}),"\n",(0,i.jsx)(n.li,{children:"Default"}),"\n"]}),"\n",(0,i.jsx)(n.h4,{id:"document-configuration",children:"Document Configuration"}),"\n",(0,i.jsxs)(n.table,{children:[(0,i.jsx)(n.thead,{children:(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.th,{children:"Name"}),(0,i.jsx)(n.th,{children:"Type"}),(0,i.jsx)(n.th,{children:"Corresponding Command Line Parameter"}),(0,i.jsx)(n.th,{children:"Description"})]})}),(0,i.jsxs)(n.tbody,{children:[(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{children:(0,i.jsx)(n.code,{children:"append"})}),(0,i.jsx)(n.td,{children:"list of strings"}),(0,i.jsx)(n.td,{children:(0,i.jsx)(n.code,{children:"--append-test-file-paths"})}),(0,i.jsxs)(n.td,{children:["Include these paths in order, as if they were part of this file. All tests within the appended paths are appended to the tests defined in this file. Use-case is common/shared test tear-down. Paths must be relative to the current ",(0,i.jsx)(n.code,{children:"$TESTDIR"}),"."]})]}),(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{children:(0,i.jsx)(n.code,{children:"defaults"})}),(0,i.jsx)(n.td,{children:(0,i.jsx)(n.a,{href:"#testcase-configuration",children:"TestCase Configuration"})}),(0,i.jsx)(n.td,{children:"n/a"}),(0,i.jsx)(n.td,{children:"Defaults for per-test-case configuration within the test file."})]}),(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{children:(0,i.jsx)(n.code,{children:"prepend"})}),(0,i.jsx)(n.td,{children:"list of strings"}),(0,i.jsx)(n.td,{children:(0,i.jsx)(n.code,{children:"--prepend-test-file-paths"})}),(0,i.jsxs)(n.td,{children:["Include these paths in order, as if they were part of this file. All tests within the prepend paths are prepended to the tests defined in this file. Use-case is common/shared test setup. Paths must be relative to the current ",(0,i.jsx)(n.code,{children:"$TESTDIR"}),"."]})]}),(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{children:(0,i.jsx)(n.code,{children:"shell"})}),(0,i.jsx)(n.td,{children:"string"}),(0,i.jsx)(n.td,{children:(0,i.jsx)(n.code,{children:"--shell"})}),(0,i.jsxs)(n.td,{children:["The path to the shell. If a full path is not provided, then the command must be in ",(0,i.jsx)(n.code,{children:"$PATH"}),". ",(0,i.jsxs)(n.strong,{children:["Only ",(0,i.jsx)(n.code,{children:"bash"})," compatible shells are currently supported!"]})]})]}),(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{children:(0,i.jsx)(n.code,{children:"total_timeout"})}),(0,i.jsx)(n.td,{children:(0,i.jsx)(n.a,{href:"https://docs.rs/humantime/latest/humantime/",children:"duration string"})}),(0,i.jsx)(n.td,{children:(0,i.jsx)(n.code,{children:"--timeout-seconds"})}),(0,i.jsx)(n.td,{children:"All tests within the file (including appended and prepended) must finish executing within this time."})]})]})]}),"\n",(0,i.jsx)(n.p,{children:(0,i.jsx)(n.strong,{children:"Defaults (Markdown and Cram)"})}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-yaml",children:"append: []\ndefaults: {}\nprepend: []\nshell: bash\ntotal_timeout: 15m\n"})}),"\n",(0,i.jsx)(n.p,{children:(0,i.jsx)(n.strong,{children:"Caveats"})}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsx)(n.li,{children:"Per-document configuration in files that are appended or prepended is ignored"}),"\n"]}),"\n",(0,i.jsx)(n.h4,{id:"testcase-configuration",children:"TestCase Configuration"}),"\n",(0,i.jsxs)(n.table,{children:[(0,i.jsx)(n.thead,{children:(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.th,{children:"Name"}),(0,i.jsx)(n.th,{children:"Type"}),(0,i.jsx)(n.th,{children:"Corresponding Command Line Parameter"}),(0,i.jsx)(n.th,{children:"Description"})]})}),(0,i.jsxs)(n.tbody,{children:[(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{children:(0,i.jsx)(n.code,{children:"detached"})}),(0,i.jsx)(n.td,{children:"boolean"}),(0,i.jsx)(n.td,{children:"n/a"}),(0,i.jsxs)(n.td,{children:["Tell Scrut that the shell expression of this test will detach itself, so Scrut will not consider this a test (i.e. no output or exit code evaluation). Purpose is to allow the user to detach a command (like ",(0,i.jsx)(n.code,{children:"nohup some-command &"}),") that is doing something asynchronous (e.g. starting a server to which the tested CLI is a client)."]})]}),(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{children:(0,i.jsx)(n.code,{children:"environment"})}),(0,i.jsx)(n.td,{children:"object"}),(0,i.jsx)(n.td,{children:"n/a"}),(0,i.jsx)(n.td,{children:"A set of environment variable names and values that will be explicitly set for the test."})]}),(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{children:(0,i.jsx)(n.code,{children:"keep_crlf"})}),(0,i.jsx)(n.td,{children:"boolean"}),(0,i.jsx)(n.td,{children:(0,i.jsx)(n.code,{children:"--keep-output-crlf"})}),(0,i.jsx)(n.td,{children:"Whether CRLF should be translated to LF (=false) or whether CR needs to be explicitly handled (=true)."})]}),(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{children:(0,i.jsx)(n.code,{children:"output_stream"})}),(0,i.jsxs)(n.td,{children:["enum (",(0,i.jsx)(n.code,{children:"stdout"}),", ",(0,i.jsx)(n.code,{children:"stderr"}),", ",(0,i.jsx)(n.code,{children:"combined"}),")"]}),(0,i.jsxs)(n.td,{children:[(0,i.jsx)(n.code,{children:"--combine-output"})," and ",(0,i.jsx)(n.code,{children:"--no-combine-output"})]}),(0,i.jsxs)(n.td,{children:["Which output stream to choose when applying output expectations: ",(0,i.jsx)(n.code,{children:"stdout"})," (all expectations apply to what is printed on STDOUT), ",(0,i.jsx)(n.code,{children:"stderr"})," (all expectations apply to what is printed on STDERR), ",(0,i.jsx)(n.code,{children:"combined"})," (STDOUT and STDERR will combined into a single stream where all expectations are applied on)"]})]}),(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{children:(0,i.jsx)(n.code,{children:"skip_document_code"})}),(0,i.jsx)(n.td,{children:"positive integer"}),(0,i.jsx)(n.td,{children:"n/a"}),(0,i.jsx)(n.td,{children:"The exit code, that if returned by any test, leads to skipping of the whole file."})]}),(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{children:(0,i.jsx)(n.code,{children:"timeout"})}),(0,i.jsxs)(n.td,{children:["null or ",(0,i.jsx)(n.a,{href:"https://docs.rs/humantime/latest/humantime/",children:"duration string"})]}),(0,i.jsx)(n.td,{children:"n/a"}),(0,i.jsx)(n.td,{children:"A max execution time a test can run before it is considered failed (and will be aborted)."})]}),(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{children:(0,i.jsx)(n.code,{children:"wait"})}),(0,i.jsxs)(n.td,{children:["null or ",(0,i.jsx)(n.a,{href:"https://docs.rs/humantime/latest/humantime/",children:"duration string"})," or ",(0,i.jsx)(n.a,{href:"#wait-configuration",children:"Wait Configuration"})]}),(0,i.jsx)(n.td,{children:"n/a"}),(0,i.jsxs)(n.td,{children:["See ",(0,i.jsx)(n.a,{href:"#wait-configuration",children:"Wait Configuration"})]})]})]})]}),"\n",(0,i.jsx)(n.p,{children:(0,i.jsx)(n.strong,{children:"Defaults (Markdown)"})}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-yaml",children:"detached: false\nenvironment: {}\nkeep_crlf: false\noutput_stream: stdout\nskip_document_code: 80\ntimeout: null\nwait: null\n"})}),"\n",(0,i.jsx)(n.p,{children:(0,i.jsx)(n.strong,{children:"Defaults (Cram)"})}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-yaml",children:"detached: false\nenvironment: {}\nkeep_crlf: true\noutput_stream: combined\nskip_document_code: 80\ntimeout: null\nwait: null\n"})}),"\n",(0,i.jsx)(n.h3,{id:"wait-configuration",children:"Wait Configuration"}),"\n",(0,i.jsxs)(n.p,{children:["This configuration corresponds to the per-test-case ",(0,i.jsx)(n.code,{children:"detached"})," configuration and helps to write client / server tests where first a server is started (i.e. a test that runs detached) and then a client communicates with the server (i.e. a test that waits)"]}),"\n",(0,i.jsxs)(n.table,{children:[(0,i.jsx)(n.thead,{children:(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.th,{children:"Name"}),(0,i.jsx)(n.th,{children:"Type"}),(0,i.jsx)(n.th,{children:"Description"})]})}),(0,i.jsxs)(n.tbody,{children:[(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{children:(0,i.jsx)(n.code,{children:"timeout"})}),(0,i.jsx)(n.td,{children:(0,i.jsx)(n.a,{href:"https://docs.rs/humantime/latest/humantime/",children:"duration string"})}),(0,i.jsx)(n.td,{children:"How long to wait for the test to run."})]}),(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{children:(0,i.jsx)(n.code,{children:"path"})}),(0,i.jsx)(n.td,{children:"null or string"}),(0,i.jsxs)(n.td,{children:["If set then the wait will end early once the path exists. This path must be in ",(0,i.jsx)(n.code,{children:"$TMPDIR"})]})]})]})]}),"\n",(0,i.jsx)(n.p,{children:(0,i.jsx)(n.strong,{children:"Example"})}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-markdown",children:'# A server/client test example\n\nShow-case how a server/client test that initially starts a server\n\n## Start a server\n\n```scrut {detached: true}\n$ my-server --start && touch "$TMPDIR"/server-started\n```\n\n## Run client test once server is up\n\n```scrut {wait: {timeout: 5m, path: server-started}}\n$ my-client --do-a-thing\n```\n'})}),"\n",(0,i.jsx)(n.h2,{id:"cram-format",children:"Cram Format"}),"\n",(0,i.jsxs)(n.p,{children:["Also supported, for compatibility, is the Cram file format. The general guidance to write ",(0,i.jsx)(n.em,{children:"Test Cases"})," in Cram files is:"]}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:["The first line of ",(0,i.jsx)(n.em,{children:"Shell Expression"})," must start with ",(0,i.jsx)(n.code,{children:" $"})," (space + space + dollar + space), any subsequent with ",(0,i.jsx)(n.code,{children:" >"})," (space + space + closing angle bracket + space)","\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsx)(n.li,{children:"This is slightly different from classic scrut syntax. Be mindful of the additional spaces"}),"\n"]}),"\n"]}),"\n",(0,i.jsxs)(n.li,{children:["Lines following the ",(0,i.jsx)(n.em,{children:"Shell Expression"}),", that are also indented with two spaces, are considered ",(0,i.jsx)(n.em,{children:"Expectations"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:["If an Exit Code other than 0 is expected, it can be denoted in square brackets ",(0,i.jsx)(n.code,{children:" [123]"})," once per Test Case"]}),"\n",(0,i.jsxs)(n.li,{children:["Note: Empty output lines (=empty ",(0,i.jsx)(n.em,{children:"Expectations"}),") must still have two leading space characters"]}),"\n",(0,i.jsxs)(n.li,{children:["Note: A fully empty line (no leading spaces) denotes the end of the current ",(0,i.jsx)(n.em,{children:"Test Case"})]}),"\n"]}),"\n"]}),"\n",(0,i.jsxs)(n.li,{children:["If the ",(0,i.jsx)(n.em,{children:"Shell Expression"})," is preceded by a non-empty line (that is ",(0,i.jsx)(n.em,{children:"not"})," indented) the line is considered the ",(0,i.jsx)(n.em,{children:"Title"})," of the ",(0,i.jsx)(n.em,{children:"Test Case"})]}),"\n"]}),"\n",(0,i.jsx)(n.p,{children:"Here an example:"}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-cram",children:"This is a comment\n  $ scrut --help\n  Scrut help output\n\nAnother Test Case in the same file\n  $ scrut --version\n  Scrut version output\n"})}),"\n",(0,i.jsx)(n.p,{children:"Multiple tests Test Cases can be written in sequence, without any empty lines in between:"}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-cram",children:"A title for the first Test Case\n  $ first --command\n  $ second --command\n  $ third --comand\n  Output Expectation\n"})}),"\n",(0,i.jsxs)(n.blockquote,{children:["\n",(0,i.jsxs)(n.p,{children:[(0,i.jsx)(n.strong,{children:"Note"}),": Remember the indenting space characters!"]}),"\n"]}),"\n",(0,i.jsx)(n.h2,{id:"which-format-to-chose",children:"Which format to chose?"}),"\n",(0,i.jsx)(n.p,{children:"We recommend the Markdown format which was introduced with two goals in mind:"}),"\n",(0,i.jsxs)(n.ol,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.strong,{children:"Tests \u2764\ufe0f Documentation"}),": The value of tests is not only in proving behavior, but also in documenting it - and thereby also in teaching it. The Markdown Test Case format allows you to keep tests around in a way that future generations of maintainers will love you for."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.strong,{children:"Bad Spaces \ud83d\udc7e"}),": To denote an expected empty line of output in Cram format you have to provide two empty spaces ",(0,i.jsx)(n.code,{children:" "}),". This goes counter a lot of default behavior in the development toolchain. Many CI/CD tools are tuned to automatically ignore changes that only pertain spaces. Code review tools often deliberately hide those changes. Spaces are generally hard to see in code editors - if they are visualized at all. Breaking tests that are caused by an accidentally removed or added space cause rage quitting."]}),"\n"]}),"\n",(0,i.jsx)(n.p,{children:"If these arguments resonate with you, go for the Markdown format. If not you are probably better of with Cram that allows for a more condensed writing style. Choices, choices."})]})}function h(e={}){const{wrapper:n}={...(0,r.R)(),...e.components};return n?(0,i.jsx)(n,{...e,children:(0,i.jsx)(c,{...e})}):c(e)}}}]);