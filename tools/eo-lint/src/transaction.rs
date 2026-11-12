//! Database transactions that wait for nothing but themselves.

use syn::visit::{self, Visit};
use syn::{Block, ExprAwait, Ident, Local, LocalInit, Pat, Stmt};

use crate::finding::Finding;

/// Visitor that tells whether code mentions a name.
#[derive(Debug)]
struct Trace<'a> {
    /// The name to look for.
    name: &'a str,
    /// Whether the name was seen.
    seen: bool,
}

impl<'a> Trace<'a> {
    /// Starts looking for the name.
    const fn new(name: &'a str) -> Self {
        Self { name, seen: false }
    }
}

impl<'ast> Visit<'ast> for Trace<'_> {
    fn visit_ident(&mut self, ident: &'ast Ident) {
        self.seen |= ident == self.name;
    }
}

impl From<Trace<'_>> for bool {
    fn from(trace: Trace<'_>) -> Self {
        trace.seen
    }
}

/// Questions about one statement of a block.
trait Mentions {
    /// Returns true when the statement mentions the name.
    fn mentions(&self, name: &str) -> bool;
    /// Returns true when the statement binds a new transaction.
    fn opens(&self) -> bool;
}

impl Mentions for Stmt {
    fn mentions(&self, name: &str) -> bool {
        let mut trace = Trace::new(name);
        trace.visit_stmt(self);
        trace.into()
    }

    fn opens(&self) -> bool {
        matches!(self, Self::Local(Local {
            pat: Pat::Ident(_),
            init: Some(LocalInit { .. }),
            ..
        }) if self.mentions("begin"))
    }
}

/// Visitor that collects every await of a statement inside a transaction.
#[derive(Debug)]
struct Waits<'a> {
    /// Path of the inspected file.
    file: &'a str,
    /// Violations found so far.
    findings: Vec<Finding>,
}

impl<'a> Waits<'a> {
    /// Starts collecting awaits of one file.
    const fn new(file: &'a str) -> Self {
        Self {
            file,
            findings: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for Waits<'_> {
    fn visit_expr_await(&mut self, node: &'ast ExprAwait) {
        self.findings.push(Finding::new(
            self.file,
            node.await_token.span.start().line,
            "EO011",
            "awaits something else inside a transaction, do I/O before or after it",
        ));
        visit::visit_expr_await(self, node);
    }
}

/// Visitor that collects awaits on other futures inside open transactions.
#[derive(Debug)]
pub struct Transactions {
    /// Path of the inspected file.
    file: String,
    /// Violations found so far.
    findings: Vec<Finding>,
}

impl Transactions {
    /// Starts the inspection of one file.
    #[must_use]
    pub fn new(file: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            findings: Vec::new(),
        }
    }
}

impl From<Transactions> for Vec<Finding> {
    fn from(transactions: Transactions) -> Self {
        transactions.findings
    }
}

impl<'ast> Visit<'ast> for Transactions {
    fn visit_block(&mut self, node: &'ast Block) {
        let mut open = false;
        for statement in &node.stmts {
            if !open {
                open = statement.opens();
            } else if statement.mentions("commit") {
                open = false;
            } else {
                let mut waits = Waits::new(&self.file);
                waits.visit_stmt(statement);
                self.findings.extend(waits.findings);
            }
        }
        visit::visit_block(self, node);
    }
}
