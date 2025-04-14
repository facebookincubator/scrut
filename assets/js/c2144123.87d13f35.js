"use strict";(self.webpackChunkstaticdocs_starter=self.webpackChunkstaticdocs_starter||[]).push([[8503],{975:(e,t,n)=>{n.d(t,{Ay:()=>c,RM:()=>i});var s=n(74848),r=n(28453);const i=[];function o(e){return(0,s.jsx)(s.Fragment,{})}function c(e={}){const{wrapper:t}={...(0,r.R)(),...e.components};return t?(0,s.jsx)(t,{...e,children:(0,s.jsx)(o,{...e})}):o()}},28453:(e,t,n)=>{n.d(t,{R:()=>o,x:()=>c});var s=n(96540);const r={},i=s.createContext(r);function o(e){const t=s.useContext(i);return s.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function c(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(r):e.components||r:o(e.components),s.createElement(i.Provider,{value:t},e.children)}},91536:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>d,contentTitle:()=>a,default:()=>h,frontMatter:()=>c,metadata:()=>s,toc:()=>l});const s=JSON.parse('{"id":"integration/docker","title":"Scrut in Docker Container","description":"Scrut can be run in a Docker container. This is useful when integrating into CI/CD or if no local Rust development environment is available.","source":"@site/docs/integration/docker.md","sourceDirName":"integration","slug":"/integration/docker","permalink":"/scrut/docs/integration/docker","draft":false,"unlisted":false,"editUrl":"https://www.internalfb.com/code/fbsource/fbcode/clifoundation/scrut/website/docs/integration/docker.md","tags":[],"version":"current","frontMatter":{},"sidebar":"tutorialSidebar","previous":{"title":"Next Up","permalink":"/scrut/docs/tutorial/next-up"},"next":{"title":"Dotslash and Version Pinning","permalink":"/scrut/docs/integration/dotslash"}}');var r=n(74848),i=n(28453),o=n(975);const c={},a="Scrut in Docker Container",d={},l=[...o.RM,{value:"Get Scrut Docker Image",id:"get-scrut-docker-image",level:2},{value:"Pre-Built Image from GHCR",id:"pre-built-image-from-ghcr",level:3},{value:"Build Locally",id:"build-locally",level:3},{value:"Run Scrut in Docker Container",id:"run-scrut-in-docker-container",level:2}];function u(e){const t={a:"a",admonition:"admonition",code:"code",h1:"h1",h2:"h2",h3:"h3",header:"header",p:"p",pre:"pre",...(0,i.R)(),...e.components},{FbInternalOnly:n}=t;return n||function(e,t){throw new Error("Expected "+(t?"component":"object")+" `"+e+"` to be defined: you likely forgot to import, pass, or provide it.")}("FbInternalOnly",!0),(0,r.jsxs)(r.Fragment,{children:[(0,r.jsx)(t.header,{children:(0,r.jsx)(t.h1,{id:"scrut-in-docker-container",children:"Scrut in Docker Container"})}),"\n",(0,r.jsx)(n,{children:(0,r.jsx)(o.Ay,{})}),"\n",(0,r.jsx)(t.p,{children:"Scrut can be run in a Docker container. This is useful when integrating into CI/CD or if no local Rust development environment is available."}),"\n",(0,r.jsx)(t.h2,{id:"get-scrut-docker-image",children:"Get Scrut Docker Image"}),"\n",(0,r.jsx)(t.p,{children:"There are two ways:"}),"\n",(0,r.jsx)(t.h3,{id:"pre-built-image-from-ghcr",children:"Pre-Built Image from GHCR"}),"\n",(0,r.jsxs)(t.p,{children:["Here is how you can ",(0,r.jsx)(t.a,{href:"https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry",children:"work with theGitHub Container Registry"}),"."]}),"\n",(0,r.jsx)(t.p,{children:"The image is then available as:"}),"\n",(0,r.jsx)(t.pre,{children:(0,r.jsx)(t.code,{children:"ghcr.io/facebookexternal/scrut:<VERSION>\n"})}),"\n",(0,r.jsx)(t.h3,{id:"build-locally",children:"Build Locally"}),"\n",(0,r.jsxs)(t.p,{children:["Check out the ",(0,r.jsx)(t.a,{href:"https://github.com/facebookincubator/scrut",children:"Scrut git repository on GitHub"})," locally. It comes with a ",(0,r.jsx)(t.code,{children:"Dockerfile"})," in the root directory."]}),"\n",(0,r.jsx)(t.p,{children:"Now build the image:"}),"\n",(0,r.jsx)(t.pre,{children:(0,r.jsx)(t.code,{className:"language-bash",children:"$ docker build -t scrut:latest .\n"})}),"\n",(0,r.jsx)(t.admonition,{type:"note",children:(0,r.jsxs)(t.p,{children:["The build requires ",(0,r.jsx)(t.a,{href:"https://docs.docker.com/build/buildkit/",children:"Docker BuildKit"}),"."]})}),"\n",(0,r.jsxs)(t.admonition,{type:"tip",children:[(0,r.jsx)(t.p,{children:"The container build automatically runs both unit and integrating tests. This makes it a good, isolated development environment if you are interested in contributing to Scrut."}),(0,r.jsxs)(t.p,{children:["If you want to skip the tests, resulting in a faster build, you can set ",(0,r.jsx)(t.code,{children:"--build-arg SKIP_TESTS=yes"})," when executing ",(0,r.jsx)(t.code,{children:"docker build"}),"."]})]}),"\n",(0,r.jsx)(t.h2,{id:"run-scrut-in-docker-container",children:"Run Scrut in Docker Container"}),"\n",(0,r.jsxs)(t.p,{children:["Once you have the image available make sure to mount the directory containing the test suite as a volume into the container under ",(0,r.jsx)(t.code,{children:"/app"}),"."]}),"\n",(0,r.jsx)(t.p,{children:"Following an example with a small Rust CLI:"}),"\n",(0,r.jsx)(t.pre,{children:(0,r.jsx)(t.code,{className:"language-bash",children:"$ cd my-cli\n$ tree\n.\n\u251c\u2500\u2500 Cargo.toml\n\u251c\u2500\u2500 dist\n\u2502\xa0\xa0 \u2514\u2500\u2500 my-cli\n\u251c\u2500\u2500 src\n\u2502\xa0\xa0 \u251c\u2500\u2500 command_something_else.rs\n\u2502\xa0\xa0 \u251c\u2500\u2500 command_user_list.rs\n\u2502\xa0\xa0 \u251c\u2500\u2500 command_user_login.rs\n\u2502\xa0\xa0 \u2514\u2500\u2500 main.rs\n\u2514\u2500\u2500 tests\n    \u251c\u2500\u2500 smoke.md\n    \u251c\u2500\u2500 something-else.md\n    \u251c\u2500\u2500 user-listing.md\n    \u2514\u2500\u2500 user-login.md\n"})}),"\n",(0,r.jsx)(t.p,{children:"Now you would run Scrut like this:"}),"\n",(0,r.jsx)(t.pre,{children:(0,r.jsx)(t.code,{className:"language-bash",metastring:'title="Terminal"',children:"$ docker run --rm -ti -v $(pwd):/app scrut:latest test --verbose tests/\n\ud83d\udd0e Found 4 test document(s)\n\u2705 tests/user-login.md: passed 3 testcases\n\u2705 tests/smoke.md: passed 5 testcases\n\u2705 tests/user-listing.md: passed 1 testcase\n\u2705 tests/something-else.md: passed 13 testcases\n\nResult: 4 document(s) with 22 testcase(s): 22 succeeded, 0 failed and 0 skipped\n"})}),"\n",(0,r.jsxs)(t.admonition,{type:"tip",children:[(0,r.jsxs)(t.p,{children:["Running tests inside a container can change the path location of the CLI binary. Consider using the ",(0,r.jsx)(t.code,{children:"--prepend-test-file-paths"})," parameter to inject a ",(0,r.jsx)(t.a,{href:"/docs/reference/fundamentals/test-document/",children:"test document"})," that extends the ",(0,r.jsx)(t.code,{children:"PATH"})," environment variable as needed. Here an example:"]}),(0,r.jsx)(t.pre,{children:(0,r.jsx)(t.code,{className:"language-markdown",metastring:'title="docker-prepend.md"',children:'# Add `/app/dist` to `PATH`\n\n```scrut\n$ export PATH="/app/dist:$PATH"\n```\n'})}),(0,r.jsxs)(t.p,{children:["And then all calls to ",(0,r.jsx)(t.code,{children:"my-cli"})," in the ",(0,r.jsx)(t.a,{href:"/docs/reference/fundamentals/test-document/",children:"test documents"})," will be resolved to ",(0,r.jsx)(t.code,{children:"/app/dist/my-cli"}),":"]}),(0,r.jsx)(t.pre,{children:(0,r.jsx)(t.code,{className:"language-bash",metastring:'title="Terminal"',children:"$ docker run --rm -ti -v $(pwd):/app scrut:latest \\\n    test --verbose --prepend-test-file-paths=./docker-prepend.md tests/\n\ud83d\udd0e Found 4 test document(s)\n\u2705 tests/user-login.md: passed 4 testcases\n\u2705 tests/smoke.md: passed 6 testcases\n\u2705 tests/user-listing.md: passed 2 testcase\n\u2705 tests/something-else.md: passed 14 testcases\n\nResult: 4 document(s) with 26 testcase(s): 26 succeeded, 0 failed and 0 skipped\n"})})]})]})}function h(e={}){const{wrapper:t}={...(0,i.R)(),...e.components};return t?(0,r.jsx)(t,{...e,children:(0,r.jsx)(u,{...e})}):u(e)}}}]);