//! Violations found by eo-lint.

use std::fmt;

/// Rule violation at a line of a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Path of the file.
    file: String,
    /// Line of the violation.
    line: usize,
    /// Rule code such as `EO001`.
    rule: &'static str,
    /// What is wrong and how to fix it.
    message: String,
}

impl Finding {
    /// Creates a finding.
    #[must_use]
    pub fn new(
        file: impl Into<String>,
        line: usize,
        rule: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            file: file.into(),
            line,
            rule,
            message: message.into(),
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{}: {} {}",
            self.file, self.line, self.rule, self.message
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Finding;

    #[test]
    fn prints_like_a_compiler() {
        assert_eq!(
            Finding::new("src/lib.rs", 7, "EO001", "bad name").to_string(),
            "src/lib.rs:7: EO001 bad name"
        );
    }
}
