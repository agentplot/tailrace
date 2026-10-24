//! The coverage check over this tree: every clause of Parts A and B cited
//! by a scenario or named in the register. It prints every registered
//! clause as pending, the counts, and every finding, and exits 1 on any
//! finding (section 12, rule core.33; design D8).

use std::process::ExitCode;

use tailrace_conformance::{citations, coverage, register, statement};

fn main() -> ExitCode {
    let coverage = coverage::check(statement::PARTS, &citations(), register::PENDING);
    println!("{coverage}");
    println!("read at {}", statement::READ_AT);
    if coverage.passed() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
