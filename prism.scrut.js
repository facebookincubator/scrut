/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

const Prism = require('prism-react-renderer').Prism;

Prism.languages.scrut = {
    'special-line': {
      pattern: /^([>$]).*$/m,
      lookbehind: true,
      alias: 'keyword',
      bold: true
    }
};
