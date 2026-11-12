//! Boundary of domain code: no I/O crates and no outer layers.

use syn::visit::{self, Visit};
use syn::{Ident, Path, UseTree};

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

/// Name from an outer layer used by domain code.
#[derive(Debug, Clone)]
struct Intrusion<'a> {
    /// Path of the domain file.
    file: &'a str,
    /// Identifier where the name is used.
    ident: &'a Ident,
    /// The name as the message shows it.
    name: String,
}

impl<'a> Intrusion<'a> {
    /// Points at a name used in a domain file.
    const fn new(file: &'a str, ident: &'a Ident, name: String) -> Self {
        Self { file, ident, name }
    }
}

impl From<Intrusion<'_>> for Finding {
    fn from(intrusion: Intrusion<'_>) -> Self {
        Self::new(
            intrusion.file,
            intrusion.ident.span().start().line,
            "EO010",
            format!(
                "domain code uses `{}`, which belongs to an outer layer",
                intrusion.name
            ),
        )
    }
}

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
        let UseTree::Path(path) = node else {
            return;
        };
        if OUTSIDE.iter().any(|name| path.ident == name) {
            self.findings
                .push(Intrusion::new(&self.file, &path.ident, path.ident.to_string()).into());
        }
        if path.ident == "crate"
            && let UseTree::Path(layer) = path.tree.as_ref()
            && (layer.ident == "api" || layer.ident == "infra")
        {
            self.findings.push(
                Intrusion::new(&self.file, &layer.ident, format!("crate::{}", layer.ident)).into(),
            );
        }
    }

    fn visit_path(&mut self, node: &'ast Path) {
        if let Some(first) = node.segments.first()
            && OUTSIDE.iter().any(|name| first.ident == name)
        {
            self.findings
                .push(Intrusion::new(&self.file, &first.ident, first.ident.to_string()).into());
        }
        visit::visit_path(self, node);
    }
}
