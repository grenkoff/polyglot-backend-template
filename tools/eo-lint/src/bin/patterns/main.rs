//! Command line entry that prints the registry of design patterns.

use std::io::Write;
use std::process::ExitCode;

use eo_lint::registry::{PatternTree, Patterns, Table};

/// Composition root: prints the patterns named under the given directories as Markdown.
fn main() -> ExitCode {
    let mut entries = Vec::new();
    for root in std::env::args().skip(1) {
        match PatternTree::new(&root).entries() {
            Ok(found) => entries.extend(found),
            Err(error) => {
                let _ = writeln!(std::io::stderr(), "patterns: {error}");
                return ExitCode::FAILURE;
            }
        }
    }
    entries.sort();
    if write!(std::io::stdout().lock(), "{}", Table::new(entries)).is_err() {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
