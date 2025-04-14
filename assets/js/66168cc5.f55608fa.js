"use strict";(self.webpackChunkstaticdocs_starter=self.webpackChunkstaticdocs_starter||[]).push([[421],{24286:(e,t,s)=>{s.d(t,{Ay:()=>c,RM:()=>i});var n=s(74848),r=s(28453);const i=[];function o(e){return(0,n.jsx)(n.Fragment,{})}function c(e={}){const{wrapper:t}={...(0,r.R)(),...e.components};return t?(0,n.jsx)(t,{...e,children:(0,n.jsx)(o,{...e})}):o()}},28453:(e,t,s)=>{s.d(t,{R:()=>o,x:()=>c});var n=s(96540);const r={},i=n.createContext(r);function o(e){const t=n.useContext(i);return n.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function c(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(r):e.components||r:o(e.components),n.createElement(i.Provider,{value:t},e.children)}},72870:(e,t,s)=>{s.r(t),s.d(t,{assets:()=>a,contentTitle:()=>l,default:()=>h,frontMatter:()=>c,metadata:()=>n,toc:()=>d});const n=JSON.parse('{"id":"tutorial/create-test","title":"Test Creation","description":"As previously decided, the first test will validate that jq --version executes successfully. Running this command should produce output similar to the following (your version may vary):","source":"@site/docs/tutorial/02-create-test.md","sourceDirName":"tutorial","slug":"/tutorial/create-test","permalink":"/scrut/docs/tutorial/create-test","draft":false,"unlisted":false,"editUrl":"https://www.internalfb.com/code/fbsource/fbcode/clifoundation/scrut/website/docs/tutorial/02-create-test.md","tags":[],"version":"current","sidebarPosition":2,"frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"What to test?","permalink":"/scrut/docs/tutorial/what-to-test"},"next":{"title":"Basic Expectations","permalink":"/scrut/docs/tutorial/basic-expectations"}}');var r=s(74848),i=s(28453),o=s(24286);const c={},l="Test Creation",a={},d=[...o.RM,{value:"Using Scrut&#39;s Built-in Test Creation",id:"using-scruts-built-in-test-creation",level:2},{value:"Use STDIN to receive commands",id:"use-stdin-to-receive-commands",level:3},{value:"Write tests manually",id:"write-tests-manually",level:2}];function u(e){const t={admonition:"admonition",code:"code",h1:"h1",h2:"h2",h3:"h3",header:"header",li:"li",p:"p",pre:"pre",ul:"ul",...(0,i.R)(),...e.components},{FbInternalOnly:s}=t;return s||function(e,t){throw new Error("Expected "+(t?"component":"object")+" `"+e+"` to be defined: you likely forgot to import, pass, or provide it.")}("FbInternalOnly",!0),(0,r.jsxs)(r.Fragment,{children:[(0,r.jsx)(t.header,{children:(0,r.jsx)(t.h1,{id:"test-creation",children:"Test Creation"})}),"\n",(0,r.jsx)(s,{children:(0,r.jsx)(o.Ay,{})}),"\n",(0,r.jsxs)(t.p,{children:["As previously decided, the first test will validate that ",(0,r.jsx)(t.code,{children:"jq --version"})," executes successfully. Running this command should produce output similar to the following (your version may vary):"]}),"\n",(0,r.jsx)(t.pre,{children:(0,r.jsx)(t.code,{className:"language-bash",metastring:'title="Terminal"',children:"$ jq --version\njq-1.7.1\n"})}),"\n",(0,r.jsx)(t.h2,{id:"using-scruts-built-in-test-creation",children:"Using Scrut's Built-in Test Creation"}),"\n",(0,r.jsx)(t.p,{children:"Generating a Scrut test from the command line is pretty straight forward:"}),"\n",(0,r.jsx)(t.pre,{children:(0,r.jsx)(t.code,{className:"language-bash",metastring:'title="Terminal"',children:"$ scrut create --output tests/smoke.md -- jq --version\n\u270d\ufe0f /tmp/smoke.md: Writing generated test document\n"})}),"\n",(0,r.jsx)(t.p,{children:"This will create a test file that should look like this:"}),"\n",(0,r.jsx)(t.pre,{children:(0,r.jsx)(t.code,{className:"language-markdown",metastring:'title="tests/smoke.md"',children:"# Command executes successfully\n\n```scrut\n$ jq --version\njq-1.7.1\n```\n"})}),"\n",(0,r.jsx)(t.p,{children:"You can now execute the newly created test file with:"}),"\n",(0,r.jsx)(t.pre,{children:(0,r.jsx)(t.code,{className:"language-bash",metastring:'title="Terminal"',children:"$ scrut test tests/smoke.md\n\ud83d\udd0e Found 1 test document(s)\n\nResult: 1 document(s) with 1 testcase(s): 1 succeeded, 0 failed and 0 skipped\n"})}),"\n",(0,r.jsxs)(t.admonition,{type:"note",children:[(0,r.jsxs)(t.p,{children:["The ",(0,r.jsx)(t.code,{children:"scrut test"})," command accepts arbitrary files or directories. All of the following (assuming the paths exist) are valid:"]}),(0,r.jsxs)(t.ul,{children:["\n",(0,r.jsxs)(t.li,{children:[(0,r.jsx)(t.code,{children:"scrut test tests"})," - test every test file found (recursively) in ",(0,r.jsx)(t.code,{children:"tests"})]}),"\n",(0,r.jsxs)(t.li,{children:[(0,r.jsx)(t.code,{children:"scrut test tests/smoke.md tests/other.md"})," - test both files ",(0,r.jsx)(t.code,{children:"tests/smoke.md"})," and ",(0,r.jsx)(t.code,{children:"tests/other.md"})]}),"\n",(0,r.jsxs)(t.li,{children:[(0,r.jsx)(t.code,{children:"scrut test tests other-tests"})," - test all files found (recursively) in the ",(0,r.jsx)(t.code,{children:"tests"})," and ",(0,r.jsx)(t.code,{children:"other-tests"})," directories"]}),"\n"]})]}),"\n",(0,r.jsx)(t.h3,{id:"use-stdin-to-receive-commands",children:"Use STDIN to receive commands"}),"\n",(0,r.jsx)(t.p,{children:"Alternatively you can also pipe the command via STDIN to scrut create:"}),"\n",(0,r.jsx)(t.pre,{children:(0,r.jsx)(t.code,{className:"language-bash",metastring:'title="Terminal"',children:'$ echo "jq --version" | scrut create - > tests/smoke.md\n\u270d\ufe0f STDOUT: Writing generated test document\n'})}),"\n",(0,r.jsxs)(t.p,{children:["Here also ",(0,r.jsx)(t.code,{children:"--output"})," was omitted, in which case ",(0,r.jsx)(t.code,{children:"scrut create"})," will print the newly created test file to STDOUT. Check out ",(0,r.jsx)(t.code,{children:"scrut create --help"})," to see all options."]}),"\n",(0,r.jsx)(t.h2,{id:"write-tests-manually",children:"Write tests manually"}),"\n",(0,r.jsxs)(t.p,{children:["You can of course also create your ",(0,r.jsx)(t.code,{children:"tests/smoke.md"})," file manually in a text editor. As Scrut test documents are written in Markdown any Markdown syntax highlighting plugin for your IDE of choice will help greatly."]})]})}function h(e={}){const{wrapper:t}={...(0,i.R)(),...e.components};return t?(0,r.jsx)(t,{...e,children:(0,r.jsx)(u,{...e})}):u(e)}}}]);