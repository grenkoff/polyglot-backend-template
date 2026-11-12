//! Rust sources to inspect: text, a file or a directory tree.

use std::path::{Path, PathBuf};

use syn::visit::Visit;
use walkdir::WalkDir;

use crate::code::{Code, Rust};
use crate::finding::Finding;
use crate::inspection::Inspection;
use crate::isolation::Isolation;
use crate::transaction::Transactions;

/// Directories that hold no hand-written code to inspect.
const SKIPPED: &[&str] = &[
    "target",
    ".git",
    ".venv",
    "node_modules",
    "tests",
    "benches",
];

/// Why sources could not be inspected.
#[derive(Debug, thiserror::Error)]
pub enum Failure {
    /// A file could not be read.
    #[error("cannot read {0}: {1}")]
    Read(String, std::io::Error),
    /// A file is not valid Rust.
    #[error("cannot parse {0}: {1}")]
    Parse(String, syn::Error),
    /// A directory could not be walked.
    #[error("cannot walk directories: {0}")]
    Walk(#[from] walkdir::Error),
    /// A documentation names a pattern outside the allowed list.
    #[error("{0}:{1}: `{2}` is not a known pattern")]
    Unknown(String, usize, String),
}

/// Rust code that can be inspected.
pub trait Source {
    /// Returns the violations found in the code.
    fn findings(&self) -> Result<Vec<Finding>, Failure>;
}

/// Language whose code eo-lint inspects.
pub trait Linted {
    /// Returns the violations found in the code of the given file.
    fn findings(&self, file: &str, text: &str) -> Result<Vec<Finding>, Failure>;
}

impl Linted for Rust {
    fn findings(&self, file: &str, text: &str) -> Result<Vec<Finding>, Failure> {
        let tree = syn::parse_file(text).map_err(|error| Failure::Parse(file.to_owned(), error))?;
        let entry = file.ends_with("main.rs") || file.ends_with("build.rs");
        let mut inspection = Inspection::new(file, entry);
        inspection.visit_file(&tree);
        let mut findings = Vec::from(inspection);
        if Path::new(file)
            .ancestors()
            .any(|folder| folder.ends_with("src/domain"))
        {
            let mut isolation = Isolation::new(file);
            isolation.visit_file(&tree);
            findings.extend(Vec::from(isolation));
        }
        let mut transactions = Transactions::new(file);
        transactions.visit_file(&tree);
        findings.extend(Vec::from(transactions));
        Ok(findings)
    }
}

/// Rust file on disk.
#[derive(Debug, Clone)]
pub struct Disk {
    /// Path to the file.
    path: PathBuf,
}

impl Disk {
    /// Points at a file.
    #[must_use]
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl Source for Disk {
    fn findings(&self) -> Result<Vec<Finding>, Failure> {
        let file = self.path.display().to_string();
        let text = std::fs::read_to_string(&self.path)
            .map_err(|error| Failure::Read(file.clone(), error))?;
        Code::new(file, text, Rust).findings()
    }
}

/// Directory tree of Rust files.
#[derive(Debug, Clone)]
pub struct Tree {
    /// Root directory to walk.
    root: PathBuf,
}

impl Tree {
    /// Points at a root directory.
    #[must_use]
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }
}

impl Source for Tree {
    fn findings(&self) -> Result<Vec<Finding>, Failure> {
        let mut all = Vec::new();
        let entries = WalkDir::new(&self.root)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|entry| {
                entry.depth() == 0
                    || !SKIPPED.contains(&entry.file_name().to_string_lossy().as_ref())
            });
        for entry in entries {
            let entry = entry?;
            if entry.file_type().is_file()
                && entry.path().extension().is_some_and(|ext| ext == "rs")
            {
                all.extend(Disk::new(entry.path()).findings()?);
            }
        }
        Ok(all)
    }
}

#[cfg(test)]
mod tests {
    use super::{Failure, Source, Tree};
    use crate::code::{Code, Rust};

    #[test]
    fn reports_broken_code() {
        assert!(matches!(
            Code::new("lib.rs", "fn (", Rust).findings(),
            Err(Failure::Parse(..))
        ));
    }

    #[test]
    fn reports_missing_files() {
        assert!(matches!(
            super::Disk::new("/surely/missing/file.rs").findings(),
            Err(Failure::Read(..))
        ));
    }

    #[test]
    fn reports_missing_directories() {
        assert!(matches!(
            Tree::new("/surely/missing/directory").findings(),
            Err(Failure::Walk(..))
        ));
    }

    #[test]
    fn skips_build_directories_but_not_the_root() {
        let base = std::env::temp_dir().join(format!("eo-lint-{}", std::process::id()));
        let root = base.join("tests");
        std::fs::create_dir_all(root.join("target")).unwrap();
        std::fs::write(root.join("lib.rs"), "fn helper() {}").unwrap();
        std::fs::write(
            root.join("target").join("generated.rs"),
            "fn generated() {}",
        )
        .unwrap();

        let findings = Tree::new(&root).findings().unwrap();

        std::fs::remove_dir_all(&base).unwrap();
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn walks_its_own_sources() {
        let findings = Tree::new(env!("CARGO_MANIFEST_DIR")).findings().unwrap();
        assert_eq!(findings, Vec::new());
    }
}
