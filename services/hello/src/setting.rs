pub trait Setting {
    fn value(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct Env {
    variable: String,
    fallback: String,
}

impl Env {
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
