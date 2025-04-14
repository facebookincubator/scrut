"use strict";(self.webpackChunkstaticdocs_starter=self.webpackChunkstaticdocs_starter||[]).push([[4077],{24286:(e,t,n)=>{n.d(t,{Ay:()=>l,RM:()=>o});var i=n(74848),s=n(28453);const o=[];function a(e){return(0,i.jsx)(i.Fragment,{})}function l(e={}){const{wrapper:t}={...(0,s.R)(),...e.components};return t?(0,i.jsx)(t,{...e,children:(0,i.jsx)(a,{...e})}):a()}},27321:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>c,contentTitle:()=>d,default:()=>u,frontMatter:()=>l,metadata:()=>i,toc:()=>r});const i=JSON.parse('{"id":"tutorial/basic-expectations","title":"Basic Expectations","description":"The smoke test from the previous chapter validates that executing jq --version will output the string jq-1.7. While this is a good start, it also has a few problems:","source":"@site/docs/tutorial/03-basic-expectations.md","sourceDirName":"tutorial","slug":"/tutorial/basic-expectations","permalink":"/scrut/docs/tutorial/basic-expectations","draft":false,"unlisted":false,"editUrl":"https://www.internalfb.com/code/fbsource/fbcode/clifoundation/scrut/website/docs/tutorial/03-basic-expectations.md","tags":[],"version":"current","sidebarPosition":3,"frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Test Creation","permalink":"/scrut/docs/tutorial/create-test"},"next":{"title":"Output Expectations","permalink":"/scrut/docs/tutorial/output-expectations"}}');var s=n(74848),o=n(28453),a=n(24286);const l={},d="Basic Expectations",c={},r=[...a.RM,{value:"Ignore Command Output",id:"ignore-command-output",level:2},{value:"Exit Code Validation by Default",id:"exit-code-validation-by-default",level:2},{value:"Expect a Non-Zero Exit Code",id:"expect-a-non-zero-exit-code",level:2}];function h(e){const t={admonition:"admonition",code:"code",em:"em",h1:"h1",h2:"h2",header:"header",li:"li",p:"p",pre:"pre",strong:"strong",ul:"ul",...(0,o.R)(),...e.components},{FbInternalOnly:n}=t;return n||function(e,t){throw new Error("Expected "+(t?"component":"object")+" `"+e+"` to be defined: you likely forgot to import, pass, or provide it.")}("FbInternalOnly",!0),(0,s.jsxs)(s.Fragment,{children:[(0,s.jsx)(t.header,{children:(0,s.jsx)(t.h1,{id:"basic-expectations",children:"Basic Expectations"})}),"\n",(0,s.jsx)(n,{children:(0,s.jsx)(a.Ay,{})}),"\n",(0,s.jsxs)(t.p,{children:["The smoke test from the previous chapter validates that executing ",(0,s.jsx)(t.code,{children:"jq --version"})," will output the string ",(0,s.jsx)(t.code,{children:"jq-1.7"}),". While this is a good start, it also has a few problems:"]}),"\n",(0,s.jsxs)(t.ul,{children:["\n",(0,s.jsx)(t.li,{children:'It is not really a smoke test, because it tests more than "does it blow up?"'}),"\n",(0,s.jsxs)(t.li,{children:["It will fail down the line when ",(0,s.jsx)(t.code,{children:"jq"})," is being updated (as ",(0,s.jsx)(t.code,{children:"jq"})," is only a stand-in for the CLI you are testing an developing, you will likely have constant version upgrades to deal with)"]}),"\n"]}),"\n",(0,s.jsx)(t.p,{children:'To make this a proper smoke test it needs to shed the validation of the specific version and only validate that the test execution does not "blow up".'}),"\n",(0,s.jsx)(t.h2,{id:"ignore-command-output",children:"Ignore Command Output"}),"\n",(0,s.jsxs)(t.p,{children:["Consider how you would get rid of the output when executing the command ",(0,s.jsx)(t.code,{children:"jq --version"})," on the shell. You would likely do something this:"]}),"\n",(0,s.jsx)(t.pre,{children:(0,s.jsx)(t.code,{className:"language-bash",metastring:'title="Terminal"',children:"$ jq --version > /dev/null\n"})}),"\n",(0,s.jsx)(t.admonition,{type:"info",children:(0,s.jsxs)(t.p,{children:["The suffix ",(0,s.jsx)(t.code,{children:"> /dev/null"})," redirects the output that ",(0,s.jsx)(t.code,{children:"jq --version"})," writes to ",(0,s.jsx)(t.strong,{children:"STDOUT"})," to ",(0,s.jsx)(t.code,{children:"/dev/null"}),", resulting in nothing being written to STDOUT. As STDERR is not considered by default this is sufficient."]})}),"\n",(0,s.jsx)(t.p,{children:"And this is exactly what needs to be changed in the test document:"}),"\n",(0,s.jsx)(t.pre,{children:(0,s.jsx)(t.code,{className:"language-markdown",metastring:'title="tests/test.md" {4}',children:"# Command executes successfully\n\n```scrut\n$ jq --version > /dev/null\n```\n"})}),"\n",(0,s.jsxs)(t.p,{children:["With the ",(0,s.jsx)(t.em,{children:"output expectation"})," removed this test will do as a smoke test."]}),"\n",(0,s.jsx)(t.h2,{id:"exit-code-validation-by-default",children:"Exit Code Validation by Default"}),"\n",(0,s.jsxs)(t.p,{children:["Scrut automatically validates that the exit code of the execution of the ",(0,s.jsx)(t.em,{children:"shell expression"})," is ",(0,s.jsx)(t.code,{children:"0"})," which signifies that the execution ended without any failure. If it is not ",(0,s.jsx)(t.code,{children:"0"}),", then the execution is considered a failure and the validation of the ",(0,s.jsx)(t.em,{children:"test case"})," will fail."]}),"\n",(0,s.jsx)(t.p,{children:'That means: We are already testing if it does "blow up", as Scrut would fail the test if the execution blows up and ends in a non-zero exit code.'}),"\n",(0,s.jsx)(t.p,{children:"To make this clear, consider the following document of a test that will fail:"}),"\n",(0,s.jsx)(t.pre,{children:(0,s.jsx)(t.code,{className:"language-markdown",metastring:'title="tests/fail.md"',children:"# Test will fail\n\n```scrut\n$ false\n```\n"})}),"\n",(0,s.jsx)(t.admonition,{type:"info",children:(0,s.jsxs)(t.p,{children:["The ",(0,s.jsx)(t.code,{children:"false"})," command always fails and exits with a the exit code ",(0,s.jsx)(t.code,{children:"1"}),"."]})}),"\n",(0,s.jsx)(t.p,{children:"And here is how Scrut would tell you about the failure:"}),"\n",(0,s.jsx)(t.pre,{children:(0,s.jsx)(t.code,{className:"language-bash",metastring:'title="Terminal"',children:"$ scrut test tests/fail.md\n\ud83d\udd0e Found 1 test document(s)\n\u274c tests/fail.md: failed 1 out of 1 testcase\n\n// =============================================================================\n// @ tests/fail.md:4\n// -----------------------------------------------------------------------------\n// # Test will fail\n// -----------------------------------------------------------------------------\n// $ false\n// =============================================================================\n\nunexpected exit code\n  expected: 0\n  actual:   1\n\n## STDOUT\n## STDERR\n\n\nResult: 1 document(s) with 1 testcase(s): 0 succeeded, 1 failed and 0 skipped\n"})}),"\n",(0,s.jsx)(t.h2,{id:"expect-a-non-zero-exit-code",children:"Expect a Non-Zero Exit Code"}),"\n",(0,s.jsxs)(t.p,{children:["If the ",(0,s.jsx)(t.em,{children:"shell expression"})," that is being tested is actually expected to return a non-zero exit code, then the ",(0,s.jsx)(t.code,{children:"[<exit-code>]"})," expectation can be used to communicate this to Scrut. Here an example:"]}),"\n",(0,s.jsx)(t.pre,{children:(0,s.jsx)(t.code,{className:"language-markdown",metastring:'title="fail.md"',children:"# Test will fail\n\n```scrut\n$ false\n[1]\n```\n"})}),"\n",(0,s.jsxs)(t.p,{children:["The ",(0,s.jsx)(t.code,{children:"[1]"})," signifies that the test validation should expect an exit code of ",(0,s.jsx)(t.code,{children:"1"}),". Now the above document is valid again:"]}),"\n",(0,s.jsx)(t.pre,{children:(0,s.jsx)(t.code,{className:"language-bash",metastring:'title="Terminal"',children:"$ scrut test tests/fail.md\n\ud83d\udd0e Found 1 test document(s)\n\nResult: 1 document(s) with 1 testcase(s): 1 succeeded, 0 failed and 0 skipped\n"})}),"\n",(0,s.jsxs)(t.p,{children:["If any different number than ",(0,s.jsx)(t.code,{children:"1"})," would have been set then the validation would fail."]}),"\n",(0,s.jsx)(t.admonition,{type:"note",children:(0,s.jsxs)(t.p,{children:["Scrut automatically assumes ",(0,s.jsx)(t.code,{children:"0"})," exit code by default. Specifying it with ",(0,s.jsx)(t.code,{children:"[0]"})," is not needed (but also not invalid)."]})})]})}function u(e={}){const{wrapper:t}={...(0,o.R)(),...e.components};return t?(0,s.jsx)(t,{...e,children:(0,s.jsx)(h,{...e})}):h(e)}},28453:(e,t,n)=>{n.d(t,{R:()=>a,x:()=>l});var i=n(96540);const s={},o=i.createContext(s);function a(e){const t=i.useContext(o);return i.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function l(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(s):e.components||s:a(e.components),i.createElement(o.Provider,{value:t},e.children)}}}]);