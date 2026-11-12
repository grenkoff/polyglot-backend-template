//! Registry of the design patterns that the documentation of the code names.

use syn::visit::Visit;
use syn::{Attribute, Expr, ExprLit, ItemFn, ItemStruct, Lit, Meta};

use std::path::Path;

use crate::source::Failure;

/// Quotes that open and close a Python docstring.
const QUOTES: &str = r#"""""#;

/// Names of the patterns that documentation may start with.
const PATTERNS: &[&str] = &[
    "Decorator",
    "Strategy",
    "Adapter",
    "Facade",
    "Factory",
    "Observer",
    "State",
    "Command",
    "Composition root",
];

/// First line of a documentation, which may start with a pattern name.
#[derive(Debug, Clone, Copy)]
struct Summary<'a> {
    /// The line itself.
    line: &'a str,
}

impl<'a> Summary<'a> {
    /// Wraps the first line of a documentation.
    const fn new(line: &'a str) -> Self {
        Self { line }
    }

    /// Returns the pattern the line names, if it names one.
    fn pattern(&self) -> Option<String> {
        let (prefix, _) = self.line.trim().split_once(": ")?;
        PATTERNS.contains(&prefix).then(|| prefix.to_owned())
    }
}

/// Line of Python code that may be a one-line docstring.
#[derive(Debug, Clone, Copy)]
struct Docstring<'a> {
    /// The line itself.
    line: &'a str,
}

impl<'a> Docstring<'a> {
    /// Wraps a line of Python code.
    const fn new(line: &'a str) -> Self {
        Self { line }
    }

    /// Returns the pattern the docstring names, if the line is one that names it.
    fn pattern(&self) -> Option<String> {
        let text = self.line.strip_prefix(QUOTES)?.strip_suffix(QUOTES)?;
        Summary::new(text).pattern()
    }
}

/// Documentation of a Rust item.
#[derive(Debug, Clone, Copy)]
struct Documentation<'a> {
    /// Attributes of the item.
    attrs: &'a [Attribute],
}

impl<'a> Documentation<'a> {
    /// Wraps the attributes of an item.
    const fn new(attrs: &'a [Attribute]) -> Self {
        Self { attrs }
    }

    /// Returns the pattern the documentation names, if it names one.
    fn pattern(&self) -> Option<String> {
        let first = self.attrs.iter().find_map(|attr| match &attr.meta {
            Meta::NameValue(pair) if pair.path.is_ident("doc") => match &pair.value {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(text),
                    ..
                }) => Some(text.value()),
                _ => None,
            },
            _ => None,
        })?;
        Summary::new(&first).pattern()
    }
}

/// Visitor that collects the items whose documentation names a pattern.
#[derive(Debug)]
struct Catalogue<'a> {
    /// Path shown in the registry.
    file: &'a str,
    /// Entries found so far.
    entries: Vec<Entry>,
}

impl<'ast> Visit<'ast> for Catalogue<'_> {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        if let Some(pattern) = Documentation::new(&node.attrs).pattern() {
            self.entries
                .push(Entry::new(pattern, node.ident.to_string(), self.file));
        }
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if let Some(pattern) = Documentation::new(&node.attrs).pattern() {
            self.entries
                .push(Entry::new(pattern, node.sig.ident.to_string(), self.file));
        }
    }
}

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
        let tree = syn::parse_file(&self.text)
            .map_err(|error| Failure::Parse(self.file.clone(), error))?;
        let mut catalogue = Catalogue {
            file: &self.file,
            entries: Vec::new(),
        };
        catalogue.visit_file(&tree);
        Ok(catalogue.entries)
    }
}

/// Python code given as text.
#[derive(Debug, Clone)]
pub struct PythonPatterns {
    /// Path shown in the registry.
    file: String,
    /// The code itself.
    text: String,
}

impl PythonPatterns {
    /// Wraps code with the path it came from.
    #[must_use]
    pub fn new(file: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            text: text.into(),
        }
    }
}

impl Patterns for PythonPatterns {
    fn entries(&self) -> Result<Vec<Entry>, Failure> {
        let lines: Vec<&str> = self
            .text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect();
        let module = Path::new(&self.file)
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        let heading = lines
            .first()
            .and_then(|line| Docstring::new(line).pattern())
            .map(|pattern| Entry::new(pattern, module, &self.file));
        let classes = lines.windows(2).filter_map(|pair| {
            let name = pair[0].strip_prefix("class ")?.split(['(', ':']).next()?;
            Docstring::new(pair[1])
                .pattern()
                .map(|pattern| Entry::new(pattern, name, &self.file))
        });
        Ok(heading.into_iter().chain(classes).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::{Entry, Patterns, PythonPatterns, RustPatterns};

    #[test]
    fn lists_python_classes_and_modules_with_patterns() {
        let code = r#""""Composition root: binds every protocol."""

import attrs


@final
class Remote:
    """Adapter: asks the hello service over gRPC."""


class Plain:
    """Greets without a pattern."""
"#;
        assert_eq!(
            PythonPatterns::new("server/implemented.py", code)
                .entries()
                .unwrap(),
            vec![
                Entry::new("Composition root", "implemented", "server/implemented.py"),
                Entry::new("Adapter", "Remote", "server/implemented.py"),
            ]
        );
    }

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

    #[test]
    fn rejects_unknown_pattern_names() {
        let rust = "/// Request for a greeting: a name and a flag.
            struct Request;
            /// Visitor: walks the tree.
            struct Walk;";
        let python = "class Walk:\n    \"\"\"Visitor: walks the tree.\"\"\"\n";
        assert_eq!(
            RustPatterns::new("src/walk.rs", rust)
                .entries()
                .map_err(|error| error.to_string()),
            Err(String::from(
                "src/walk.rs:3: `Visitor` is not a known pattern"
            ))
        );
        assert_eq!(
            PythonPatterns::new("walk.py", python)
                .entries()
                .map_err(|error| error.to_string()),
            Err(String::from("walk.py:2: `Visitor` is not a known pattern"))
        );
    }
}
