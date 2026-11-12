//! Database transactions that wait for nothing but themselves.

use syn::visit::{self, Visit};
use syn::{Block, ExprAwait, Ident, Local, Pat, Stmt};

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
    /// Returns the name of the transaction the statement binds, if it binds one.
    fn transaction(&self) -> Option<String>;
}

impl Mentions for Stmt {
    fn mentions(&self, name: &str) -> bool {
        let mut trace = Trace::new(name);
        trace.visit_stmt(self);
        trace.into()
    }

    fn transaction(&self) -> Option<String> {
        match self {
            Self::Local(Local {
                pat: Pat::Ident(binding),
                init: Some(_),
                ..
            }) if self.mentions("begin") => Some(binding.ident.to_string()),
            _ => None,
        }
    }
}

/// Visitor that collects awaits on other futures inside a transaction.
#[derive(Debug)]
struct Waits<'a> {
    /// Path of the inspected file.
    file: &'a str,
    /// Name of the open transaction.
    transaction: &'a str,
    /// Violations found so far.
    findings: Vec<Finding>,
}

impl<'a> Waits<'a> {
    /// Starts collecting awaits that do not use the transaction.
    const fn new(file: &'a str, transaction: &'a str) -> Self {
        Self {
            file,
            transaction,
            findings: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for Waits<'_> {
    fn visit_expr_await(&mut self, node: &'ast ExprAwait) {
        let mut trace = Trace::new(self.transaction);
        trace.visit_expr(&node.base);
        if !bool::from(trace) {
            self.findings.push(Finding::new(
                self.file,
                node.await_token.span.start().line,
                "EO011",
                "awaits something else inside a transaction, do I/O before or after it",
            ));
        }
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
        let mut open: Option<String> = None;
        for statement in &node.stmts {
            open = match open {
                None => statement.transaction(),
                Some(_) if statement.mentions("commit") => None,
                Some(transaction) => {
                    let mut waits = Waits::new(&self.file, &transaction);
                    waits.visit_stmt(statement);
                    self.findings.extend(waits.findings);
                    Some(transaction)
                }
            };
        }
        visit::visit_block(self, node);
    }
}
