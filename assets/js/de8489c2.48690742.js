"use strict";(self.webpackChunkstaticdocs_starter=self.webpackChunkstaticdocs_starter||[]).push([[7732],{28453:(e,n,t)=>{t.d(n,{R:()=>i,x:()=>o});var s=t(96540);const r={},c=s.createContext(r);function i(e){const n=s.useContext(c);return s.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function o(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(r):e.components||r:i(e.components),s.createElement(c.Provider,{value:n},e.children)}},69594:(e,n,t)=>{t.r(n),t.d(n,{assets:()=>a,contentTitle:()=>o,default:()=>h,frontMatter:()=>i,metadata:()=>s,toc:()=>d});const s=JSON.parse('{"id":"reference/behavior/execution-model","title":"Execution Model","description":"A Scrut test document can contain arbitrary amounts of test cases. Scrut provides a shared execution environment for all executions from a single document, which results in certain behaviors and side-effects that should be known:","source":"@site/docs/reference/behavior/execution-model.md","sourceDirName":"reference/behavior","slug":"/reference/behavior/execution-model","permalink":"/scrut/docs/reference/behavior/execution-model","draft":false,"unlisted":false,"editUrl":"https://www.internalfb.com/code/fbsource/fbcode/clifoundation/scrut/website/docs/reference/behavior/execution-model.md","tags":[],"version":"current","frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Markdown Format","permalink":"/scrut/docs/reference/formats/markdown-format"},"next":{"title":"Exit Codes","permalink":"/scrut/docs/reference/behavior/exit-codes"}}');var r=t(74848),c=t(28453);const i={},o="Execution Model",a={},d=[{value:"Shared Shell Environment",id:"shared-shell-environment",level:2},{value:"Shared Ephemeral Directories",id:"shared-ephemeral-directories",level:2},{value:"Process Isolation",id:"process-isolation",level:2}];function l(e){const n={a:"a",admonition:"admonition",blockquote:"blockquote",code:"code",em:"em",h1:"h1",h2:"h2",header:"header",li:"li",p:"p",strong:"strong",ul:"ul",...(0,c.R)(),...e.components};return(0,r.jsxs)(r.Fragment,{children:[(0,r.jsx)(n.header,{children:(0,r.jsx)(n.h1,{id:"execution-model",children:"Execution Model"})}),"\n",(0,r.jsxs)(n.p,{children:["A Scrut ",(0,r.jsx)(n.a,{href:"/docs/reference/fundamentals/test-document/",children:"test document"})," can contain arbitrary amounts of ",(0,r.jsx)(n.a,{href:"/docs/reference/fundamentals/test-case/",children:"test cases"}),". Scrut provides a shared execution environment for all executions from a single document, which results in certain behaviors and side-effects that should be known:"]}),"\n",(0,r.jsx)(n.h2,{id:"shared-shell-environment",children:"Shared Shell Environment"}),"\n",(0,r.jsxs)(n.p,{children:["Each subsequent ",(0,r.jsx)(n.a,{href:"/docs/reference/fundamentals/test-case/",children:"test case"})," in the same document inherits the shell environment of the previous ",(0,r.jsx)(n.a,{href:"/docs/reference/fundamentals/test-case/",children:"test case"}),". This means: All environment variables, shell variables, aliases, functions, etc that have been set in one ",(0,r.jsx)(n.a,{href:"/docs/reference/fundamentals/test-case/",children:"test case"})," are available to the immediate following ",(0,r.jsx)(n.a,{href:"/docs/reference/fundamentals/test-case/",children:"test case"}),"."]}),"\n",(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsxs)(n.li,{children:["E.g. ",(0,r.jsx)(n.code,{children:"export FOO=bar"})," in one ",(0,r.jsx)(n.a,{href:"/docs/reference/fundamentals/test-case/",children:"test case"})," will still be set in the following ",(0,r.jsx)(n.a,{href:"/docs/reference/fundamentals/test-case/",children:"test case"}),"."]}),"\n",(0,r.jsxs)(n.li,{children:[(0,r.jsx)(n.em,{children:"Exception"}),": Environments set in ",(0,r.jsx)(n.a,{href:"/docs/reference/fundamentals/inline-configuration/",children:(0,r.jsx)(n.code,{children:"detached"})})," ",(0,r.jsx)(n.a,{href:"/docs/reference/fundamentals/test-case/",children:"test cases"})," are not inherited."]}),"\n"]}),"\n",(0,r.jsx)(n.h2,{id:"shared-ephemeral-directories",children:"Shared Ephemeral Directories"}),"\n",(0,r.jsxs)(n.p,{children:["Each ",(0,r.jsx)(n.a,{href:"/docs/reference/fundamentals/test-case/",children:"test cases"})," in the same document executes in the the same ",(0,r.jsx)(n.a,{href:"/docs/reference/behavior/working-directory/",children:"working directory"})," and is provided with the same temporary directory (",(0,r.jsx)(n.a,{href:"/docs/reference/fundamentals/environment-variables/",children:(0,r.jsx)(n.code,{children:"$TEMPDIR"})}),"). Both directories will be removed (cleaned up) after test execution - independent of whether the test execution succeeds or fails."]}),"\n",(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsxs)(n.li,{children:[(0,r.jsx)(n.em,{children:"Exception"}),": If the ",(0,r.jsx)(n.code,{children:"--work-directory"})," command-line parameter is provided, then this directory will not be cleaned up (deleted) after execution. A temporary directory, that still will be removed after execution, will be created within the working directory."]}),"\n"]}),"\n",(0,r.jsx)(n.h2,{id:"process-isolation",children:"Process Isolation"}),"\n",(0,r.jsxs)(n.p,{children:["Scrut starts individual ",(0,r.jsx)(n.code,{children:"bash"})," processes for executing each ",(0,r.jsx)(n.a,{href:"/docs/reference/fundamentals/shell-expression/",children:"shell expression"})," of each ",(0,r.jsx)(n.a,{href:"/docs/reference/fundamentals/test-case/",children:"test case"})," in the same document. The environment of the previous execution is pulled in through a shared ",(0,r.jsx)(n.code,{children:"state"})," file, that contains all environment variables, shell variables, aliases, functions and settings as they were set when the the previous ",(0,r.jsx)(n.a,{href:"/docs/reference/fundamentals/test-case/",children:"test case"})," execution ended."]}),"\n",(0,r.jsxs)(n.admonition,{title:"Markdown vs Cram",type:"warning",children:[(0,r.jsxs)(n.p,{children:[(0,r.jsx)(n.a,{href:"/docs/reference/formats/markdown-format/",children:"Markdown"})," is the default Scrut ",(0,r.jsx)(n.a,{href:"/docs/reference/fundamentals/test-document/",children:"test document"})," format. ",(0,r.jsx)(n.a,{href:"/docs/reference/formats/cram-format/",children:"Cram"})," is supported for legacy reasons. Hence it's legacy mode of execution is also respected. The main difference in Cram from the above is:"]}),(0,r.jsxs)(n.blockquote,{children:["\n",(0,r.jsxs)(n.p,{children:["Each execution from the same ",(0,r.jsx)(n.a,{href:"/docs/reference/fundamentals/test-document/",children:"test document"})," is executed ",(0,r.jsx)(n.em,{children:"in the same shell process"}),"."]}),"\n"]}),(0,r.jsxs)(n.p,{children:["This is less flexible (e.g. Scrut cannot constraint max execution time per ",(0,r.jsx)(n.a,{href:"/docs/reference/fundamentals/test-case/",children:"test case"}),") and more prone to unintended side-effects (e.g. ",(0,r.jsx)(n.code,{children:"set -e"})," terminating all test executions, not only a single test case or detached processes interfering with output association to specific tests). ",(0,r.jsx)(n.strong,{children:"We recommend to use Markdown"}),"."]})]})]})}function h(e={}){const{wrapper:n}={...(0,c.R)(),...e.components};return n?(0,r.jsx)(n,{...e,children:(0,r.jsx)(l,{...e})}):l(e)}}}]);