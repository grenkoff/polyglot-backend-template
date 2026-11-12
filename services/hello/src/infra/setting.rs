//! Settings of the service.

/// One setting of the service.
pub trait Setting {
    /// Returns the current value.
    fn value(&self) -> String;
}

/// Setting read from an environment variable.
#[derive(Debug, Clone)]
pub struct Env {
    /// Name of the environment variable.
    variable: String,
    /// Value used when the variable is not set.
    fallback: String,
}

impl Env {
    /// Reads the given variable, with a fallback for local development.
    #[must_use]
    pub fn new(variable: impl Into<String>, fallback: impl Into<String>) -> Self {
        Self {
            variable: variable.into(),
            fallback: fallback.into(),
        }
    }
}

impl Setting for Env {
    fn value(&self) -> String {
        std::env::var(&self.variable).unwrap_or_else(|_| self.fallback.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::{Env, Setting};

    #[test]
    fn falls_back_when_variable_is_missing() {
        assert_eq!(
            Env::new("HELLO_SURELY_MISSING_VARIABLE", "fallback").value(),
            "fallback"
        );
    }

    #[test]
    fn reads_existing_variable() {
        assert_ne!(Env::new("PATH", "").value(), "");
    }
}
