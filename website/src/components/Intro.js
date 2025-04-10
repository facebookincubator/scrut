/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

import React from 'react';
import clsx from 'clsx';
import styles from './Intro.module.css';


export default function CodeTeaser() {
    return (
        <div className={clsx(["container", styles.intro])}>
            <p>
                Scrut is an integration / end-to-end <strong>testing framework for CLIs</strong> with a concise syntax that makes <strong>writing and maintaining tests easy</strong>.
            </p>
        </div>
    );
  }
