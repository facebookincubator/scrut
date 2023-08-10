/// Command Line Application Testing - with no fuzz
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate lazy_static;

pub mod debug;
pub mod diff;
pub mod escaping;
pub mod executors;
pub mod expectation;
pub mod generators;
pub mod newline;
pub mod outcome;
pub mod output;
pub mod parsers;
pub mod renderers;
pub mod rules;
pub mod testcase;
