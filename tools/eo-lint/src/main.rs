//! Command line entry of eo-lint.

use std::io::Write;
use std::process::ExitCode;

use eo_lint::source::{Source, Tree};

/// Inspects the given directories and reports every violation.
fn main() -> ExitCode {
    let mut roots: Vec<String> = std::env::args().skip(1).collect();
    if roots.is_empty() {
        roots.push(String::from("."));
    }
    let mut out = std::io::stdout().lock();
    let mut total = 0_usize;
    for root in roots {
        match Tree::new(&root).findings() {
            Ok(findings) => {
                for finding in &findings {
                    if writeln!(out, "{finding}").is_err() {
                        return ExitCode::FAILURE;
                    }
                }
                total += findings.len();
            }
            Err(error) => {
                let _ = writeln!(out, "eo-lint: {error}");
                return ExitCode::from(2);
            }
        }
    }
    if total == 0 {
        ExitCode::SUCCESS
    } else {
        let _ = writeln!(
            out,
            "eo-lint: {total} violation(s) of Elegant Objects rules"
        );
        ExitCode::FAILURE
    }
}
