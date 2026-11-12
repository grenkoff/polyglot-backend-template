//! Code given as text and the languages it is written in.

use crate::finding::Finding;
use crate::registry::{Documented, Entry, Patterns};
use crate::source::{Failure, Linted, Source};

/// Strategy: Rust, the language of the services and tools.
#[derive(Debug, Clone, Copy)]
pub struct Rust;

/// Strategy: Python, the language of core.
#[derive(Debug, Clone, Copy)]
pub struct Python;

/// Code in one language given as text.
#[derive(Debug, Clone)]
pub struct Code<L> {
    /// Path shown in findings and in the registry.
    file: String,
    /// The code itself.
    text: String,
    /// Language the code is written in.
    language: L,
}

impl<L> Code<L> {
    /// Wraps code with the path it came from and its language.
    #[must_use]
    pub fn new(file: impl Into<String>, text: impl Into<String>, language: L) -> Self {
        Self {
            file: file.into(),
            text: text.into(),
            language,
        }
    }
}

impl<L: Linted> Source for Code<L> {
    fn findings(&self) -> Result<Vec<Finding>, Failure> {
        self.language.findings(&self.file, &self.text)
    }
}

impl<L: Documented> Patterns for Code<L> {
    fn entries(&self) -> Result<Vec<Entry>, Failure> {
        self.language.entries(&self.file, &self.text)
    }
}
