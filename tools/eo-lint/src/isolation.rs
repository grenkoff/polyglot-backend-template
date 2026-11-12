//! Boundary of domain code: no I/O crates and no outer layers.

use syn::visit::{self, Visit};
use syn::{Ident, ItemUse, Path, UseTree};

use crate::finding::Finding;

/// Parts of an import tree.
trait Imports {
    /// Returns the trees that start a name, with groups opened.
    fn roots(&self) -> Vec<&UseTree>;
    /// Returns the first name of the tree, if it starts with one.
    fn head(&self) -> Option<&Ident>;
}

impl Imports for UseTree {
    fn roots(&self) -> Vec<&UseTree> {
        match self {
            Self::Group(group) => group.items.iter().flat_map(Imports::roots).collect(),
            Self::Path(_) | Self::Name(_) | Self::Rename(_) | Self::Glob(_) => vec![self],
        }
    }

    fn head(&self) -> Option<&Ident> {
        match self {
            Self::Path(path) => Some(&path.ident),
            Self::Name(name) => Some(&name.ident),
            Self::Rename(rename) => Some(&rename.ident),
            Self::Group(_) | Self::Glob(_) => None,
        }
    }
}

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
    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        for root in node.tree.roots() {
            let intrusions: Vec<(&Ident, String)> = match root {
                UseTree::Path(path) if path.ident == "crate" => path
                    .tree
                    .roots()
                    .into_iter()
                    .filter_map(Imports::head)
                    .filter(|layer| *layer == "api" || *layer == "infra")
                    .map(|layer| (layer, format!("crate::{layer}")))
                    .collect(),
                other => other
                    .head()
                    .filter(|name| OUTSIDE.iter().any(|outside| name == outside))
                    .map(|name| (name, name.to_string()))
                    .into_iter()
                    .collect(),
            };
            self.findings.extend(
                intrusions
                    .into_iter()
                    .map(|(ident, name)| Finding::from(Intrusion::new(&self.file, ident, name))),
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
