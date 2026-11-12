//! Registry of the design patterns that the documentation of the code names.

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Attribute, Expr, ExprLit, ItemFn, ItemStruct, Lit, Meta};
use walkdir::WalkDir;

use std::path::{Path, PathBuf};

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

    /// Returns the name the line starts with, when it is written like a pattern name.
    fn name(&self) -> Option<&'a str> {
        let (prefix, _) = self.line.trim().split_once(": ")?;
        let mut words = prefix.split(' ');
        let first = words.next()?;
        let rest: Vec<&str> = words.collect();
        let capitalized = first.chars().next().is_some_and(char::is_uppercase)
            && first.chars().all(char::is_alphabetic);
        let lowercase = rest.iter().all(|word| word.chars().all(char::is_lowercase));
        (capitalized && lowercase && rest.len() < 2).then_some(prefix)
    }
}

/// Object whose documentation starts with something written like a pattern name.
#[derive(Debug, Clone)]
struct Claim {
    /// The name the documentation starts with.
    pattern: String,
    /// Name of the object.
    object: String,
    /// File the object lives in.
    file: String,
    /// Line of the documentation.
    line: usize,
}

impl TryFrom<Claim> for Entry {
    type Error = Failure;

    fn try_from(claim: Claim) -> Result<Self, Failure> {
        if PATTERNS.contains(&claim.pattern.as_str()) {
            Ok(Self::new(claim.pattern, claim.object, claim.file))
        } else {
            Err(Failure::Unknown(claim.file, claim.line, claim.pattern))
        }
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

    /// Returns the name the docstring starts with, when it is written like a pattern.
    fn name(&self) -> Option<&'a str> {
        let text = self.line.strip_prefix(QUOTES)?.strip_suffix(QUOTES)?;
        Summary::new(text).name()
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

    /// Returns the name the documentation starts with and its line, if written like a pattern.
    fn name(&self) -> Option<(String, usize)> {
        let (first, line) = self.attrs.iter().find_map(|attr| match &attr.meta {
            Meta::NameValue(pair) if pair.path.is_ident("doc") => match &pair.value {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(text),
                    ..
                }) => Some((text.value(), attr.span().start().line)),
                _ => None,
            },
            _ => None,
        })?;
        Summary::new(&first)
            .name()
            .map(|name| (name.to_owned(), line))
    }
}

/// Visitor that collects the items whose documentation starts like a pattern name.
#[derive(Debug)]
struct Catalogue<'a> {
    /// Path shown in the registry.
    file: &'a str,
    /// Claims found so far.
    claims: Vec<Claim>,
}

impl<'ast> Visit<'ast> for Catalogue<'_> {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        if let Some((pattern, line)) = Documentation::new(&node.attrs).name() {
            self.claims.push(Claim {
                pattern,
                object: node.ident.to_string(),
                file: self.file.to_owned(),
                line,
            });
        }
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if let Some((pattern, line)) = Documentation::new(&node.attrs).name() {
            self.claims.push(Claim {
                pattern,
                object: node.sig.ident.to_string(),
                file: self.file.to_owned(),
                line,
            });
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
            claims: Vec::new(),
        };
        catalogue.visit_file(&tree);
        catalogue.claims.into_iter().map(Entry::try_from).collect()
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
        let lines: Vec<(usize, &str)> = self
            .text
            .lines()
            .map(str::trim)
            .enumerate()
            .filter(|(_, line)| !line.is_empty())
            .collect();
        let module = Path::new(&self.file)
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        let heading = lines.first().and_then(|&(index, line)| {
            Docstring::new(line).name().map(|pattern| Claim {
                pattern: pattern.to_owned(),
                object: module,
                file: self.file.clone(),
                line: index + 1,
            })
        });
        let classes = lines.windows(2).filter_map(|pair| {
            let name = pair[0].1.strip_prefix("class ")?.split(['(', ':']).next()?;
            Docstring::new(pair[1].1).name().map(|pattern| Claim {
                pattern: pattern.to_owned(),
                object: name.to_owned(),
                file: self.file.clone(),
                line: pair[1].0 + 1,
            })
        });
        heading
            .into_iter()
            .chain(classes)
            .map(Entry::try_from)
            .collect()
    }
}

/// Rust and Python code under a directory.
#[derive(Debug, Clone)]
pub struct PatternTree {
    /// Root directory to walk.
    root: PathBuf,
}

impl PatternTree {
    /// Points at a root directory.
    #[must_use]
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }
}

impl Patterns for PatternTree {
    fn entries(&self) -> Result<Vec<Entry>, Failure> {
        let mut all = Vec::new();
        for entry in WalkDir::new(&self.root) {
            let entry = entry?;
            let path = entry.path();
            let file = path.display().to_string();
            let text = || std::fs::read_to_string(path).map_err(|e| Failure::Read(file.clone(), e));
            match path.extension().and_then(|extension| extension.to_str()) {
                Some("rs") => all.extend(RustPatterns::new(&file, text()?).entries()?),
                Some("py") => all.extend(PythonPatterns::new(&file, text()?).entries()?),
                _ => {}
            }
        }
        all.sort();
        Ok(all)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{Entry, PatternTree, Patterns, PythonPatterns, RustPatterns};

    fn folder(name: &str, files: &[(&str, &str)]) -> PathBuf {
        let root = std::env::temp_dir().join(format!("eo-registry-{}-{name}", std::process::id()));
        for (path, text) in files {
            let file = root.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, text).unwrap();
        }
        root
    }

    #[test]
    fn sorts_rows_by_pattern_and_object() {
        let root = folder(
            "sorted",
            &[
                ("b.rs", "/// Decorator: wraps.\nstruct Zeta;"),
                ("a.py", "class Alpha:\n    \"\"\"Adapter: adapts.\"\"\"\n"),
                ("c.rs", "/// Decorator: wraps.\nstruct Beta;"),
            ],
        );

        let entries = PatternTree::new(&root).entries().unwrap();

        std::fs::remove_dir_all(&root).unwrap();
        let file = |name: &str| root.join(name).display().to_string();
        assert_eq!(
            entries,
            vec![
                Entry::new("Adapter", "Alpha", file("a.py")),
                Entry::new("Decorator", "Beta", file("c.rs")),
                Entry::new("Decorator", "Zeta", file("b.rs")),
            ]
        );
    }

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
