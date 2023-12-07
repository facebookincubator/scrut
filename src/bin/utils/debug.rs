/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::path::Path;

use scrut::escaping::Escaper;
use scrut::output::Output;
use scrut::testcase::TestCase;

pub(crate) fn debug_testcases(testcases: &[TestCase], test_file_path: &Path, outputs: &[Output]) {
    for (index, testcase) in testcases.iter().enumerate() {
        eprintln!("~~~~~~~~~~~~~~~~~~~");
        eprintln!("@ {}", test_file_path.display());
        eprintln!("# {}", testcase.title);
        eprintln!("$ {}", &testcase.shell_expression);
        if index < outputs.len() {
            eprintln!("-> {}", outputs[index].exit_code);
            eprintln!("{}", outputs[index].to_error_string(&Escaper::default()));
        } else {
            eprintln!("- no output -");
        }
        eprintln!("~~~~~~~~~~~~~~~~~~~")
    }
}
