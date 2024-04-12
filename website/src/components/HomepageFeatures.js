/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

import React from 'react';
import clsx from 'clsx';
import styles from './HomepageFeatures.module.css';

const FeatureList = [
  {
    title: 'Easy to Use',
    Svg: require('../../static/img/undraw_docusaurus_mountain.svg').default,
    description: (
      <>
        Scrut was designed to be simple and straightforward.
        If you know how to execute your CLI on a shell and you know how to write Markdown then
        you already know how to write tests for your CLI in Scrut.
      </>
    ),
  },
  {
    title: 'Any size fits',
    Svg: require('../../static/img/undraw_docusaurus_tree.svg').default,
    description: (
      <>
        Scrut is simple, yet powerful enough to handle any size CLI. From a simple
        bash script to a complex Java / Rust / C++ / ... binary with many dependencies.
        Scrut can handle it all.
      </>
    ),
  },
  {
    title: 'Maintenance is Life',
    Svg: require('../../static/img/undraw_docusaurus_react.svg').default,
    description: (
      <>
        Do your future self (and any other future maintainer) a big favor and document
        the intended behavior of your CLI in the form of test-cases in easily readable
        Markdown test-files.
      </>
    ),
  },
];

function Feature({Svg, title, description}) {
  return (
    <div className={clsx('col col--4')}>
      {/* <div className="text--center">
        <Svg className={styles.featureSvg} alt={title} />
      </div> */}
      <div className="text--center padding-horiz--md">
        <h3>{title}</h3>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures() {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
