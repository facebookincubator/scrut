/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

import React from 'react';
import clsx from 'clsx';
import styles from './GetStarted.module.css';


export default function GetStarted() {
    return (
        <div className={clsx(["container", styles.getStarted])}>
            <a href="/scrut/docs/getting-started/" className={styles.getStartedButton}>
                Get Started
            </a>
        </div>
    );
  }
