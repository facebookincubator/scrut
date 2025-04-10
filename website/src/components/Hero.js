/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

import React from 'react';
import clsx from 'clsx';
import styles from './Hero.module.css';
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";

export default function Hero() {
    const { siteConfig } = useDocusaurusContext();
    return (
        <header
        className={clsx("hero hero--primary hero--image", styles.heroImage)}
        >
            <h1 className="hero__title">{siteConfig.title}</h1>
            <h2 className="hero__subtitle">{siteConfig.tagline}</h2>
        </header>
    );
}
