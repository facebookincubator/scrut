"use strict";(self.webpackChunkstaticdocs_starter=self.webpackChunkstaticdocs_starter||[]).push([[8432],{28453:(e,n,t)=>{t.d(n,{R:()=>o,x:()=>c});var s=t(96540);const a={},r=s.createContext(a);function o(e){const n=s.useContext(r);return s.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function c(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(a):e.components||a:o(e.components),s.createElement(r.Provider,{value:n},e.children)}},49564:(e,n,t)=>{t.r(n),t.d(n,{assets:()=>i,contentTitle:()=>c,default:()=>h,frontMatter:()=>o,metadata:()=>s,toc:()=>d});const s=JSON.parse('{"id":"reference/formats/markdown-format","title":"Markdown Format","description":"We chose Markdown as the primary test fle format for Scrut, because it is an amazingly simple, yet powerful language that is easily usable for humans. It is already supported by many tools and editors and it lends itself to write documentation and tests in the same location.","source":"@site/docs/reference/formats/markdown-format.md","sourceDirName":"reference/formats","slug":"/reference/formats/markdown-format","permalink":"/scrut/docs/reference/formats/markdown-format","draft":false,"unlisted":false,"editUrl":"https://www.internalfb.com/code/fbsource/fbcode/clifoundation/scrut/website/docs/reference/formats/markdown-format.md","tags":[],"version":"current","frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Cram Format","permalink":"/scrut/docs/reference/formats/cram-format"},"next":{"title":"Execution Model","permalink":"/scrut/docs/reference/behavior/execution-model"}}');var a=t(74848),r=t(28453);const o={},c="Markdown Format",i={},d=[{value:"Test Case Anatomy",id:"test-case-anatomy",level:2},{value:"Constraints",id:"constraints",level:2},{value:"Configuration",id:"configuration",level:2}];function l(e){const n={a:"a",admonition:"admonition",code:"code",em:"em",h1:"h1",h2:"h2",header:"header",li:"li",p:"p",pre:"pre",strong:"strong",ul:"ul",...(0,r.R)(),...e.components};return(0,a.jsxs)(a.Fragment,{children:[(0,a.jsx)(n.header,{children:(0,a.jsx)(n.h1,{id:"markdown-format",children:"Markdown Format"})}),"\n",(0,a.jsxs)(n.p,{children:["We chose ",(0,a.jsx)(n.a,{href:"https://www.markdownguide.org/",children:"Markdown"})," as the primary test fle format for Scrut, because it is an amazingly simple, yet powerful language that is easily usable for humans. It is already supported by many tools and editors and it lends itself to write documentation and tests in the same location."]}),"\n",(0,a.jsxs)(n.p,{children:["A markdown ",(0,a.jsx)(n.a,{href:"/docs/reference/fundamentals/test-document/",children:"test document"}),' differs from a "normal" Markdown only in one way: It contains ',(0,a.jsx)(n.a,{href:"https://www.markdownguide.org/basic-syntax/#code",children:"code blocks"})," that are annotated with the ",(0,a.jsx)(n.code,{children:"scrut"})," language:"]}),"\n",(0,a.jsx)(n.pre,{children:(0,a.jsx)(n.code,{className:"language-markdown",metastring:"showLineNumbers",children:'# This is a normal markdown document\n\n```scrut\n$ some command\nsome output\n```\n\n\ud83d\udc46 code block is a Scrut test case,\n   because it is annotated with the `scrut` language.\n\n\ud83d\udc47 code block is NOT a Scrut test case,\n   because it is not annotated with the `scrut` language.\n\n```python\nprint("I am a snek")\n```\n'})}),"\n",(0,a.jsx)(n.h2,{id:"test-case-anatomy",children:"Test Case Anatomy"}),"\n",(0,a.jsxs)(n.p,{children:["A ",(0,a.jsx)(n.a,{href:"/docs/reference/fundamentals/test-case/",children:"test case"})," in Markdown is structured as follows:"]}),"\n",(0,a.jsxs)(n.ul,{children:["\n",(0,a.jsxs)(n.li,{children:[(0,a.jsx)(n.a,{href:"/docs/reference/fundamentals/shell-expression/",children:"shell expressions"})," and ",(0,a.jsx)(n.a,{href:"/docs/reference/fundamentals/output-expectations/",children:"output expectations"})," live in the same code-block, that must be annotated with the language ",(0,a.jsx)(n.code,{children:"scrut"}),"\n",(0,a.jsxs)(n.ul,{children:["\n",(0,a.jsxs)(n.li,{children:["The first line of a ",(0,a.jsx)(n.a,{href:"/docs/reference/fundamentals/shell-expression/",children:"shell expressions"})," must start with ",(0,a.jsx)(n.code,{children:"$ "})," (dollar, sign followed by a space), any subsequent with ",(0,a.jsx)(n.code,{children:"> "})," (closing angle bracket / chevron, followed by a space)"]}),"\n",(0,a.jsxs)(n.li,{children:["All other lines in the code block (including empty ones) that follow the ",(0,a.jsx)(n.a,{href:"/docs/reference/fundamentals/shell-expression/",children:"shell expression"})," are considered ",(0,a.jsx)(n.a,{href:"/docs/reference/fundamentals/output-expectations/",children:"output expectations"})]}),"\n",(0,a.jsxs)(n.li,{children:["Lines starting with ",(0,a.jsx)(n.code,{children:"#"})," that precede the ",(0,a.jsx)(n.a,{href:"/docs/reference/fundamentals/shell-expression/",children:"shell expression"})," are ignored (comments)"]}),"\n",(0,a.jsxs)(n.li,{children:["If an ",(0,a.jsx)(n.a,{href:"/docs/reference/behavior/exit-codes/",children:"exit code"})," other than ",(0,a.jsx)(n.code,{children:"0"})," is expected, it can be denoted in square brackets ",(0,a.jsx)(n.code,{children:"[123]"})," once per ",(0,a.jsx)(n.a,{href:"/docs/reference/fundamentals/test-case/",children:"test case"})]}),"\n"]}),"\n"]}),"\n",(0,a.jsxs)(n.li,{children:["The first line before the code block that is either a paragraph or a header will be used as the ",(0,a.jsx)(n.em,{children:"title"})," of the ",(0,a.jsx)(n.a,{href:"/docs/reference/fundamentals/test-case/",children:"test case"})]}),"\n"]}),"\n",(0,a.jsx)(n.p,{children:"Here an example:"}),"\n",(0,a.jsx)(n.pre,{children:(0,a.jsx)(n.code,{className:"language-markdown",metastring:"showLineNumbers",children:"This is the title\n\n```scrut\n$ command | \\\n>   other-command\nexpected output line\nanother expected output line\n[123]\n```\n"})}),"\n",(0,a.jsx)(n.h2,{id:"constraints",children:"Constraints"}),"\n",(0,a.jsxs)(n.p,{children:["The following ",(0,a.jsx)(n.strong,{children:"constraints"})," apply:"]}),"\n",(0,a.jsxs)(n.ul,{children:["\n",(0,a.jsxs)(n.li,{children:["A markdown document can contain as many ",(0,a.jsx)(n.a,{href:"/docs/reference/fundamentals/test-case/",children:"test cases"})," as needed (0..n)"]}),"\n",(0,a.jsxs)(n.li,{children:["Each code block in a ",(0,a.jsx)(n.a,{href:"/docs/reference/fundamentals/test-case/",children:"test case"})," may only have ",(0,a.jsx)(n.em,{children:"one"})," (1) ",(0,a.jsx)(n.a,{href:"/docs/reference/fundamentals/shell-expression/",children:"shell expression"})," (each ",(0,a.jsx)(n.a,{href:"/docs/reference/fundamentals/test-case/",children:"test case"})," is considered atomic)"]}),"\n",(0,a.jsxs)(n.li,{children:["Code blocks that do not denote a language (or a different language than ",(0,a.jsx)(n.code,{children:"scrut"}),") will be ignored"]}),"\n"]}),"\n",(0,a.jsxs)(n.p,{children:["With that in mind, consider the following markdown document that contains not only ",(0,a.jsx)(n.a,{href:"/docs/reference/fundamentals/test-case/",children:"test cases"})," but arbitrary other text and other code blocks. This is idiomatic Scrut markdown document that combines tests and documentation:"]}),"\n",(0,a.jsx)(n.pre,{children:(0,a.jsx)(n.code,{className:"language-markdown",children:'# This is just regular markdown\n\nIt contains both Scrut tests **and**  abitrary text, including code examples,\nthat are unrelated to Scrut.\n\n```python\nimport os\n\nprint("This code block ignored by Scrut")\n```\n\n## Here is a scrut test\n\n```scrut\n$ echo Hello\nHello\n```\n\n## Embedded with other documentation\n\nSo it\'s a mix of test and not tests.\n\nAny amount of tests are fine:\n\n```scrut\n$ echo World\nWorld\n```\n\nJust make sure to write only one [test case](/docs/reference/fundamentals/test-case/) per code-block.\n'})}),"\n",(0,a.jsx)(n.admonition,{type:"note",children:(0,a.jsx)(n.p,{children:"If you are testing actual markdown output, be aware that you can embed code blocks in other code blocks, if the outer code block uses one more backtick (opening and closing!) than the embedded one(s). Just have a look at the source code of this document right above this text."})}),"\n",(0,a.jsx)(n.h2,{id:"configuration",children:"Configuration"}),"\n",(0,a.jsxs)(n.p,{children:["Markdown ",(0,a.jsx)(n.a,{href:"/docs/reference/fundamentals/test-document/",children:"test documents"})," may contain inline configuration. Read more in ",(0,a.jsx)(n.a,{href:"/docs/reference/fundamentals/inline-configuration/",children:"Reference > Fundamentals > Inline Configuration"}),"."]})]})}function h(e={}){const{wrapper:n}={...(0,r.R)(),...e.components};return n?(0,a.jsx)(n,{...e,children:(0,a.jsx)(l,{...e})}):l(e)}}}]);