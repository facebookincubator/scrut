(self.webpackChunkstaticdocs_starter=self.webpackChunkstaticdocs_starter||[]).push([[9350],{18426:(e,t)=>{function n(e){let t,n=[];for(let r of e.split(",").map((e=>e.trim())))if(/^-?\d+$/.test(r))n.push(parseInt(r,10));else if(t=r.match(/^(-?\d+)(-|\.\.\.?|\u2025|\u2026|\u22EF)(-?\d+)$/)){let[e,r,o,a]=t;if(r&&a){r=parseInt(r),a=parseInt(a);const e=r<a?1:-1;"-"!==o&&".."!==o&&"\u2025"!==o||(a+=e);for(let t=r;t!==a;t+=e)n.push(t)}}return n}t.default=n,e.exports=n},25688:(e,t)=>{"use strict";Object.defineProperty(t,"__esModule",{value:!0});t.default="data:image/png;base64,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"},28785:function(e,t,n){"use strict";var r=this&&this.__importDefault||function(e){return e&&e.__esModule?e:{default:e}};Object.defineProperty(t,"__esModule",{value:!0}),t.getAssetUrl=t.writeImagesToDisk=t.storeImage=void 0;const o=n(66590),a=r(n(57975)),s="rendered-components",c=[];function l(e,t,n){return`${e}.${t}.${n}.png`}t.storeImage=function(e,t,n,r){const o=Buffer.from(r,"base64");c.push({filename:l(e,t,n),content:o})},t.writeImagesToDisk=function(){return o.promises.mkdir(a.default.join("build",s)).then((()=>Promise.all(c.map((e=>o.promises.writeFile(a.default.join("build",s,e.filename),e.content)))).then((()=>{}))))},t.getAssetUrl=function(e,t,n){return`/${s}/`+l(e,t,n)}},31177:(e,t,n)=>{"use strict";n.d(t,{A:()=>a});var r=n(8532),o=n(53115);function a(){const{prism:e}=(0,o.p)(),{colorMode:t}=(0,r.G)(),n=e.theme,a=e.darkTheme||n;return"dark"===t?a:n}},33231:(e,t,n)=>{"use strict";n.r(t),n.d(t,{default:()=>z});var r=n(96540),o=n(9136),a=n(34164),s=n(31177),c=n(204),l=n(18426),i=n.n(l);const d=/title=(?<quote>["'])(?<title>.*?)\1/,u=/\{(?<range>[\d,-]+)\}/,f={js:{start:"\\/\\/",end:""},jsBlock:{start:"\\/\\*",end:"\\*\\/"},jsx:{start:"\\{\\s*\\/\\*",end:"\\*\\/\\s*\\}"},bash:{start:"#",end:""},html:{start:"\x3c!--",end:"--\x3e"}},m={...f,lua:{start:"--",end:""},wasm:{start:"\\;\\;",end:""},tex:{start:"%",end:""},vb:{start:"['\u2018\u2019]",end:""},vbnet:{start:"(?:_\\s*)?['\u2018\u2019]",end:""},rem:{start:"[Rr][Ee][Mm]\\b",end:""},f90:{start:"!",end:""},ml:{start:"\\(\\*",end:"\\*\\)"},cobol:{start:"\\*>",end:""}},A=Object.keys(f);function g(e,t){const n=e.map((e=>{const{start:n,end:r}=m[e];return`(?:${n}\\s*(${t.flatMap((e=>[e.line,e.block?.start,e.block?.end].filter(Boolean))).join("|")})\\s*${r})`})).join("|");return new RegExp(`^\\s*(?:${n})\\s*$`)}function p(e,t){let n=e.replace(/\n$/,"");const{language:r,magicComments:o,metastring:a}=t;if(a&&u.test(a)){const e=a.match(u).groups.range;if(0===o.length)throw new Error(`A highlight range has been given in code block's metastring (\`\`\` ${a}), but no magic comment config is available. Docusaurus applies the first magic comment entry's className for metastring ranges.`);const t=o[0].className,r=i()(e).filter((e=>e>0)).map((e=>[e-1,[t]]));return{lineClassNames:Object.fromEntries(r),code:n}}if(void 0===r)return{lineClassNames:{},code:n};const s=function(e,t){switch(e){case"js":case"javascript":case"ts":case"typescript":return g(["js","jsBlock"],t);case"jsx":case"tsx":return g(["js","jsBlock","jsx"],t);case"html":return g(["js","jsBlock","html"],t);case"python":case"py":case"bash":return g(["bash"],t);case"markdown":case"md":return g(["html","jsx","bash"],t);case"tex":case"latex":case"matlab":return g(["tex"],t);case"lua":case"haskell":return g(["lua"],t);case"sql":return g(["lua","jsBlock"],t);case"wasm":return g(["wasm"],t);case"vb":case"vba":case"visual-basic":return g(["vb","rem"],t);case"vbnet":return g(["vbnet","rem"],t);case"batch":return g(["rem"],t);case"basic":return g(["rem","f90"],t);case"fsharp":return g(["js","ml"],t);case"ocaml":case"sml":return g(["ml"],t);case"fortran":return g(["f90"],t);case"cobol":return g(["cobol"],t);default:return g(A,t)}}(r,o),c=n.split("\n"),l=Object.fromEntries(o.map((e=>[e.className,{start:0,range:""}]))),d=Object.fromEntries(o.filter((e=>e.line)).map((e=>{let{className:t,line:n}=e;return[n,t]}))),f=Object.fromEntries(o.filter((e=>e.block)).map((e=>{let{className:t,block:n}=e;return[n.start,t]}))),m=Object.fromEntries(o.filter((e=>e.block)).map((e=>{let{className:t,block:n}=e;return[n.end,t]})));for(let i=0;i<c.length;){const e=c[i].match(s);if(!e){i+=1;continue}const t=e.slice(1).find((e=>void 0!==e));d[t]?l[d[t]].range+=`${i},`:f[t]?l[f[t]].start=i:m[t]&&(l[m[t]].range+=`${l[m[t]].start}-${i-1},`),c.splice(i,1)}n=c.join("\n");const p={};return Object.entries(l).forEach((e=>{let[t,{range:n}]=e;i()(n).forEach((e=>{p[e]??=[],p[e].push(t)}))})),{lineClassNames:p,code:n}}const h={codeBlockContainer:"codeBlockContainer_Ckt0"};var b=n(74848);function v(e){let{as:t,...n}=e;const r=function(e){const t={color:"--prism-color",backgroundColor:"--prism-background-color"},n={};return Object.entries(e.plain).forEach((e=>{let[r,o]=e;const a=t[r];a&&"string"==typeof o&&(n[a]=o)})),n}((0,s.A)());return(0,b.jsx)(t,{...n,style:r,className:(0,a.A)(n.className,h.codeBlockContainer,c.G.common.codeBlock)})}const w={codeBlockContent:"codeBlockContent_biex",codeBlockTitle:"codeBlockTitle_Ktv7",codeBlock:"codeBlock_bY9V",codeBlockStandalone:"codeBlockStandalone_MEMb",codeBlockLines:"codeBlockLines_e6Vv",codeBlockLinesWithNumbering:"codeBlockLinesWithNumbering_o6Pm",buttonGroup:"buttonGroup__atx"};function C(e){let{children:t,className:n}=e;return(0,b.jsx)(v,{as:"pre",tabIndex:0,className:(0,a.A)(w.codeBlockStandalone,"thin-scrollbar",n),children:(0,b.jsx)("code",{className:w.codeBlockLines,children:t})})}var k=n(53115),B=n(26849);const j={attributes:!0,characterData:!0,childList:!0,subtree:!0};function y(e,t){const[n,o]=(0,r.useState)(),a=(0,r.useCallback)((()=>{o(e.current?.closest("[role=tabpanel][hidden]"))}),[e,o]);(0,r.useEffect)((()=>{a()}),[a]),function(e,t,n){void 0===n&&(n=j);const o=(0,B._q)(t),a=(0,B.Be)(n);(0,r.useEffect)((()=>{const t=new MutationObserver(o);return e&&t.observe(e,a),()=>t.disconnect()}),[e,o,a])}(n,(e=>{e.forEach((e=>{"attributes"===e.type&&"hidden"===e.attributeName&&(t(),a())}))}),{attributes:!0,characterData:!1,childList:!1,subtree:!1})}var E=n(71765);const N={codeLine:"codeLine_lJS_",codeLineNumber:"codeLineNumber_Tfdd",codeLineContent:"codeLineContent_feaV"};function x(e){let{line:t,classNames:n,showLineNumbers:r,getLineProps:o,getTokenProps:s}=e;1===t.length&&"\n"===t[0].content&&(t[0].content="");const c=o({line:t,className:(0,a.A)(n,r&&N.codeLine)}),l=t.map(((e,t)=>(0,b.jsx)("span",{...s({token:e})},t)));return(0,b.jsxs)("span",{...c,children:[r?(0,b.jsxs)(b.Fragment,{children:[(0,b.jsx)("span",{className:N.codeLineNumber}),(0,b.jsx)("span",{className:N.codeLineContent,children:l})]}):l,(0,b.jsx)("br",{})]})}var L=n(50539);function I(e){return(0,b.jsx)("svg",{viewBox:"0 0 24 24",...e,children:(0,b.jsx)("path",{fill:"currentColor",d:"M19,21H8V7H19M19,5H8A2,2 0 0,0 6,7V21A2,2 0 0,0 8,23H19A2,2 0 0,0 21,21V7A2,2 0 0,0 19,5M16,1H4A2,2 0 0,0 2,3V17H4V3H16V1Z"})})}function M(e){return(0,b.jsx)("svg",{viewBox:"0 0 24 24",...e,children:(0,b.jsx)("path",{fill:"currentColor",d:"M21,7L9,19L3.5,13.5L4.91,12.09L9,16.17L19.59,5.59L21,7Z"})})}const P={copyButtonCopied:"copyButtonCopied_obH4",copyButtonIcons:"copyButtonIcons_eSgA",copyButtonIcon:"copyButtonIcon_y97N",copyButtonSuccessIcon:"copyButtonSuccessIcon_LjdS"};function W(e){let{code:t,className:n}=e;const[o,s]=(0,r.useState)(!1),c=(0,r.useRef)(void 0),l=(0,r.useCallback)((()=>{!function(e,t){let{target:n=document.body}=void 0===t?{}:t;if("string"!=typeof e)throw new TypeError(`Expected parameter \`text\` to be a \`string\`, got \`${typeof e}\`.`);const r=document.createElement("textarea"),o=document.activeElement;r.value=e,r.setAttribute("readonly",""),r.style.contain="strict",r.style.position="absolute",r.style.left="-9999px",r.style.fontSize="12pt";const a=document.getSelection(),s=a.rangeCount>0&&a.getRangeAt(0);n.append(r),r.select(),r.selectionStart=0,r.selectionEnd=e.length;let c=!1;try{c=document.execCommand("copy")}catch{}r.remove(),s&&(a.removeAllRanges(),a.addRange(s)),o&&o.focus()}(t),s(!0),c.current=window.setTimeout((()=>{s(!1)}),1e3)}),[t]);return(0,r.useEffect)((()=>()=>window.clearTimeout(c.current)),[]),(0,b.jsx)("button",{type:"button","aria-label":o?(0,L.translate)({id:"theme.CodeBlock.copied",message:"Copied",description:"The copied button label on code blocks"}):(0,L.translate)({id:"theme.CodeBlock.copyButtonAriaLabel",message:"Copy code to clipboard",description:"The ARIA label for copy code blocks button"}),title:(0,L.translate)({id:"theme.CodeBlock.copy",message:"Copy",description:"The copy button label on code blocks"}),className:(0,a.A)("clean-btn",n,P.copyButton,o&&P.copyButtonCopied),onClick:l,children:(0,b.jsxs)("span",{className:P.copyButtonIcons,"aria-hidden":"true",children:[(0,b.jsx)(I,{className:P.copyButtonIcon}),(0,b.jsx)(M,{className:P.copyButtonSuccessIcon})]})})}function S(e){return(0,b.jsx)("svg",{viewBox:"0 0 24 24",...e,children:(0,b.jsx)("path",{fill:"currentColor",d:"M4 19h6v-2H4v2zM20 5H4v2h16V5zm-3 6H4v2h13.25c1.1 0 2 .9 2 2s-.9 2-2 2H15v-2l-3 3l3 3v-2h2c2.21 0 4-1.79 4-4s-1.79-4-4-4z"})})}const T={wordWrapButtonIcon:"wordWrapButtonIcon_Bwma",wordWrapButtonEnabled:"wordWrapButtonEnabled_EoeP"};function U(e){let{className:t,onClick:n,isEnabled:r}=e;const o=(0,L.translate)({id:"theme.CodeBlock.wordWrapToggle",message:"Toggle word wrap",description:"The title attribute for toggle word wrapping button of code block lines"});return(0,b.jsx)("button",{type:"button",onClick:n,className:(0,a.A)("clean-btn",t,r&&T.wordWrapButtonEnabled),"aria-label":o,title:o,children:(0,b.jsx)(S,{className:T.wordWrapButtonIcon,"aria-hidden":"true"})})}function O(e){let{children:t,className:n="",metastring:o,title:c,showLineNumbers:l,language:i}=e;const{prism:{defaultLanguage:u,magicComments:f}}=(0,k.p)(),m=function(e){return e?.toLowerCase()}(i??function(e){const t=e.split(" ").find((e=>e.startsWith("language-")));return t?.replace(/language-/,"")}(n)??u),A=(0,s.A)(),g=function(){const[e,t]=(0,r.useState)(!1),[n,o]=(0,r.useState)(!1),a=(0,r.useRef)(null),s=(0,r.useCallback)((()=>{const n=a.current.querySelector("code");e?n.removeAttribute("style"):(n.style.whiteSpace="pre-wrap",n.style.overflowWrap="anywhere"),t((e=>!e))}),[a,e]),c=(0,r.useCallback)((()=>{const{scrollWidth:e,clientWidth:t}=a.current,n=e>t||a.current.querySelector("code").hasAttribute("style");o(n)}),[a]);return y(a,c),(0,r.useEffect)((()=>{c()}),[e,c]),(0,r.useEffect)((()=>(window.addEventListener("resize",c,{passive:!0}),()=>{window.removeEventListener("resize",c)})),[c]),{codeBlockRef:a,isEnabled:e,isCodeScrollable:n,toggle:s}}(),h=function(e){return e?.match(d)?.groups.title??""}(o)||c,{lineClassNames:C,code:B}=p(t,{metastring:o,language:m,magicComments:f}),j=l??function(e){return Boolean(e?.includes("showLineNumbers"))}(o);return(0,b.jsxs)(v,{as:"div",className:(0,a.A)(n,m&&!n.includes(`language-${m}`)&&`language-${m}`),children:[h&&(0,b.jsx)("div",{className:w.codeBlockTitle,children:h}),(0,b.jsxs)("div",{className:w.codeBlockContent,children:[(0,b.jsx)(E.f4,{theme:A,code:B,language:m??"text",children:e=>{let{className:t,style:n,tokens:r,getLineProps:o,getTokenProps:s}=e;return(0,b.jsx)("pre",{tabIndex:0,ref:g.codeBlockRef,className:(0,a.A)(t,w.codeBlock,"thin-scrollbar"),style:n,children:(0,b.jsx)("code",{className:(0,a.A)(w.codeBlockLines,j&&w.codeBlockLinesWithNumbering),children:r.map(((e,t)=>(0,b.jsx)(x,{line:e,getLineProps:o,getTokenProps:s,classNames:C[t],showLineNumbers:j},t)))})})}}),(0,b.jsxs)("div",{className:w.buttonGroup,children:[(g.isEnabled||g.isCodeScrollable)&&(0,b.jsx)(U,{className:w.codeButton,onClick:()=>g.toggle(),isEnabled:g.isEnabled}),(0,b.jsx)(W,{className:w.codeButton,code:B})]})]})]})}function z(e){let{children:t,...n}=e;const a=(0,o.default)(),s=function(e){return r.Children.toArray(e).some((e=>(0,r.isValidElement)(e)))?e:Array.isArray(e)?e.join(""):e}(t),c="string"==typeof s?O:C;return(0,b.jsx)(c,{...n,children:s},String(a))}},39350:function(e,t,n){"use strict";var r=this&&this.__createBinding||(Object.create?function(e,t,n,r){void 0===r&&(r=n);var o=Object.getOwnPropertyDescriptor(t,n);o&&!("get"in o?!t.__esModule:o.writable||o.configurable)||(o={enumerable:!0,get:function(){return t[n]}}),Object.defineProperty(e,r,o)}:function(e,t,n,r){void 0===r&&(r=n),e[r]=t[n]}),o=this&&this.__setModuleDefault||(Object.create?function(e,t){Object.defineProperty(e,"default",{enumerable:!0,value:t})}:function(e,t){e.default=t}),a=this&&this.__importStar||function(e){if(e&&e.__esModule)return e;var t={};if(null!=e)for(var n in e)"default"!==n&&Object.prototype.hasOwnProperty.call(e,n)&&r(t,e,n);return o(t,e),t},s=this&&this.__importDefault||function(e){return e&&e.__esModule?e:{default:e}};Object.defineProperty(t,"__esModule",{value:!0});const c=a(n(96540)),l=s(n(33231)),i=s(n(40797)),d=s(n(9136)),u=n(78191),f=s(n(59627)),m=s(n(25688)),A=s(n(77774)),g=n(72077),p=n(28785),h=n(29030),b=[{names:["fbsource","fbs"],project:"fbsource",canonicalName:"fbsource"},{names:["www"],project:"facebook-www",canonicalName:"www"}];t.default=e=>{const{siteConfig:t}=(0,i.default)(),n=(0,g.usePluginData)("internaldocs-fb").opts.maxCodeBlockHeight,r=(0,d.default)(),o=(0,c.useRef)(null),a=(0,c.useRef)(null),s=(0,c.useRef)(null),v=(0,c.useRef)(!1),[w,C]=(0,c.useState)(!1),k=(0,c.useCallback)((e=>{v.current||(window.requestAnimationFrame((()=>{a.current&&s.current&&(e.target.scrollTop>0?a.current.style.boxShadow="0 1em 1em -1em black inset":a.current.style.boxShadow="none",e.target.scrollTop===e.target.scrollHeight-e.target.offsetHeight?s.current.style.boxShadow="none":s.current.style.boxShadow="0 -1em 1em -1em black inset"),v.current=!1})),v.current=!0)}),[]);(0,c.useEffect)((()=>{o.current&&(o.current.addEventListener("scroll",k),window.requestAnimationFrame((()=>{k({target:o.current})})))}));const{withBaseUrl:B}=(0,h.useBaseUrlUtils)(),j=function(e){try{return(0,l.default)(e)}catch(t){return c.default.createElement("p",{style:{color:"red",fontWeight:"bold"}},"Could not render codeblock")}}(Object.assign({children:""},e));if(!r)return j;if("string"!=typeof e.file)return j;let y,E,N,x;if((0,u.isInternal)()){if(!t.customFields)return j;const{fbRepoName:n,ossRepoPath:r}=t.customFields;if("string"!=typeof n)return j;y="string"==typeof r&&"string"!=typeof e.repo?function(){for(var e=arguments.length,t=new Array(e),n=0;n<e;n++)t[n]=arguments[n];return t.map((e=>e.startsWith("/")?e.slice(1):e)).map((e=>e.endsWith("/")?e.slice(0,e.length-1):e)).join("/")}(r,e.file):e.file;const o=b.find((t=>{var r;return t.names.includes((null!==(r=e.repo)&&void 0!==r?r:n).toLowerCase())}));if(void 0===o)return j;E=function(e,t){const n=new URL("https://www.internalfb.com");return n.pathname=`/code/${e.canonicalName}/${t}`,n.toString()}(o,y),N=function(e,t){const n=new URL("https://www.internalfb.com/intern/nuclide/open/arc");return n.searchParams.append("project",e.project),n.searchParams.append("paths[0]",t),n.toString()}(o,y),x=function(e,t){if("fbsource"!==e.canonicalName||!t.startsWith("fbandroid"))return null;const n=new URL("fb-ide-opener://open");return n.searchParams.append("ide","intellij"),n.searchParams.append("filepath",`/fbsource/${t}`),n.toString()}(o,y)}else{if("string"!=typeof t.organizationName||"string"!=typeof t.projectName)return j;y=e.file,E=function(e,t,n){const r=new URL("https://github.com");return r.pathname=`/${e}/${t}/blob/master/${n}`,r.toString()}(t.organizationName,t.projectName,e.file),N=null,x=null}const L=y.split("/"),I=L[L.length-1];return c.default.createElement("div",{className:`${A.default.CodeBlockFrame} ${w?A.default.WithImage:""}`},e.title?null:c.default.createElement("div",{className:A.default.CodeBlockHeader},c.default.createElement("a",{href:E,title:"Browse entire file",target:"_blank",rel:"noreferrer",onClick:()=>u.feedback.reportFeatureUsage({featureName:"browse-file",id:y}),className:A.default.CodeBlockFilenameTab},I),null!==N?c.default.createElement("a",{target:"_blank",rel:"noreferrer",href:N,onClick:()=>u.feedback.reportFeatureUsage({featureName:"open-in-vscode",id:y})},c.default.createElement("img",{title:"Open in VSCode @ FB",src:f.default})):null,null!==x?c.default.createElement("a",{target:"_blank",rel:"noreferrer",href:x,onClick:()=>u.feedback.reportFeatureUsage({featureName:"open-in-android-studio",id:y})},c.default.createElement("img",{title:"Open in Android Studio",src:m.default})):null),c.default.createElement("div",{style:{position:"relative"}},c.default.createElement("div",{ref:o,style:{maxHeight:n,overflowY:"auto"}},c.default.createElement("div",{style:{display:"flex",flexDirection:"row",width:"100%"}},c.default.createElement("div",{className:A.default.CodeBlockCodeFrame},j),c.default.createElement("div",{className:A.default.CodeBlockPreviewFrame},e.repo&&"string"==typeof e.classname&&"string"==typeof e.symbol?c.default.createElement("img",{onLoad:()=>{C(!0)},src:B((0,p.getAssetUrl)(e.repo,e.classname,e.symbol))}):null))),void 0===n?null:[c.default.createElement("div",{key:"shadowtop",ref:a,style:{bottom:0,left:0,right:0,top:0,pointerEvents:"none",transition:"all .2s ease-out",boxShadow:"none",position:"absolute"}}),c.default.createElement("div",{key:"shadowbottom",ref:s,style:{bottom:0,left:0,right:0,top:0,pointerEvents:"none",transition:"all .2s ease-out",boxShadow:"none",position:"absolute"}})]))}},57975:e=>{"use strict";function t(e){if("string"!=typeof e)throw new TypeError("Path must be a string. Received "+JSON.stringify(e))}function n(e,t){for(var n,r="",o=0,a=-1,s=0,c=0;c<=e.length;++c){if(c<e.length)n=e.charCodeAt(c);else{if(47===n)break;n=47}if(47===n){if(a===c-1||1===s);else if(a!==c-1&&2===s){if(r.length<2||2!==o||46!==r.charCodeAt(r.length-1)||46!==r.charCodeAt(r.length-2))if(r.length>2){var l=r.lastIndexOf("/");if(l!==r.length-1){-1===l?(r="",o=0):o=(r=r.slice(0,l)).length-1-r.lastIndexOf("/"),a=c,s=0;continue}}else if(2===r.length||1===r.length){r="",o=0,a=c,s=0;continue}t&&(r.length>0?r+="/..":r="..",o=2)}else r.length>0?r+="/"+e.slice(a+1,c):r=e.slice(a+1,c),o=c-a-1;a=c,s=0}else 46===n&&-1!==s?++s:s=-1}return r}var r={resolve:function(){for(var e,r="",o=!1,a=arguments.length-1;a>=-1&&!o;a--){var s;a>=0?s=arguments[a]:(void 0===e&&(e=process.cwd()),s=e),t(s),0!==s.length&&(r=s+"/"+r,o=47===s.charCodeAt(0))}return r=n(r,!o),o?r.length>0?"/"+r:"/":r.length>0?r:"."},normalize:function(e){if(t(e),0===e.length)return".";var r=47===e.charCodeAt(0),o=47===e.charCodeAt(e.length-1);return 0!==(e=n(e,!r)).length||r||(e="."),e.length>0&&o&&(e+="/"),r?"/"+e:e},isAbsolute:function(e){return t(e),e.length>0&&47===e.charCodeAt(0)},join:function(){if(0===arguments.length)return".";for(var e,n=0;n<arguments.length;++n){var o=arguments[n];t(o),o.length>0&&(void 0===e?e=o:e+="/"+o)}return void 0===e?".":r.normalize(e)},relative:function(e,n){if(t(e),t(n),e===n)return"";if((e=r.resolve(e))===(n=r.resolve(n)))return"";for(var o=1;o<e.length&&47===e.charCodeAt(o);++o);for(var a=e.length,s=a-o,c=1;c<n.length&&47===n.charCodeAt(c);++c);for(var l=n.length-c,i=s<l?s:l,d=-1,u=0;u<=i;++u){if(u===i){if(l>i){if(47===n.charCodeAt(c+u))return n.slice(c+u+1);if(0===u)return n.slice(c+u)}else s>i&&(47===e.charCodeAt(o+u)?d=u:0===u&&(d=0));break}var f=e.charCodeAt(o+u);if(f!==n.charCodeAt(c+u))break;47===f&&(d=u)}var m="";for(u=o+d+1;u<=a;++u)u!==a&&47!==e.charCodeAt(u)||(0===m.length?m+="..":m+="/..");return m.length>0?m+n.slice(c+d):(c+=d,47===n.charCodeAt(c)&&++c,n.slice(c))},_makeLong:function(e){return e},dirname:function(e){if(t(e),0===e.length)return".";for(var n=e.charCodeAt(0),r=47===n,o=-1,a=!0,s=e.length-1;s>=1;--s)if(47===(n=e.charCodeAt(s))){if(!a){o=s;break}}else a=!1;return-1===o?r?"/":".":r&&1===o?"//":e.slice(0,o)},basename:function(e,n){if(void 0!==n&&"string"!=typeof n)throw new TypeError('"ext" argument must be a string');t(e);var r,o=0,a=-1,s=!0;if(void 0!==n&&n.length>0&&n.length<=e.length){if(n.length===e.length&&n===e)return"";var c=n.length-1,l=-1;for(r=e.length-1;r>=0;--r){var i=e.charCodeAt(r);if(47===i){if(!s){o=r+1;break}}else-1===l&&(s=!1,l=r+1),c>=0&&(i===n.charCodeAt(c)?-1==--c&&(a=r):(c=-1,a=l))}return o===a?a=l:-1===a&&(a=e.length),e.slice(o,a)}for(r=e.length-1;r>=0;--r)if(47===e.charCodeAt(r)){if(!s){o=r+1;break}}else-1===a&&(s=!1,a=r+1);return-1===a?"":e.slice(o,a)},extname:function(e){t(e);for(var n=-1,r=0,o=-1,a=!0,s=0,c=e.length-1;c>=0;--c){var l=e.charCodeAt(c);if(47!==l)-1===o&&(a=!1,o=c+1),46===l?-1===n?n=c:1!==s&&(s=1):-1!==n&&(s=-1);else if(!a){r=c+1;break}}return-1===n||-1===o||0===s||1===s&&n===o-1&&n===r+1?"":e.slice(n,o)},format:function(e){if(null===e||"object"!=typeof e)throw new TypeError('The "pathObject" argument must be of type Object. Received type '+typeof e);return function(e,t){var n=t.dir||t.root,r=t.base||(t.name||"")+(t.ext||"");return n?n===t.root?n+r:n+e+r:r}("/",e)},parse:function(e){t(e);var n={root:"",dir:"",base:"",ext:"",name:""};if(0===e.length)return n;var r,o=e.charCodeAt(0),a=47===o;a?(n.root="/",r=1):r=0;for(var s=-1,c=0,l=-1,i=!0,d=e.length-1,u=0;d>=r;--d)if(47!==(o=e.charCodeAt(d)))-1===l&&(i=!1,l=d+1),46===o?-1===s?s=d:1!==u&&(u=1):-1!==s&&(u=-1);else if(!i){c=d+1;break}return-1===s||-1===l||0===u||1===u&&s===l-1&&s===c+1?-1!==l&&(n.base=n.name=0===c&&a?e.slice(1,l):e.slice(c,l)):(0===c&&a?(n.name=e.slice(1,s),n.base=e.slice(1,l)):(n.name=e.slice(c,s),n.base=e.slice(c,l)),n.ext=e.slice(s,l)),c>0?n.dir=e.slice(0,c-1):a&&(n.dir="/"),n},sep:"/",delimiter:":",win32:null,posix:null};r.posix=r,e.exports=r},59627:(e,t)=>{"use strict";Object.defineProperty(t,"__esModule",{value:!0});t.default="data:image/png;base64,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"},77774:(e,t,n)=>{"use strict";n.r(t),n.d(t,{default:()=>r});const r={CodeBlockFilenameTab:"CodeBlockFilenameTab_T2zd",CodeBlockFrame:"CodeBlockFrame_FcUo",CodeBlockHeader:"CodeBlockHeader_GbSM",CodeBlockCodeFrame:"CodeBlockCodeFrame_lJeJ",WithImage:"WithImage_nmsh",CodeBlockPreviewFrame:"CodeBlockPreviewFrame_qNOC"}}}]);