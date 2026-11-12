//! Registry of the design patterns that the documentation of the code names.

use crate::source::Failure;

/// Object that implements a pattern, as its documentation says.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entry {
    /// Name of the pattern.
    pattern: String,
    /// Name of the object.
    object: String,
    /// File the object lives in.
    file: String,
}

impl Entry {
    /// Records that the object in the file implements the pattern.
    #[must_use]
    pub fn new(
        pattern: impl Into<String>,
        object: impl Into<String>,
        file: impl Into<String>,
    ) -> Self {
        Self {
            pattern: pattern.into(),
            object: object.into(),
            file: file.into(),
        }
    }
}

/// Code whose objects name the patterns they implement.
pub trait Patterns {
    /// Returns every object that names a pattern.
    fn entries(&self) -> Result<Vec<Entry>, Failure>;
}

/// Rust code given as text.
#[derive(Debug, Clone)]
pub struct RustPatterns {
    /// Path shown in the registry.
    file: String,
    /// The code itself.
    text: String,
}

impl RustPatterns {
    /// Wraps code with the path it came from.
    #[must_use]
    pub fn new(file: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            text: text.into(),
        }
    }
}

impl Patterns for RustPatterns {
    fn entries(&self) -> Result<Vec<Entry>, Failure> {
        syn::parse_file(&self.text).map_err(|error| Failure::Parse(self.file.clone(), error))?;
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::{Entry, Patterns, RustPatterns};

    #[test]
    fn lists_rust_items_with_patterns() {
        let code = "/// Decorator: adds a polite ending.
            struct Polite;
            /// Plain greeting.
            struct Plain;
            /// Composition root: wires everything.
            fn main() {}";
        assert_eq!(
            RustPatterns::new("src/main.rs", code).entries().unwrap(),
            vec![
                Entry::new("Decorator", "Polite", "src/main.rs"),
                Entry::new("Composition root", "main", "src/main.rs"),
            ]
        );
    }
}
