use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    file: String,
    line: usize,
    rule: &'static str,
    message: String,
}

impl Finding {
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
