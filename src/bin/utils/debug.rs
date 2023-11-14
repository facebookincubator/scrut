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
