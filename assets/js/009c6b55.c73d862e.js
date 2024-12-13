"use strict";(self.webpackChunkstaticdocs_starter=self.webpackChunkstaticdocs_starter||[]).push([[9097],{40761:(e,n,l)=>{l.r(n),l.d(n,{assets:()=>r,contentTitle:()=>d,default:()=>a,frontMatter:()=>i,metadata:()=>s,toc:()=>c});const s=JSON.parse('{"id":"advanced/expectations","title":"Expectations","description":"Expectations are predictions of one or more lines of output. What you think a command will print out when you execute it. My expectation when I execute uname is that the operating system name is printed out to the shell. On a mac, I expect the following:","source":"@site/docs/advanced/expectations.md","sourceDirName":"advanced","slug":"/advanced/expectations","permalink":"/scrut/docs/advanced/expectations","draft":false,"unlisted":false,"editUrl":"https://www.internalfb.com/code/fbsource/fbcode/clifoundation/scrut/website/docs/advanced/expectations.md","tags":[],"version":"current","sidebarPosition":2,"frontMatter":{"sidebar_position":2},"sidebar":"tutorialSidebar","previous":{"title":"File Formats","permalink":"/scrut/docs/advanced/file-formats"},"next":{"title":"Specifics","permalink":"/scrut/docs/advanced/specifics"}}');var o=l(74848),t=l(28453);const i={sidebar_position:2},d="Expectations",r={},c=[{value:"Quantifiers",id:"quantifiers",level:2},{value:"Equal Expectation",id:"equal-expectation",level:2},{value:"Examples",id:"examples",level:3},{value:"Equal No EOL Expectation",id:"equal-no-eol-expectation",level:2},{value:"Examples",id:"examples-1",level:3},{value:"Glob Expectation",id:"glob-expectation",level:2},{value:"Examples",id:"examples-2",level:3},{value:"Regex Expectation",id:"regex-expectation",level:2},{value:"Examples",id:"examples-3",level:3},{value:"Escaped Expectation",id:"escaped-expectation",level:2},{value:"Examples",id:"examples-4",level:3},{value:"Escaped Glob Expectations",id:"escaped-glob-expectations",level:3}];function h(e){const n={a:"a",blockquote:"blockquote",code:"code",em:"em",h1:"h1",h2:"h2",h3:"h3",header:"header",li:"li",p:"p",pre:"pre",strong:"strong",table:"table",tbody:"tbody",td:"td",th:"th",thead:"thead",tr:"tr",ul:"ul",...(0,t.R)(),...e.components};return(0,o.jsxs)(o.Fragment,{children:[(0,o.jsx)(n.header,{children:(0,o.jsx)(n.h1,{id:"expectations",children:"Expectations"})}),"\n",(0,o.jsxs)(n.p,{children:["Expectations are predictions of one or more lines of output. ",(0,o.jsx)(n.em,{children:"What you think a command will print out when you execute it"}),". My expectation when I execute ",(0,o.jsx)(n.code,{children:"uname"})," is that the operating system name is printed out to the shell. On a mac, I expect the following:"]}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-bash",children:"$ uname\nDarwin\n"})}),"\n",(0,o.jsxs)(n.blockquote,{children:["\n",(0,o.jsxs)(n.p,{children:["See also: ",(0,o.jsx)(n.a,{href:"/scrut/docs/advanced/specifics#stdout-and-stderr",children:"STDOUT or STDERR? What is tested"})]}),"\n"]}),"\n",(0,o.jsx)(n.p,{children:"The Backus-Naur form for Expectations is sweet and short:"}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-bnf",children:' <expectation> ::= <expression> | <expression> (<mod>)\n  <expression> ::= TEXT\n         <mod> ::= <kind> | <quantifier> | <kind><quantifier>\n        <kind> ::= <equal-kind> | <no-eol-kind> | <escaped-kind> | <glob-kind> | <regex-kind>\n  <equal-kind> ::= "equal" | "eq"\n <no-eol-kind> ::= "no-eol"\n<escaped-kind> ::= "escaped" | "esc"\n   <glob-kind> ::= "glob" | "gl"\n  <regex-kind> ::= "regex" | "re"\n  <quantifier> ::= "?" | "*" | "+"\n'})}),"\n",(0,o.jsx)(n.h2,{id:"quantifiers",children:"Quantifiers"}),"\n",(0,o.jsx)(n.p,{children:"The Quantifiers can be understood as following (nothing new if you are familiar with regular expressions):"}),"\n",(0,o.jsxs)(n.ul,{children:["\n",(0,o.jsxs)(n.li,{children:[(0,o.jsx)(n.strong,{children:(0,o.jsx)(n.code,{children:"?"})}),": Zero or one occurrence; basically an optional output line"]}),"\n",(0,o.jsxs)(n.li,{children:[(0,o.jsx)(n.strong,{children:(0,o.jsx)(n.code,{children:"*"})}),": Any amount of occurrences (",(0,o.jsx)(n.code,{children:"0..n"}),"); no line, one line, more lines - all good"]}),"\n",(0,o.jsxs)(n.li,{children:[(0,o.jsx)(n.strong,{children:(0,o.jsx)(n.code,{children:"+"})}),": One or more occurrences (",(0,o.jsx)(n.code,{children:"1..n"}),"); at least one line, more are fine"]}),"\n"]}),"\n",(0,o.jsx)(n.p,{children:"Quantifiers can be used with most expectations, see the examples and description below for more details."}),"\n",(0,o.jsx)(n.h2,{id:"equal-expectation",children:"Equal Expectation"}),"\n",(0,o.jsxs)(n.p,{children:["The Equal Expectation denotes a single line of output that ends in a ",(0,o.jsx)(n.a,{href:"/scrut/docs/advanced/specifics#newline-handling",children:"newline character"}),". Because this expectation is the most common one you do not need to provide the specific kind. Here an example:"]}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{children:"A test\n\n```scrut\n$ echo Hello\nHello\n```\n"})}),"\n",(0,o.jsxs)(n.p,{children:["The line that consists only of ",(0,o.jsx)(n.code,{children:"Hello"})," ",(0,o.jsx)(n.em,{children:"is"})," the Equal Expectation and specifies that the (first line of the) output must be equal to ",(0,o.jsx)(n.code,{children:"Hello\\n"})," (with ",(0,o.jsx)(n.code,{children:"\\n"})," being the ",(0,o.jsx)(n.a,{href:"/scrut/docs/advanced/specifics#newline-handling",children:"newline of the operating system"}),")."]}),"\n",(0,o.jsx)(n.p,{children:"An extended for of the same Equal Expectation with explicit kind works as well and looks like that:"}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{children:"A test\n\n```scrut\n$ echo Hello\nHello (equal)\n```\n"})}),"\n",(0,o.jsx)(n.p,{children:"The explicit form makes most sense in conjunction with quantifiers:"}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{children:'A test\n\n```scrut\n$ echo -e "Hello\\nHello\\nHello"\nHello (equal+)\n```\n'})}),"\n",(0,o.jsx)(n.h3,{id:"examples",children:"Examples"}),"\n",(0,o.jsxs)(n.table,{children:[(0,o.jsx)(n.thead,{children:(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.th,{children:"Expression"}),(0,o.jsx)(n.th,{children:"Meaning"})]})}),(0,o.jsxs)(n.tbody,{children:[(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"Hello"})}),(0,o.jsxs)(n.td,{children:["One output line of the form ",(0,o.jsx)(n.code,{children:"Hello\\n"})]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"Hello (equal)"})}),(0,o.jsxs)(n.td,{children:["One output line of the form ",(0,o.jsx)(n.code,{children:"Hello\\n"})]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"Hello (?)"})}),(0,o.jsxs)(n.td,{children:["Optional (zero or one) output line of the form ",(0,o.jsx)(n.code,{children:"Hello\\n"})]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"Hello (*)"})}),(0,o.jsxs)(n.td,{children:["Any amount (0..n) of output lines of the form ",(0,o.jsx)(n.code,{children:"Hello\\n"})]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"Hello (+)"})}),(0,o.jsxs)(n.td,{children:["One or more (1..n) of output lines of the form ",(0,o.jsx)(n.code,{children:"Hello\\n"})]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"Hello (equal*)"})}),(0,o.jsxs)(n.td,{children:["Any amount (0..n) of output lines of the form ",(0,o.jsx)(n.code,{children:"Hello\\n"})]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"Hello (equal+)"})}),(0,o.jsxs)(n.td,{children:["One or more (1..n) of output lines of the form ",(0,o.jsx)(n.code,{children:"Hello\\n"})]})]})]})]}),"\n",(0,o.jsxs)(n.blockquote,{children:["\n",(0,o.jsxs)(n.p,{children:[(0,o.jsx)(n.strong,{children:"Note"}),": You can use ",(0,o.jsx)(n.code,{children:"eq"})," as a shorthand for ",(0,o.jsx)(n.code,{children:"equal"})]}),"\n"]}),"\n",(0,o.jsx)(n.h2,{id:"equal-no-eol-expectation",children:"Equal No EOL Expectation"}),"\n",(0,o.jsxs)(n.p,{children:["Very close to the above, but much rarer, the ",(0,o.jsx)(n.em,{children:"Equal No EOL Expectation"})," matches lines that do ",(0,o.jsx)(n.em,{children:"not"})," end in a newline. Consider:"]}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{children:"A test\n\n```scrut\n$ echo -n Hello\nHello (no-eol)\n```\n"})}),"\n",(0,o.jsxs)(n.p,{children:["The above ",(0,o.jsx)(n.code,{children:"echo -n Hello"})," prints ",(0,o.jsx)(n.code,{children:"Hello"})," ",(0,o.jsx)(n.em,{children:"without"})," a tailing newline character (there is no ",(0,o.jsx)(n.code,{children:"\\n"})," at the end of ",(0,o.jsx)(n.code,{children:"Hello"}),")."]}),"\n",(0,o.jsx)(n.p,{children:"This Expectation could possibly only be the last line of output, so quantifiers make little sense."}),"\n",(0,o.jsx)(n.h3,{id:"examples-1",children:"Examples"}),"\n",(0,o.jsxs)(n.table,{children:[(0,o.jsx)(n.thead,{children:(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.th,{children:"Expression"}),(0,o.jsx)(n.th,{children:"Meaning"})]})}),(0,o.jsx)(n.tbody,{children:(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"Hello (no-eol)"})}),(0,o.jsxs)(n.td,{children:["One output line of the form ",(0,o.jsx)(n.code,{children:"Hello"})," - a line that does not end in newline"]})]})})]}),"\n",(0,o.jsx)(n.h2,{id:"glob-expectation",children:"Glob Expectation"}),"\n",(0,o.jsx)(n.p,{children:"Glob Expectations are support two wildcard characters:"}),"\n",(0,o.jsxs)(n.ul,{children:["\n",(0,o.jsxs)(n.li,{children:[(0,o.jsx)(n.code,{children:"?"})," matches exactly one occurrence of any character"]}),"\n",(0,o.jsxs)(n.li,{children:[(0,o.jsx)(n.code,{children:"*"})," matches arbitrary many (including zero) occurrences of any character"]}),"\n"]}),"\n",(0,o.jsx)(n.p,{children:"Together with quantifiers, this allows for powerful if imprecise matches of output lines."}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{children:'This will work\n\n```scrut\n$ echo Hello You\nHello* (glob)\n```\n\nThis will work, too\n\n```scrut\n$ echo -e "Hello\\nHello There\\nHello World"\nHello* (glob+)\n```\n'})}),"\n",(0,o.jsx)(n.h3,{id:"examples-2",children:"Examples"}),"\n",(0,o.jsxs)(n.table,{children:[(0,o.jsx)(n.thead,{children:(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.th,{children:"Expression"}),(0,o.jsx)(n.th,{children:"Meaning"})]})}),(0,o.jsxs)(n.tbody,{children:[(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"Hello? (glob)"})}),(0,o.jsxs)(n.td,{children:["A single output line that starts with ",(0,o.jsx)(n.code,{children:"Hello"})," followed by one character"]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"Hello* (glob)"})}),(0,o.jsxs)(n.td,{children:["A single output line that starts with ",(0,o.jsx)(n.code,{children:"Hello"})]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"*Hello* (glob)"})}),(0,o.jsxs)(n.td,{children:["A single output line that contains ",(0,o.jsx)(n.code,{children:"Hello"})]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"*Hello (glob)"})}),(0,o.jsxs)(n.td,{children:["A single output line that ends with ",(0,o.jsx)(n.code,{children:"Hello"})]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"*Hello* (glob?)"})}),(0,o.jsxs)(n.td,{children:["An optional output line that contains ",(0,o.jsx)(n.code,{children:"Hello"})]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"*Hello* (glob*)"})}),(0,o.jsxs)(n.td,{children:["Any amount (0..n) of output lines that contain ",(0,o.jsx)(n.code,{children:"Hello"})]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"*Hello* (glob+)"})}),(0,o.jsxs)(n.td,{children:["One or more (1..n) of output lines that contain ",(0,o.jsx)(n.code,{children:"Hello"})]})]})]})]}),"\n",(0,o.jsxs)(n.blockquote,{children:["\n",(0,o.jsxs)(n.p,{children:[(0,o.jsx)(n.strong,{children:"Note"}),": You can use ",(0,o.jsx)(n.code,{children:"gl"})," as a shorthand for ",(0,o.jsx)(n.code,{children:"glob"})]}),"\n"]}),"\n",(0,o.jsx)(n.h2,{id:"regex-expectation",children:"Regex Expectation"}),"\n",(0,o.jsxs)(n.p,{children:[(0,o.jsx)(n.a,{href:"https://en.wikipedia.org/wiki/Regular_expression",children:"Regular Expressions"})," are the most powerful, yet precise, output describing rules that are supported. That comes at the price of complexity. Explaining regular expression syntax literarily ",(0,o.jsx)(n.a,{href:"https://www.goodreads.com/search?q=Regular+Expression",children:"fills books"}),", so here is not the place to attempt that. Rust uses a ",(0,o.jsx)(n.a,{href:"https://github.com/google/re2/wiki",children:"RE2"})," inspired engine. Its ",(0,o.jsx)(n.a,{href:"https://docs.rs/regex/latest/regex/#syntax",children:"syntax"})," is very similar to it. It most notably differs from Perl's ",(0,o.jsx)(n.a,{href:"https://en.wikipedia.org/wiki/Perl_Compatible_Regular_Expressions",children:"PCRE"})," because it doesn't support backtracking to ensure good performance."]}),"\n",(0,o.jsx)(n.p,{children:"Nonetheless, an obligatory example:"}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{children:'This will work\n\n```scrut\n$ echo Hello You\nHello.+ (regex)\n```\n\nThis will work, too:\n\n```scrut\n$ echo -e "Hello\\nEnding in Hello\\nHello Start"\n.*Hello.* (regex+)\n```\n'})}),"\n",(0,o.jsxs)(n.p,{children:[(0,o.jsx)(n.strong,{children:"Note"}),": All Regex Expectations are implicitly embedded within start and end markers: ",(0,o.jsx)(n.code,{children:"^<expression>$"}),". This means ",(0,o.jsx)(n.em,{children:"regular expressions are always assumed to match the full line"}),". Use ",(0,o.jsx)(n.code,{children:".*"})," to explicitly match only at the end of (",(0,o.jsx)(n.code,{children:".*<expression> (regex)"}),"), or the start of (",(0,o.jsx)(n.code,{children:"<expression>.* (regex)"}),"), or anywhere in (",(0,o.jsx)(n.code,{children:".*<expression>.* (regex)"}),") a line."]}),"\n",(0,o.jsx)(n.h3,{id:"examples-3",children:"Examples"}),"\n",(0,o.jsxs)(n.table,{children:[(0,o.jsx)(n.thead,{children:(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.th,{children:"Expression"}),(0,o.jsx)(n.th,{children:"Meaning"})]})}),(0,o.jsxs)(n.tbody,{children:[(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"Hello.* (regex)"})}),(0,o.jsxs)(n.td,{children:["A single output line that starts with ",(0,o.jsx)(n.code,{children:"Hello"})]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:".*Hello.* (regex)"})}),(0,o.jsxs)(n.td,{children:["A single output line that contains ",(0,o.jsx)(n.code,{children:"Hello"})]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:".*Hello (regex)"})}),(0,o.jsxs)(n.td,{children:["A single output line that ends with ",(0,o.jsx)(n.code,{children:"Hello"})]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:".*Hello.* (regex?)"})}),(0,o.jsxs)(n.td,{children:["An optional output line that contains ",(0,o.jsx)(n.code,{children:"Hello"})]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:".*Hello.* (regex*)"})}),(0,o.jsxs)(n.td,{children:["Any amount (0..n) of output lines that contain ",(0,o.jsx)(n.code,{children:"Hello"})]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:".*Hello.* (regex+)"})}),(0,o.jsxs)(n.td,{children:["One or more (1..n) of output lines that contain ",(0,o.jsx)(n.code,{children:"Hello"})]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"Foo: [0-9]+ (regex+)"})}),(0,o.jsxs)(n.td,{children:["One or more (1..n) of output lines that start with ",(0,o.jsx)(n.code,{children:"Foo"})," followed by a colon ",(0,o.jsx)(n.code,{children:":"}),", a whitespace ",(0,o.jsx)(n.code,{children:" "})," and then only numbers till the end of the line"]})]})]})]}),"\n",(0,o.jsxs)(n.blockquote,{children:["\n",(0,o.jsxs)(n.p,{children:[(0,o.jsx)(n.strong,{children:"Note"}),": You can use ",(0,o.jsx)(n.code,{children:"re"})," as a shorthand for ",(0,o.jsx)(n.code,{children:"regex"})]}),"\n"]}),"\n",(0,o.jsx)(n.h2,{id:"escaped-expectation",children:"Escaped Expectation"}),"\n",(0,o.jsxs)(n.p,{children:["CLIs usually only do (and mostly should) print out, well, printable characters. However, there are scenarios which you need to write binary data to STDOUT (e.g. consider a command line that generates a binary JPEG and pipes that output into yet another command that shrinks it or something ",(0,o.jsx)(n.code,{children:"$ create-jpeg | shrink-image"}),"). In addition to that adding colors can help make the output better readable - and some daredevils even throw in some emojis \ud83e\udd2c. Lastly, consider the good old tab character ",(0,o.jsx)(n.code,{children:"\\t"}),", which may be hard to read (or write) in a text editor."]}),"\n",(0,o.jsxs)(n.p,{children:["Scrut tests live in Markdown or Cram files that are intended to be edited by users. They should not contain binary, non-printable data. To that end, any non-printable output can be denoted in it's hexadecimal escaped form ",(0,o.jsx)(n.code,{children:"\\xAB"})," (with ",(0,o.jsx)(n.code,{children:"AB"})," being the hexadecimal value of the bytecode of the character) or ",(0,o.jsx)(n.code,{children:"\\t"})," to denote tab characters."]}),"\n",(0,o.jsx)(n.p,{children:"The following example shows an expectation of a string that renders as a bold, red font on the command line"}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{children:"Colorful fun\n\n```scrut\n$ echo -e 'Foo \\033[1;31mBar\\033[0m Baz'\nFoo \\x1b[1mBar\\x1b[0m Baz (escaped)\n```\n"})}),"\n",(0,o.jsxs)(n.p,{children:["Or consider some program that prints out two ",(0,o.jsx)(n.code,{children:"\\x00"})," separated strings:"]}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{children:"Colorful fun\n\n```scrut\n$ some-program\nfoo\\x00bar (escaped)\n```\n"})}),"\n",(0,o.jsx)(n.p,{children:"Or again, the good old tab character:"}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{children:"Love the CSV\n\n```scrut\n$ csv-generator\nfoo\\tbar\\tbaz (escaped)\n```\n"})}),"\n",(0,o.jsxs)(n.blockquote,{children:["\n",(0,o.jsxs)(n.p,{children:[(0,o.jsx)(n.strong,{children:"Note"}),": Newlines are ignored for Escaped Expectations. So ",(0,o.jsx)(n.code,{children:"foo\\tbar (escaped)"})," matches both ",(0,o.jsx)(n.code,{children:"foo\\tbar\\n"})," and ",(0,o.jsx)(n.code,{children:"foo\\tbar"}),"."]}),"\n"]}),"\n",(0,o.jsx)(n.h3,{id:"examples-4",children:"Examples"}),"\n",(0,o.jsxs)(n.table,{children:[(0,o.jsx)(n.thead,{children:(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.th,{children:"Expression"}),(0,o.jsx)(n.th,{children:"Meaning"})]})}),(0,o.jsxs)(n.tbody,{children:[(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"Hello\\tWorld (escaped)"})}),(0,o.jsxs)(n.td,{children:["One output line of that starts with ",(0,o.jsx)(n.code,{children:"Hello"}),", followed by a tab character, followed by ",(0,o.jsx)(n.code,{children:"World"})]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"Hello\\tWorld (escaped?)"})}),(0,o.jsxs)(n.td,{children:["An optional output line that contains ",(0,o.jsx)(n.code,{children:"Hello"}),", followed by a tab character, followed by ",(0,o.jsx)(n.code,{children:"World"})]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"Hello\\tWorld (escaped*)"})}),(0,o.jsxs)(n.td,{children:["Any amount (0..n) of output lines that contain ",(0,o.jsx)(n.code,{children:"Hello\\tWorld"}),", followed by a tab character, followed by ",(0,o.jsx)(n.code,{children:"World"})]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"Hello\\tWorld (escaped+)"})}),(0,o.jsxs)(n.td,{children:["One or more (1..n) of output lines that contain ",(0,o.jsx)(n.code,{children:"Hello\\tWorld"}),", followed by a tab character, followed by ",(0,o.jsx)(n.code,{children:"World"})]})]})]})]}),"\n",(0,o.jsxs)(n.blockquote,{children:["\n",(0,o.jsxs)(n.p,{children:[(0,o.jsx)(n.strong,{children:"Note"}),": You can use ",(0,o.jsx)(n.code,{children:"esc"})," as a shorthand for ",(0,o.jsx)(n.code,{children:"escaped"})]}),"\n"]}),"\n",(0,o.jsx)(n.h3,{id:"escaped-glob-expectations",children:"Escaped Glob Expectations"}),"\n",(0,o.jsxs)(n.p,{children:["Because it came up often enough, you can use ",(0,o.jsx)(n.code,{children:"(escaped)"})," in combination with ",(0,o.jsx)(n.code,{children:"(glob)"}),":"]}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{children:"Glob escaped output\n\n```scrut\n$ csv-generator\nfoo\\t* (escaped) (glob+)\nbar\\tbaz (escaped)\n```\n"})}),"\n",(0,o.jsxs)(n.p,{children:["The above exports one or more lines of output that start with ",(0,o.jsx)(n.code,{children:"foo"})," followed by tab. The last line of output is expected to be ",(0,o.jsx)(n.code,{children:"bar"}),", followed by tab, followed by ",(0,o.jsx)(n.code,{children:"baz"}),"."]}),"\n",(0,o.jsxs)(n.table,{children:[(0,o.jsx)(n.thead,{children:(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.th,{children:"Expression"}),(0,o.jsx)(n.th,{children:"Meaning"})]})}),(0,o.jsxs)(n.tbody,{children:[(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"Hello\\tWorld* (escaped) (glob)"})}),(0,o.jsxs)(n.td,{children:["One output line of that starts with ",(0,o.jsx)(n.code,{children:"Hello"}),", followed by a tab character, followed by ",(0,o.jsx)(n.code,{children:"World"}),", followed by anything"]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"Hello\\tWorld* (escaped) (glob?)"})}),(0,o.jsxs)(n.td,{children:["An optional output line that contains ",(0,o.jsx)(n.code,{children:"Hello"}),", followed by a tab character, followed by ",(0,o.jsx)(n.code,{children:"World"}),", followed by anything"]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"Hello\\tWorld* (escaped) (glob*)"})}),(0,o.jsxs)(n.td,{children:["Any amount (0..n) of output lines that contain ",(0,o.jsx)(n.code,{children:"Hello\\tWorld"}),", followed by a tab character, followed by ",(0,o.jsx)(n.code,{children:"World"}),", followed by anything"]})]}),(0,o.jsxs)(n.tr,{children:[(0,o.jsx)(n.td,{children:(0,o.jsx)(n.code,{children:"Hello\\tWorld* (escaped) (glob+)"})}),(0,o.jsxs)(n.td,{children:["One or more (1..n) of output lines that contain ",(0,o.jsx)(n.code,{children:"Hello\\tWorld"}),", followed by a tab character, followed by ",(0,o.jsx)(n.code,{children:"World"}),", followed by anything"]})]})]})]}),"\n",(0,o.jsxs)(n.blockquote,{children:["\n",(0,o.jsxs)(n.p,{children:[(0,o.jsx)(n.strong,{children:"Note"}),": You can use shorthands for either. Quantifiers must be always on ",(0,o.jsx)(n.code,{children:"glob"}),"."]}),"\n"]})]})}function a(e={}){const{wrapper:n}={...(0,t.R)(),...e.components};return n?(0,o.jsx)(n,{...e,children:(0,o.jsx)(h,{...e})}):h(e)}}}]);