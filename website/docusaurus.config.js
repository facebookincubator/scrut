/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

const lightCodeTheme = require('prism-react-renderer').themes.github;
const darkCodeTheme = require('prism-react-renderer').themes.dracula;

const {customFields} = require('./constants');

const { organizationName, baseUrl } =
  "GITHUB_REPOSITORY" in process.env
    ? (() => {
        const parts = process.env.GITHUB_REPOSITORY.split("/");
        return { organizationName: parts[0], baseUrl: `/${parts[1]}/` };
      })()
    : { organizationName: "facebook", baseUrl: "/" };

// With JSDoc @type annotations, IDEs can provide config autocompletion
/** @type {import('@docusaurus/types').DocusaurusConfig} */
(module.exports = {
  title: 'Scrut',
  tagline: 'A CLI Testing Framework',
  url: 'https://internalfb.com',
  baseUrl,
  onBrokenLinks: 'warn',
  onBrokenMarkdownLinks: 'warn',
  trailingSlash: true,
  favicon: 'img/favicon.ico',
  organizationName,
  projectName: 'scrut',
  markdown: {
    mermaid: true,
  },
  themes: ['@docusaurus/theme-mermaid'],
  customFields,
  staticDirectories: ['static'],


  presets: [
    [
      'docusaurus-plugin-internaldocs-fb/docusaurus-preset',
      /** @type {import('docusaurus-plugin-internaldocs-fb').PresetOptions} */
      ({
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          editUrl: 'https://www.internalfb.com/code/fbsource/fbcode/clifoundation/scrut/website', // TODO Please change this to your repo.
        },
        experimentalXRepoSnippets: {
          baseDir: '.',
        },
        staticDocsProject: 'Scrut',
        trackingFile: 'fbcode/staticdocs/WATCHED_FILES',
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      navbar: {
        title: 'Scrut',
        logo: {
          alt: 'Scrut Logo',
          src: 'img/logo.svg',
        },
        items: [
          {
            type: 'doc',
            docId: 'README',
            position: 'left',
            label: 'Docs',
          },
          /* {to: '/blog', label: 'Blog', position: 'left'}, */
          {
            href: 'https://github.com/facebookincubator/scrut',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Legal',
            items: [
              {
                label: 'Terms of Use',
                to: 'https://opensource.fb.com/legal/terms',
              },
              {
                label: 'Privacy Policy',
                to: 'https://opensource.fb.com/legal/privacy',
              },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} Meta Platforms, Inc`,
      },
      prism: {
        theme: lightCodeTheme,
        darkTheme: darkCodeTheme,
      },
    }),
});
