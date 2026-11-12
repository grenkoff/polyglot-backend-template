//! Boundary of domain code: no I/O crates and no outer layers.

use syn::visit::{self, Visit};
use syn::{Path, UseTree};

use crate::finding::Finding;

/// Crates that do I/O or describe the wire format.
const OUTSIDE: &[&str] = &[
    "tokio",
    "tonic",
    "axum",
    "sqlx",
    "async_nats",
    "prost",
    "contracts",
];

/// Visitor that collects what domain code takes from outer layers.
#[derive(Debug)]
pub struct Isolation {
    /// Path of the inspected file.
    file: String,
    /// Violations found so far.
    findings: Vec<Finding>,
}

impl Isolation {
    /// Starts the inspection of one domain file.
    #[must_use]
    pub fn new(file: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            findings: Vec::new(),
        }
    }
}

impl From<Isolation> for Vec<Finding> {
    fn from(isolation: Isolation) -> Self {
        isolation.findings
    }
}

impl<'ast> Visit<'ast> for Isolation {
    fn visit_use_tree(&mut self, node: &'ast UseTree) {
        if let UseTree::Path(path) = node
            && OUTSIDE.iter().any(|name| path.ident == name)
        {
            self.findings.push(Finding::new(
                &self.file,
                path.ident.span().start().line,
                "EO010",
                format!(
                    "domain code uses `{}`, keep I/O in outer layers",
                    path.ident
                ),
            ));
        }
    }

    fn visit_path(&mut self, node: &'ast Path) {
        if let Some(first) = node.segments.first()
            && OUTSIDE.iter().any(|name| first.ident == name)
        {
            self.findings.push(Finding::new(
                &self.file,
                first.ident.span().start().line,
                "EO010",
                format!(
                    "domain code uses `{}`, keep I/O in outer layers",
                    first.ident
                ),
            ));
        }
        visit::visit_path(self, node);
    }
}
