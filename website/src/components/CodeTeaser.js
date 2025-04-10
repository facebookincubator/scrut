/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

import React from 'react';
import clsx from 'clsx';
import styles from './CodeTeaser.module.css';
import CodeBlock from '@theme/CodeBlock';

const codeExample = `# Scrut tests are Markdown files

Documented tests for the win.

## Hello World

\`\`\`scrut
$ echo "Hello World"
Hello World
\`\`\`

## Failing World

\`\`\`scrut
$ echo "Failing World"
Hello World
\`\`\`
`;

const scrutExample = `$ scrut test /tmp/test.md
🔎 Found 1 test document(s)
❌ /tmp/test.md: 1 out of 2 tests failed

// =========================================================
// @ /tmp/test.md:15
// ---------------------------------------------------------
// # Failing World
// ---------------------------------------------------------
// $ echo "Failing World"
// =========================================================

1     | - Hello World
   1  | + Failing World


Result: 1 document(s) with 2 testcase(s): 1 succeeded, 1 failed and 0 skipped
`;

export default function CodeTeaser() {
    return (
        <div className={styles.codeTeaser}>
            <div className={styles.codeTeaserInner}>
                <div className="row">
                    <div className={clsx('col col--5')}>
                        <h2>Write a test</h2>
                        <CodeBlock language="markdown" title="tests.md" showLineNumbers>{codeExample}</CodeBlock>
                    </div>
                    <div className={clsx('col col--7')}>
                        <h2>Run a test</h2>
                        <CodeBlock language="bash" title="Terminal" showLineNumbers>{scrutExample}</CodeBlock>
                    </div>
                </div>
            </div>
        </div>
    );
  }
