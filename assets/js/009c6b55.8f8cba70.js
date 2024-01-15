"use strict";(self.webpackChunkstaticdocs_starter=self.webpackChunkstaticdocs_starter||[]).push([[514],{3905:function(e,n,t){t.r(n),t.d(n,{MDXContext:function(){return m},MDXProvider:function(){return x},mdx:function(){return N},useMDXComponents:function(){return s},withMDXComponents:function(){return p}});var a=t(67294);function l(e,n,t){return n in e?Object.defineProperty(e,n,{value:t,enumerable:!0,configurable:!0,writable:!0}):e[n]=t,e}function o(){return o=Object.assign||function(e){for(var n=1;n<arguments.length;n++){var t=arguments[n];for(var a in t)Object.prototype.hasOwnProperty.call(t,a)&&(e[a]=t[a])}return e},o.apply(this,arguments)}function r(e,n){var t=Object.keys(e);if(Object.getOwnPropertySymbols){var a=Object.getOwnPropertySymbols(e);n&&(a=a.filter((function(n){return Object.getOwnPropertyDescriptor(e,n).enumerable}))),t.push.apply(t,a)}return t}function d(e){for(var n=1;n<arguments.length;n++){var t=null!=arguments[n]?arguments[n]:{};n%2?r(Object(t),!0).forEach((function(n){l(e,n,t[n])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(t)):r(Object(t)).forEach((function(n){Object.defineProperty(e,n,Object.getOwnPropertyDescriptor(t,n))}))}return e}function i(e,n){if(null==e)return{};var t,a,l=function(e,n){if(null==e)return{};var t,a,l={},o=Object.keys(e);for(a=0;a<o.length;a++)t=o[a],n.indexOf(t)>=0||(l[t]=e[t]);return l}(e,n);if(Object.getOwnPropertySymbols){var o=Object.getOwnPropertySymbols(e);for(a=0;a<o.length;a++)t=o[a],n.indexOf(t)>=0||Object.prototype.propertyIsEnumerable.call(e,t)&&(l[t]=e[t])}return l}var m=a.createContext({}),p=function(e){return function(n){var t=s(n.components);return a.createElement(e,o({},n,{components:t}))}},s=function(e){var n=a.useContext(m),t=n;return e&&(t="function"==typeof e?e(n):d(d({},n),e)),t},x=function(e){var n=s(e.components);return a.createElement(m.Provider,{value:n},e.children)},u={inlineCode:"code",wrapper:function(e){var n=e.children;return a.createElement(a.Fragment,{},n)}},c=a.forwardRef((function(e,n){var t=e.components,l=e.mdxType,o=e.originalType,r=e.parentName,m=i(e,["components","mdxType","originalType","parentName"]),p=s(t),x=l,c=p["".concat(r,".").concat(x)]||p[x]||u[x]||o;return t?a.createElement(c,d(d({ref:n},m),{},{components:t})):a.createElement(c,d({ref:n},m))}));function N(e,n){var t=arguments,l=n&&n.mdxType;if("string"==typeof e||l){var o=t.length,r=new Array(o);r[0]=c;var d={};for(var i in n)hasOwnProperty.call(n,i)&&(d[i]=n[i]);d.originalType=e,d.mdxType="string"==typeof e?e:l,r[1]=d;for(var m=2;m<o;m++)r[m]=t[m];return a.createElement.apply(null,r)}return a.createElement.apply(null,t)}c.displayName="MDXCreateElement"},37994:function(e,n,t){t.r(n),t.d(n,{assets:function(){return p},contentTitle:function(){return i},default:function(){return u},frontMatter:function(){return d},metadata:function(){return m},toc:function(){return s}});var a=t(83117),l=t(80102),o=(t(67294),t(3905)),r=["components"],d={sidebar_position:2},i="Expectations",m={unversionedId:"advanced/expectations",id:"advanced/expectations",title:"Expectations",description:"Expectations are predictions of one or more lines of output. What you think a command will print out when you execute it. My expectation when I execute uname is that the operating system name is printed out to the shell. On a mac, I expect the following:",source:"@site/docs/advanced/expectations.md",sourceDirName:"advanced",slug:"/advanced/expectations",permalink:"/scrut/docs/advanced/expectations",draft:!1,editUrl:"https://www.internalfb.com/code/fbsource/fbcode/clifoundation/scrut/website/docs/advanced/expectations.md",tags:[],version:"current",sidebarPosition:2,frontMatter:{sidebar_position:2},sidebar:"tutorialSidebar",previous:{title:"File Formats",permalink:"/scrut/docs/advanced/file-formats"},next:{title:"Specifics",permalink:"/scrut/docs/advanced/specifics"}},p={},s=[{value:"Quantifiers",id:"quantifiers",level:2},{value:"Equal Expectation",id:"equal-expectation",level:2},{value:"Examples",id:"examples",level:3},{value:"Equal No EOL Expectation",id:"equal-no-eol-expectation",level:2},{value:"Examples",id:"examples-1",level:3},{value:"Glob Expectation",id:"glob-expectation",level:2},{value:"Examples",id:"examples-2",level:3},{value:"Regex Expectation",id:"regex-expectation",level:2},{value:"Examples",id:"examples-3",level:3},{value:"Escaped Expectation",id:"escaped-expectation",level:2},{value:"Examples",id:"examples-4",level:3},{value:"Escaped Glob Expectations",id:"escaped-glob-expectations",level:3}],x={toc:s};function u(e){var n=e.components,t=(0,l.Z)(e,r);return(0,o.mdx)("wrapper",(0,a.Z)({},x,t,{components:n,mdxType:"MDXLayout"}),(0,o.mdx)("h1",{id:"expectations"},"Expectations"),(0,o.mdx)("p",null,"Expectations are predictions of one or more lines of output. ",(0,o.mdx)("em",{parentName:"p"},"What you think a command will print out when you execute it"),". My expectation when I execute ",(0,o.mdx)("inlineCode",{parentName:"p"},"uname")," is that the operating system name is printed out to the shell. On a mac, I expect the following:"),(0,o.mdx)("pre",null,(0,o.mdx)("code",{parentName:"pre",className:"language-bash"},"$ uname\nDarwin\n")),(0,o.mdx)("blockquote",null,(0,o.mdx)("p",{parentName:"blockquote"},"See also: ",(0,o.mdx)("a",{parentName:"p",href:"/scrut/docs/advanced/specifics#stdout-and-stderr"},"STDOUT or STDERR? What is tested"))),(0,o.mdx)("p",null,"The Backus-Naur form for Expectations is sweet and short:"),(0,o.mdx)("pre",null,(0,o.mdx)("code",{parentName:"pre",className:"language-bnf"},' <expectation> ::= <expression> | <expression> (<mod>)\n  <expression> ::= TEXT\n         <mod> ::= <kind> | <quantifier> | <kind><quantifier>\n        <kind> ::= <equal-kind> | <no-eol-kind> | <escaped-kind> | <glob-kind> | <regex-kind>\n  <equal-kind> ::= "equal" | "eq"\n <no-eol-kind> ::= "no-eol"\n<escaped-kind> ::= "escaped" | "esc"\n   <glob-kind> ::= "glob" | "gl"\n  <regex-kind> ::= "regex" | "re"\n  <quantifier> ::= "?" | "*" | "+"\n')),(0,o.mdx)("h2",{id:"quantifiers"},"Quantifiers"),(0,o.mdx)("p",null,"The Quantifiers can be understood as following (nothing new if you are familiar with regular expressions):"),(0,o.mdx)("ul",null,(0,o.mdx)("li",{parentName:"ul"},(0,o.mdx)("strong",{parentName:"li"},(0,o.mdx)("inlineCode",{parentName:"strong"},"?")),": Zero or one occurrence; basically an optional output line"),(0,o.mdx)("li",{parentName:"ul"},(0,o.mdx)("strong",{parentName:"li"},(0,o.mdx)("inlineCode",{parentName:"strong"},"*")),": Any amount of occurrences (",(0,o.mdx)("inlineCode",{parentName:"li"},"0..n"),"); no line, one line, more lines - all good"),(0,o.mdx)("li",{parentName:"ul"},(0,o.mdx)("strong",{parentName:"li"},(0,o.mdx)("inlineCode",{parentName:"strong"},"+")),": One or more occurrences (",(0,o.mdx)("inlineCode",{parentName:"li"},"1..n"),"); at least one line, more are fine")),(0,o.mdx)("p",null,"Quantifiers can be used with most expectations, see the examples and description below for more details."),(0,o.mdx)("h2",{id:"equal-expectation"},"Equal Expectation"),(0,o.mdx)("p",null,"The Equal Expectation denotes a single line of output that ends in a ",(0,o.mdx)("a",{parentName:"p",href:"/scrut/docs/advanced/specifics#newline-handling"},"newline character"),". Because this expectation is the most common one you do not need to provide the specific kind. Here an example:"),(0,o.mdx)("pre",null,(0,o.mdx)("code",{parentName:"pre"},"A test\n\n```scrut\n$ echo Hello\nHello\n```\n")),(0,o.mdx)("p",null,"The line that consists only of ",(0,o.mdx)("inlineCode",{parentName:"p"},"Hello")," ",(0,o.mdx)("em",{parentName:"p"},"is")," the Equal Expectation and specifies that the (first line of the) output must be equal to ",(0,o.mdx)("inlineCode",{parentName:"p"},"Hello\\n")," (with ",(0,o.mdx)("inlineCode",{parentName:"p"},"\\n")," being the ",(0,o.mdx)("a",{parentName:"p",href:"/scrut/docs/advanced/specifics#newline-handling"},"newline of the operating system"),")."),(0,o.mdx)("p",null,"An extended for of the same Equal Expectation with explicit kind works as well and looks like that:"),(0,o.mdx)("pre",null,(0,o.mdx)("code",{parentName:"pre"},"A test\n\n```scrut\n$ echo Hello\nHello (equal)\n```\n")),(0,o.mdx)("p",null,"The explicit form makes most sense in conjunction with quantifiers:"),(0,o.mdx)("pre",null,(0,o.mdx)("code",{parentName:"pre"},'A test\n\n```scrut\n$ echo -e "Hello\\nHello\\nHello"\nHello (equal+)\n```\n')),(0,o.mdx)("h3",{id:"examples"},"Examples"),(0,o.mdx)("table",null,(0,o.mdx)("thead",{parentName:"table"},(0,o.mdx)("tr",{parentName:"thead"},(0,o.mdx)("th",{parentName:"tr",align:null},"Expression"),(0,o.mdx)("th",{parentName:"tr",align:null},"Meaning"))),(0,o.mdx)("tbody",{parentName:"table"},(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"Hello")),(0,o.mdx)("td",{parentName:"tr",align:null},"One output line of the form ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello\\n"))),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"Hello (equal)")),(0,o.mdx)("td",{parentName:"tr",align:null},"One output line of the form ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello\\n"))),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"Hello (?)")),(0,o.mdx)("td",{parentName:"tr",align:null},"Optional (zero or one) output line of the form ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello\\n"))),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"Hello (*)")),(0,o.mdx)("td",{parentName:"tr",align:null},"Any amount (0..n) of output lines of the form ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello\\n"))),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"Hello (+)")),(0,o.mdx)("td",{parentName:"tr",align:null},"One or more (1..n) of output lines of the form ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello\\n"))),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"Hello (equal*)")),(0,o.mdx)("td",{parentName:"tr",align:null},"Any amount (0..n) of output lines of the form ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello\\n"))),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"Hello (equal+)")),(0,o.mdx)("td",{parentName:"tr",align:null},"One or more (1..n) of output lines of the form ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello\\n"))))),(0,o.mdx)("blockquote",null,(0,o.mdx)("p",{parentName:"blockquote"},(0,o.mdx)("strong",{parentName:"p"},"Note"),": You can use ",(0,o.mdx)("inlineCode",{parentName:"p"},"eq")," as a shorthand for ",(0,o.mdx)("inlineCode",{parentName:"p"},"equal"))),(0,o.mdx)("h2",{id:"equal-no-eol-expectation"},"Equal No EOL Expectation"),(0,o.mdx)("p",null,"Very close to the above, but much rarer, the ",(0,o.mdx)("em",{parentName:"p"},"Equal No EOL Expectation")," matches lines that do ",(0,o.mdx)("em",{parentName:"p"},"not")," end in a newline. Consider:"),(0,o.mdx)("pre",null,(0,o.mdx)("code",{parentName:"pre"},"A test\n\n```scrut\n$ echo -n Hello\nHello (no-eol)\n```\n")),(0,o.mdx)("p",null,"The above ",(0,o.mdx)("inlineCode",{parentName:"p"},"echo -n Hello")," prints ",(0,o.mdx)("inlineCode",{parentName:"p"},"Hello")," ",(0,o.mdx)("em",{parentName:"p"},"without")," a tailing newline character (there is no ",(0,o.mdx)("inlineCode",{parentName:"p"},"\\n")," at the end of ",(0,o.mdx)("inlineCode",{parentName:"p"},"Hello"),")."),(0,o.mdx)("p",null,"This Expectation could possibly only be the last line of output, so quantifiers make little sense."),(0,o.mdx)("h3",{id:"examples-1"},"Examples"),(0,o.mdx)("table",null,(0,o.mdx)("thead",{parentName:"table"},(0,o.mdx)("tr",{parentName:"thead"},(0,o.mdx)("th",{parentName:"tr",align:null},"Expression"),(0,o.mdx)("th",{parentName:"tr",align:null},"Meaning"))),(0,o.mdx)("tbody",{parentName:"table"},(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"Hello (no-eol)")),(0,o.mdx)("td",{parentName:"tr",align:null},"One output line of the form ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello")," - a line that does not end in newline")))),(0,o.mdx)("h2",{id:"glob-expectation"},"Glob Expectation"),(0,o.mdx)("p",null,"Glob Expectations are support two wildcard characters:"),(0,o.mdx)("ul",null,(0,o.mdx)("li",{parentName:"ul"},(0,o.mdx)("inlineCode",{parentName:"li"},"?")," matches exactly one occurrence of any character"),(0,o.mdx)("li",{parentName:"ul"},(0,o.mdx)("inlineCode",{parentName:"li"},"*")," matches arbitrary many (including zero) occurrences of any character")),(0,o.mdx)("p",null,"Together with quantifiers, this allows for powerful if imprecise matches of output lines."),(0,o.mdx)("pre",null,(0,o.mdx)("code",{parentName:"pre"},'This will work\n\n```scrut\n$ echo Hello You\nHello* (glob)\n```\n\nThis will work, too\n\n```scrut\n$ echo -e "Hello\\nHello There\\nHello World"\nHello* (glob+)\n```\n')),(0,o.mdx)("h3",{id:"examples-2"},"Examples"),(0,o.mdx)("table",null,(0,o.mdx)("thead",{parentName:"table"},(0,o.mdx)("tr",{parentName:"thead"},(0,o.mdx)("th",{parentName:"tr",align:null},"Expression"),(0,o.mdx)("th",{parentName:"tr",align:null},"Meaning"))),(0,o.mdx)("tbody",{parentName:"table"},(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"Hello? (glob)")),(0,o.mdx)("td",{parentName:"tr",align:null},"A single output line that starts with ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello")," followed by one character")),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"Hello* (glob)")),(0,o.mdx)("td",{parentName:"tr",align:null},"A single output line that starts with ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello"))),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"*Hello* (glob)")),(0,o.mdx)("td",{parentName:"tr",align:null},"A single output line that contains ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello"))),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"*Hello (glob)")),(0,o.mdx)("td",{parentName:"tr",align:null},"A single output line that ends with ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello"))),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"*Hello* (glob?)")),(0,o.mdx)("td",{parentName:"tr",align:null},"An optional output line that contains ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello"))),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"*Hello* (glob*)")),(0,o.mdx)("td",{parentName:"tr",align:null},"Any amount (0..n) of output lines that contain ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello"))),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"*Hello* (glob+)")),(0,o.mdx)("td",{parentName:"tr",align:null},"One or more (1..n) of output lines that contain ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello"))))),(0,o.mdx)("blockquote",null,(0,o.mdx)("p",{parentName:"blockquote"},(0,o.mdx)("strong",{parentName:"p"},"Note"),": You can use ",(0,o.mdx)("inlineCode",{parentName:"p"},"gl")," as a shorthand for ",(0,o.mdx)("inlineCode",{parentName:"p"},"glob"))),(0,o.mdx)("h2",{id:"regex-expectation"},"Regex Expectation"),(0,o.mdx)("p",null,(0,o.mdx)("a",{parentName:"p",href:"https://en.wikipedia.org/wiki/Regular_expression"},"Regular Expressions")," are the most powerful, yet precise, output describing rules that are supported. That comes at the price of complexity. Explaining regular expression syntax literarily ",(0,o.mdx)("a",{parentName:"p",href:"https://www.goodreads.com/search?q=Regular+Expression"},"fills books"),", so here is not the place to attempt that. Rust uses a ",(0,o.mdx)("a",{parentName:"p",href:"https://github.com/google/re2/wiki"},"RE2")," inspired engine. Its ",(0,o.mdx)("a",{parentName:"p",href:"https://docs.rs/regex/latest/regex/#syntax"},"syntax")," is very similar to it. It most notably differs from Perl's ",(0,o.mdx)("a",{parentName:"p",href:"https://en.wikipedia.org/wiki/Perl_Compatible_Regular_Expressions"},"PCRE")," because it doesn't support backtracking to ensure good performance."),(0,o.mdx)("p",null,"Nonetheless, an obligatory example:"),(0,o.mdx)("pre",null,(0,o.mdx)("code",{parentName:"pre"},'This will work\n\n```scrut\n$ echo Hello You\nHello.+ (regex)\n```\n\nThis will work, too:\n\n```scrut\n$ echo -e "Hello\\nEnding in Hello\\nHello Start"\n.*Hello.* (regex+)\n```\n')),(0,o.mdx)("p",null,(0,o.mdx)("strong",{parentName:"p"},"Note"),": All Regex Expectations are implicitly embedded within start and end markers: ",(0,o.mdx)("inlineCode",{parentName:"p"},"^<expression>$"),". This means ",(0,o.mdx)("em",{parentName:"p"},"regular expressions are always assumed to match the full line"),". Use ",(0,o.mdx)("inlineCode",{parentName:"p"},".*")," to explicitly match only at the end of (",(0,o.mdx)("inlineCode",{parentName:"p"},".*<expression> (regex)"),"), or the start of (",(0,o.mdx)("inlineCode",{parentName:"p"},"<expression>.* (regex)"),"), or anywhere in (",(0,o.mdx)("inlineCode",{parentName:"p"},".*<expression>.* (regex)"),") a line."),(0,o.mdx)("h3",{id:"examples-3"},"Examples"),(0,o.mdx)("table",null,(0,o.mdx)("thead",{parentName:"table"},(0,o.mdx)("tr",{parentName:"thead"},(0,o.mdx)("th",{parentName:"tr",align:null},"Expression"),(0,o.mdx)("th",{parentName:"tr",align:null},"Meaning"))),(0,o.mdx)("tbody",{parentName:"table"},(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"Hello.* (regex)")),(0,o.mdx)("td",{parentName:"tr",align:null},"A single output line that starts with ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello"))),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},".*Hello.* (regex)")),(0,o.mdx)("td",{parentName:"tr",align:null},"A single output line that contains ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello"))),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},".*Hello (regex)")),(0,o.mdx)("td",{parentName:"tr",align:null},"A single output line that ends with ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello"))),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},".*Hello.* (regex?)")),(0,o.mdx)("td",{parentName:"tr",align:null},"An optional output line that contains ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello"))),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},".*Hello.* (regex*)")),(0,o.mdx)("td",{parentName:"tr",align:null},"Any amount (0..n) of output lines that contain ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello"))),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},".*Hello.* (regex+)")),(0,o.mdx)("td",{parentName:"tr",align:null},"One or more (1..n) of output lines that contain ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello"))),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"Foo: [0-9]+ (regex+)")),(0,o.mdx)("td",{parentName:"tr",align:null},"One or more (1..n) of output lines that start with ",(0,o.mdx)("inlineCode",{parentName:"td"},"Foo")," followed by a colon ",(0,o.mdx)("inlineCode",{parentName:"td"},":"),", a whitespace ",(0,o.mdx)("inlineCode",{parentName:"td"}," ")," and then only numbers till the end of the line")))),(0,o.mdx)("blockquote",null,(0,o.mdx)("p",{parentName:"blockquote"},(0,o.mdx)("strong",{parentName:"p"},"Note"),": You can use ",(0,o.mdx)("inlineCode",{parentName:"p"},"re")," as a shorthand for ",(0,o.mdx)("inlineCode",{parentName:"p"},"regex"))),(0,o.mdx)("h2",{id:"escaped-expectation"},"Escaped Expectation"),(0,o.mdx)("p",null,"CLIs usually only do (and mostly should) print out, well, printable characters. However, there are scenarios which you need to write binary data to STDOUT (e.g. consider a command line that generates a binary JPEG and pipes that output into yet another command that shrinks it or something ",(0,o.mdx)("inlineCode",{parentName:"p"},"$ create-jpeg | shrink-image"),"). In addition to that adding colors can help make the output better readable - and some daredevils even throw in some emojis \ud83e\udd2c. Lastly, consider the good old tab character ",(0,o.mdx)("inlineCode",{parentName:"p"},"\\t"),", which may be hard to read (or write) in a text editor."),(0,o.mdx)("p",null,"Scrut tests live in Markdown or Cram files that are intended to be edited by users. They should not contain binary, non-printable data. To that end, any non-printable output can be denoted in it's hexadecimal escaped form ",(0,o.mdx)("inlineCode",{parentName:"p"},"\\xAB")," (with ",(0,o.mdx)("inlineCode",{parentName:"p"},"AB")," being the hexadecimal value of the bytecode of the character) or ",(0,o.mdx)("inlineCode",{parentName:"p"},"\\t")," to denote tab characters."),(0,o.mdx)("p",null,"The following example shows an expectation of a string that renders as a bold, red font on the command line"),(0,o.mdx)("pre",null,(0,o.mdx)("code",{parentName:"pre"},"Colorful fun\n\n```scrut\n$ echo -e 'Foo \\033[1;31mBar\\033[0m Baz'\nFoo \\x1b[1mBar\\x1b[0m Baz (escaped)\n```\n")),(0,o.mdx)("p",null,"Or consider some program that prints out two ",(0,o.mdx)("inlineCode",{parentName:"p"},"\\x00")," separated strings:"),(0,o.mdx)("pre",null,(0,o.mdx)("code",{parentName:"pre"},"Colorful fun\n\n```scrut\n$ some-program\nfoo\\x00bar (escaped)\n```\n")),(0,o.mdx)("p",null,"Or again, the good old tab character:"),(0,o.mdx)("pre",null,(0,o.mdx)("code",{parentName:"pre"},"Love the CSV\n\n```scrut\n$ csv-generator\nfoo\\tbar\\tbaz (escaped)\n```\n")),(0,o.mdx)("blockquote",null,(0,o.mdx)("p",{parentName:"blockquote"},(0,o.mdx)("strong",{parentName:"p"},"Note"),": Newlines are ignored for Escaped Expectations. So ",(0,o.mdx)("inlineCode",{parentName:"p"},"foo\\tbar (escaped)")," matches both ",(0,o.mdx)("inlineCode",{parentName:"p"},"foo\\tbar\\n")," and ",(0,o.mdx)("inlineCode",{parentName:"p"},"foo\\tbar"),".")),(0,o.mdx)("h3",{id:"examples-4"},"Examples"),(0,o.mdx)("table",null,(0,o.mdx)("thead",{parentName:"table"},(0,o.mdx)("tr",{parentName:"thead"},(0,o.mdx)("th",{parentName:"tr",align:null},"Expression"),(0,o.mdx)("th",{parentName:"tr",align:null},"Meaning"))),(0,o.mdx)("tbody",{parentName:"table"},(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"Hello\\tWorld (escaped)")),(0,o.mdx)("td",{parentName:"tr",align:null},"One output line of that starts with ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello"),", followed by a tab character, followed by ",(0,o.mdx)("inlineCode",{parentName:"td"},"World"))),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"Hello\\tWorld (escaped?)")),(0,o.mdx)("td",{parentName:"tr",align:null},"An optional output line that contains ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello"),", followed by a tab character, followed by ",(0,o.mdx)("inlineCode",{parentName:"td"},"World"))),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"Hello\\tWorld (escaped*)")),(0,o.mdx)("td",{parentName:"tr",align:null},"Any amount (0..n) of output lines that contain ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello\\tWorld"),", followed by a tab character, followed by ",(0,o.mdx)("inlineCode",{parentName:"td"},"World"))),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"Hello\\tWorld (escaped+)")),(0,o.mdx)("td",{parentName:"tr",align:null},"One or more (1..n) of output lines that contain ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello\\tWorld"),", followed by a tab character, followed by ",(0,o.mdx)("inlineCode",{parentName:"td"},"World"))))),(0,o.mdx)("blockquote",null,(0,o.mdx)("p",{parentName:"blockquote"},(0,o.mdx)("strong",{parentName:"p"},"Note"),": You can use ",(0,o.mdx)("inlineCode",{parentName:"p"},"esc")," as a shorthand for ",(0,o.mdx)("inlineCode",{parentName:"p"},"escaped"))),(0,o.mdx)("h3",{id:"escaped-glob-expectations"},"Escaped Glob Expectations"),(0,o.mdx)("p",null,"Because it came up often enough, you can use ",(0,o.mdx)("inlineCode",{parentName:"p"},"(escaped)")," in combination with ",(0,o.mdx)("inlineCode",{parentName:"p"},"(glob)"),":"),(0,o.mdx)("pre",null,(0,o.mdx)("code",{parentName:"pre"},"Glob escaped output\n\n```scrut\n$ csv-generator\nfoo\\t* (escaped) (glob+)\nbar\\tbaz (escaped)\n```\n")),(0,o.mdx)("p",null,"The above exports one or more lines of output that start with ",(0,o.mdx)("inlineCode",{parentName:"p"},"foo")," followed by tab. The last line of output is expected to be ",(0,o.mdx)("inlineCode",{parentName:"p"},"bar"),", followed by tab, followed by ",(0,o.mdx)("inlineCode",{parentName:"p"},"baz"),"."),(0,o.mdx)("table",null,(0,o.mdx)("thead",{parentName:"table"},(0,o.mdx)("tr",{parentName:"thead"},(0,o.mdx)("th",{parentName:"tr",align:null},"Expression"),(0,o.mdx)("th",{parentName:"tr",align:null},"Meaning"))),(0,o.mdx)("tbody",{parentName:"table"},(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"Hello\\tWorld* (escaped) (glob)")),(0,o.mdx)("td",{parentName:"tr",align:null},"One output line of that starts with ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello"),", followed by a tab character, followed by ",(0,o.mdx)("inlineCode",{parentName:"td"},"World"),", followed by anything")),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"Hello\\tWorld* (escaped) (glob?)")),(0,o.mdx)("td",{parentName:"tr",align:null},"An optional output line that contains ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello"),", followed by a tab character, followed by ",(0,o.mdx)("inlineCode",{parentName:"td"},"World"),", followed by anything")),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"Hello\\tWorld* (escaped) (glob*)")),(0,o.mdx)("td",{parentName:"tr",align:null},"Any amount (0..n) of output lines that contain ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello\\tWorld"),", followed by a tab character, followed by ",(0,o.mdx)("inlineCode",{parentName:"td"},"World"),", followed by anything")),(0,o.mdx)("tr",{parentName:"tbody"},(0,o.mdx)("td",{parentName:"tr",align:null},(0,o.mdx)("inlineCode",{parentName:"td"},"Hello\\tWorld* (escaped) (glob+)")),(0,o.mdx)("td",{parentName:"tr",align:null},"One or more (1..n) of output lines that contain ",(0,o.mdx)("inlineCode",{parentName:"td"},"Hello\\tWorld"),", followed by a tab character, followed by ",(0,o.mdx)("inlineCode",{parentName:"td"},"World"),", followed by anything")))),(0,o.mdx)("blockquote",null,(0,o.mdx)("p",{parentName:"blockquote"},(0,o.mdx)("strong",{parentName:"p"},"Note"),": You can use shorthands for either. Quantifiers must be always on ",(0,o.mdx)("inlineCode",{parentName:"p"},"glob"),".")))}u.isMDXComponent=!0}}]);