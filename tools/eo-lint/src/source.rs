use std::path::{Path, PathBuf};

use syn::visit::Visit;
use walkdir::WalkDir;

use crate::finding::Finding;
use crate::inspection::Inspection;

const SKIPPED: &[&str] = &[
    "target",
    ".git",
    ".venv",
    "node_modules",
    "tests",
    "benches",
];

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error("cannot read {0}: {1}")]
    Read(String, std::io::Error),
    #[error("cannot parse {0}: {1}")]
    Parse(String, syn::Error),
    #[error("cannot walk directories: {0}")]
    Walk(#[from] walkdir::Error),
}

pub trait Source {
    fn findings(&self) -> Result<Vec<Finding>, Failure>;
}

#[derive(Debug, Clone)]
pub struct Code {
    file: String,
    text: String,
}

impl Code {
    #[must_use]
    pub fn new(file: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            text: text.into(),
        }
    }
}

impl Source for Code {
    fn findings(&self) -> Result<Vec<Finding>, Failure> {
        let tree = syn::parse_file(&self.text)
            .map_err(|error| Failure::Parse(self.file.clone(), error))?;
        let entry = self.file.ends_with("main.rs") || self.file.ends_with("build.rs");
        let mut inspection = Inspection::new(&self.file, entry);
        inspection.visit_file(&tree);
        Ok(inspection.into())
    }
}

#[derive(Debug, Clone)]
pub struct Disk {
    path: PathBuf,
}

impl Disk {
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
        Code::new(file, text).findings()
    }
}

#[derive(Debug, Clone)]
pub struct Tree {
    root: PathBuf,
}

impl Tree {
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
    use super::{Code, Failure, Source, Tree};

    #[test]
    fn reports_broken_code() {
        assert!(matches!(
            Code::new("lib.rs", "fn (").findings(),
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
