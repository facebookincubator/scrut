/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

import React from "react";
import clsx from "clsx";
import Layout from "@theme/Layout";
import Link from "@docusaurus/Link";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import styles from "./index.module.css";
import CodeTeaser from "../components/CodeTeaser";
import Features from "../components/Features";
import GetStarted from "../components/GetStarted";
import Hero from "../components/Hero";
import Intro from "../components/Intro";
import {
  FbInternalOnly,
  OssOnly,
} from "docusaurus-plugin-internaldocs-fb/internal";


export default function Home() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title={`Hello from ${siteConfig.title}`}
      description="An integration and end-to-end testing framework for CLIs."
    >
      <Hero />
      <main>
        <Intro />
        <CodeTeaser />
        <GetStarted />
        <Features />
        <GetStarted />
      </main>
    </Layout>
  );
}
