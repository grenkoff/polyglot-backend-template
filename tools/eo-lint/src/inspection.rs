//! Elegant Objects rules checked on the syntax tree of one file.

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Expr, Fields, GenericArgument, ImplItem, ImplItemFn, ItemEnum, ItemFn, ItemImpl, ItemMod,
    ItemStatic, ItemStruct, ItemTrait, PathArguments, Receiver, ReceiverKind, ReturnType, Stmt,
    TraitItem, Type, Visibility,
};

/// Tells whether a receiver lets a method change its object.
pub trait Mutation {
    /// Returns true for `&mut self` and `mut self`.
    fn mutates(&self) -> bool;
}

impl Mutation for Receiver {
    fn mutates(&self) -> bool {
        self.mutability.is_some() || matches!(self.kind, ReceiverKind::Reference(_, _, Some(_)))
    }
}

use crate::attributes::{Attributes, Marks};
use crate::finding::Finding;

/// Words ending in -er or -or that name things, not doers.
const NOUNS: &[&str] = &[
    "Error",
    "Order",
    "Number",
    "Header",
    "Footer",
    "Folder",
    "Paper",
    "Layer",
    "Parameter",
    "User",
    "Owner",
    "Member",
    "Customer",
    "Letter",
    "Character",
    "Corner",
    "Border",
    "Color",
    "Vector",
    "Floor",
    "Door",
    "Mirror",
    "Tier",
    "Water",
    "Cluster",
    "Chapter",
    "Counter",
    "Power",
    "Ledger",
    "Weather",
    "Answer",
    "Matter",
    "Center",
    "Meter",
    "Liter",
    "Hour",
    "Minor",
    "Major",
];

/// Visitor that collects the violations of one file.
#[derive(Debug)]
pub struct Inspection {
    /// Path of the inspected file.
    file: String,
    /// Whether the file is an entry point such as `main.rs`.
    entry: bool,
    /// Violations found so far.
    findings: Vec<Finding>,
    /// The last doc line seen, to catch long docs.
    doc: DocLine,
}

impl Inspection {
    /// Starts the inspection of one file.
    #[must_use]
    pub fn new(file: impl Into<String>, entry: bool) -> Self {
        Self {
            file: file.into(),
            entry,
            findings: Vec::new(),
            doc: DocLine::new(0, false, 0),
        }
    }
}

impl From<Inspection> for Vec<Finding> {
    fn from(inspection: Inspection) -> Self {
        inspection.findings
    }
}

/// Name of a type, checked for doer suffixes.
#[derive(Debug, Clone, Copy)]
pub struct TypeName<'a> {
    /// Identifier of the type.
    ident: &'a syn::Ident,
}

impl<'a> TypeName<'a> {
    /// Wraps a type identifier.
    #[must_use]
    pub const fn new(ident: &'a syn::Ident) -> Self {
        Self { ident }
    }
}

/// Tells whether a name describes an object that does something.
pub trait Agency {
    /// Returns true for names ending in -er or -or that are not known nouns.
    fn doer(&self) -> bool;
}

impl Agency for TypeName<'_> {
    fn doer(&self) -> bool {
        let name = self.ident.to_string();
        let start = name
            .rfind(|letter: char| letter.is_ascii_uppercase())
            .unwrap_or(0);
        let word = &name[start..];
        (word.ends_with("er") || word.ends_with("or")) && !NOUNS.contains(&word)
    }
}

/// Return type of a function, compared with its owner.
#[derive(Debug, Clone, Copy)]
pub struct Output<'a> {
    /// The declared return type.
    output: &'a ReturnType,
    /// Name of the type that owns the function.
    owner: &'a str,
}

impl<'a> Output<'a> {
    /// Wraps a return type and the name of its owner.
    #[must_use]
    pub const fn new(output: &'a ReturnType, owner: &'a str) -> Self {
        Self { output, owner }
    }
}

/// Tells whether a function produces its own type.
pub trait Product {
    /// Returns true for `Self`, the owner, or a wrapper of them.
    fn selfish(&self) -> bool;
}

impl Product for Output<'_> {
    fn selfish(&self) -> bool {
        let ReturnType::Type(_, kind) = self.output else {
            return false;
        };
        let Type::Path(path) = kind.as_ref() else {
            return false;
        };
        path.path.segments.last().is_some_and(|segment| {
            let named = segment.ident == "Self" || segment.ident == self.owner;
            let wrapped = matches!(&segment.arguments, PathArguments::AngleBracketed(args)
                if args.args.iter().any(|arg| matches!(arg,
                    GenericArgument::Type(Type::Path(inner))
                        if inner.path.is_ident("Self") || inner.path.is_ident(self.owner))));
            named || wrapped
        })
    }
}

/// Statements of a constructor body.
#[derive(Debug, Clone, Copy)]
pub struct Body<'a> {
    /// The statements themselves.
    stmts: &'a [Stmt],
}

impl<'a> Body<'a> {
    /// Wraps the statements of a body.
    #[must_use]
    pub const fn new(stmts: &'a [Stmt]) -> Self {
        Self { stmts }
    }
}

/// Tells whether a constructor does more than assign fields.
pub trait Assignment {
    /// Returns true when the body is more than one plain construction.
    fn busy(&self) -> bool;
}

impl Assignment for Body<'_> {
    fn busy(&self) -> bool {
        match self.stmts {
            [Stmt::Expr(expr, None)] => !matches!(
                expr,
                Expr::Struct(_) | Expr::Call(_) | Expr::MethodCall(_) | Expr::Path(_)
            ),
            _ => true,
        }
    }
}

/// Inherent method checked against Elegant Objects rules.
#[derive(Debug, Clone, Copy)]
pub struct Method<'a> {
    /// The method itself.
    item: &'a ImplItemFn,
    /// Name of the type that owns the method.
    owner: &'a str,
}

impl<'a> Method<'a> {
    /// Wraps a method with its owner.
    #[must_use]
    pub const fn new(item: &'a ImplItemFn, owner: &'a str) -> Self {
        Self { item, owner }
    }
}

/// Lists the rules a method breaks.
pub trait Verdict {
    /// Returns each broken rule with its message.
    fn verdicts(&self) -> Vec<(&'static str, String)>;
}

impl Verdict for Method<'_> {
    fn verdicts(&self) -> Vec<(&'static str, String)> {
        let name = self.item.sig.ident.to_string();
        let receiver = self.item.sig.receiver();
        let constructor = Output::new(&self.item.sig.output, self.owner).selfish();
        let checks = [
            (
                "EO002",
                name.starts_with("get_") || name.starts_with("set_"),
                format!("`{name}` is a getter or setter, name it after what it returns"),
            ),
            (
                "EO003",
                receiver.is_some_and(Mutation::mutates),
                format!("`{name}` takes `&mut self`, objects must be immutable"),
            ),
            (
                "EO004",
                receiver.is_some() && matches!(self.item.vis, Visibility::Public(_)),
                format!("`{name}` is public but comes from no trait"),
            ),
            (
                "EO008",
                receiver.is_none() && !constructor,
                format!("`{name}` is a static method, make it an object"),
            ),
            (
                "EO008",
                receiver.is_none() && constructor && Body::new(&self.item.block.stmts).busy(),
                format!("constructor `{name}` has code, it must only assign fields"),
            ),
        ];
        checks
            .into_iter()
            .filter(|(_, broken, _)| *broken)
            .map(|(rule, _, message)| (rule, message))
            .collect()
    }
}

/// Position of a doc line and the run of doc lines it ends.
#[derive(Debug, Clone, Copy)]
pub struct DocLine {
    /// Line of the doc attribute.
    line: usize,
    /// Whether it documents the enclosing module (`//!`).
    inner: bool,
    /// How many doc lines in a row end here.
    run: usize,
}

impl DocLine {
    /// Creates a doc line position.
    #[must_use]
    pub const fn new(line: usize, inner: bool, run: usize) -> Self {
        Self { line, inner, run }
    }
}

/// Follows doc lines one after another.
pub trait Sequence {
    /// Returns the position of the next doc line.
    fn next(&self, line: usize, inner: bool) -> DocLine;
    /// Returns true on the second line of a doc, where it becomes too long.
    fn excess(&self) -> bool;
}

impl Sequence for DocLine {
    fn next(&self, line: usize, inner: bool) -> DocLine {
        let continued = self.line + 1 == line && self.inner == inner;
        Self::new(line, inner, if continued { self.run + 1 } else { 1 })
    }

    fn excess(&self) -> bool {
        self.run == 2
    }
}

impl<'ast> Visit<'ast> for Inspection {
    fn visit_attribute(&mut self, node: &'ast syn::Attribute) {
        if node.path().is_ident("doc") {
            let line = node.span().start().line;
            let doc = self
                .doc
                .next(line, matches!(node.style, syn::AttrStyle::Inner(_)));
            if doc.excess() {
                self.findings.push(Finding::new(
                    &self.file,
                    line,
                    "EO012",
                    "documentation has more than one line, keep it to one",
                ));
            }
            self.doc = doc;
        }
    }

    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        if !Marks::new(&node.attrs).testing() {
            visit::visit_item_mod(self, node);
        }
    }

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        if TypeName::new(&node.ident).doer() {
            self.findings.push(Finding::new(
                &self.file,
                node.ident.span().start().line,
                "EO001",
                format!(
                    "`{}` names a doer, name the object after what it is",
                    node.ident
                ),
            ));
        }
        let fields = match &node.fields {
            Fields::Named(named) => named.named.iter().collect(),
            Fields::Unnamed(unnamed) => unnamed.unnamed.iter().collect(),
            Fields::Unit => Vec::new(),
        };
        for field in fields {
            if !matches!(field.vis, Visibility::Inherited) {
                self.findings.push(Finding::new(
                    &self.file,
                    field.span().start().line,
                    "EO006",
                    format!(
                        "`{}` has a public field, expose behaviour instead",
                        node.ident
                    ),
                ));
            }
        }
        visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        if TypeName::new(&node.ident).doer() {
            self.findings.push(Finding::new(
                &self.file,
                node.ident.span().start().line,
                "EO001",
                format!(
                    "`{}` names a doer, name the type after what it is",
                    node.ident
                ),
            ));
        }
        visit::visit_item_enum(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast ItemTrait) {
        if TypeName::new(&node.ident).doer() {
            self.findings.push(Finding::new(
                &self.file,
                node.ident.span().start().line,
                "EO001",
                format!("`{}` names a doer, name the trait after a role", node.ident),
            ));
        }
        for item in &node.items {
            if let TraitItem::Fn(method) = item {
                let name = method.sig.ident.to_string();
                if name.starts_with("get_") || name.starts_with("set_") {
                    self.findings.push(Finding::new(
                        &self.file,
                        method.sig.ident.span().start().line,
                        "EO002",
                        format!("`{name}` is a getter or setter, name it after what it returns"),
                    ));
                }
                if method.sig.receiver().is_some_and(Mutation::mutates) {
                    self.findings.push(Finding::new(
                        &self.file,
                        method.sig.ident.span().start().line,
                        "EO003",
                        format!("`{name}` takes `&mut self`, objects must be immutable"),
                    ));
                }
            }
        }
        visit::visit_item_trait(self, node);
    }

    fn visit_item_static(&mut self, node: &'ast ItemStatic) {
        self.findings.push(Finding::new(
            &self.file,
            node.ident.span().start().line,
            "EO007",
            format!(
                "`static {}` is global state, pass objects explicitly",
                node.ident
            ),
        ));
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if Marks::new(&node.attrs).testing() {
            return;
        }
        let main = self.entry && node.sig.ident == "main";
        if !main {
            self.findings.push(Finding::new(
                &self.file,
                node.sig.ident.span().start().line,
                "EO005",
                format!(
                    "`fn {}` is a procedure, make it a method of an object",
                    node.sig.ident
                ),
            ));
        }
        visit::visit_item_fn(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        if Marks::new(&node.attrs).testing() {
            return;
        }
        if node.trait_.is_some() {
            visit::visit_item_impl(self, node);
            return;
        }
        let owner = match node.self_ty.as_ref() {
            Type::Path(path) => path
                .path
                .segments
                .last()
                .map(|segment| segment.ident.to_string())
                .unwrap_or_default(),
            _ => String::new(),
        };
        for item in &node.items {
            if let ImplItem::Fn(method) = item {
                let line = method.sig.ident.span().start().line;
                self.findings.extend(
                    Method::new(method, &owner)
                        .verdicts()
                        .into_iter()
                        .map(|(rule, message)| Finding::new(&self.file, line, rule, message)),
                );
            }
        }
        visit::visit_item_impl(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let name = node.method.to_string();
        if matches!(
            name.as_str(),
            "downcast" | "downcast_ref" | "downcast_mut" | "type_id"
        ) {
            self.findings.push(Finding::new(
                &self.file,
                node.method.span().start().line,
                "EO009",
                format!("`{name}` is reflection, rely on traits instead"),
            ));
        }
        visit::visit_expr_method_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use crate::source::{Code, Source};

    fn rules(code: &str) -> Vec<String> {
        rules_of("lib.rs", code)
    }

    fn rules_of(file: &str, code: &str) -> Vec<String> {
        Code::new(file, code)
            .findings()
            .unwrap()
            .iter()
            .map(|finding| finding.to_string().split(' ').nth(1).unwrap().to_owned())
            .collect()
    }

    #[test]
    fn accepts_elegant_code() {
        let code = r#"
            pub trait Greeting { fn text(&self) -> String; }
            pub struct Plain { name: String }
            impl Plain {
                pub fn new(name: impl Into<String>) -> Self { Self { name: name.into() } }
            }
            impl Greeting for Plain {
                fn text(&self) -> String { format!("Hello, {}!", self.name) }
            }
            impl std::fmt::Display for Plain {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) }
            }
            pub struct Order;
            #[cfg(test)]
            mod tests { fn helper() {} pub struct Manager { pub x: u8 } }
        "#;
        assert_eq!(rules(code), Vec::<String>::new());
    }

    #[test]
    fn rejects_doers() {
        assert_eq!(
            rules("struct UserManager; enum Parser {} trait Handler {}"),
            ["EO001", "EO001", "EO001"]
        );
    }

    #[test]
    fn rejects_getters_and_setters() {
        let code = "trait Age { fn get_age(&self) -> u8; }
            struct Cat; impl Cat { fn set_age(&self) {} }";
        assert_eq!(rules(code), ["EO002", "EO002"]);
    }

    #[test]
    fn rejects_mutable_objects() {
        let code = "trait Cell { fn put(&mut self, v: u8); }
            struct Cat; impl Cat { fn grow(&mut self) {} }";
        assert_eq!(rules(code), ["EO003", "EO003"]);
    }

    #[test]
    fn rejects_objects_mutated_by_value() {
        let code = "trait Cell { fn put(mut self, v: u8); }
            struct Cat; impl Cat { fn grow(mut self) {} fn age(&self) {} fn pass(self) {} }";
        assert_eq!(rules(code), ["EO003", "EO003"]);
    }

    #[test]
    fn rejects_public_methods_without_trait() {
        assert_eq!(
            rules("struct Cat; impl Cat { pub fn meow(&self) {} }"),
            ["EO004"]
        );
    }

    #[test]
    fn rejects_procedures_but_allows_main_in_entry_files() {
        assert_eq!(rules("fn helper() {}"), ["EO005"]);
        assert_eq!(rules("fn main() {}"), ["EO005"]);
        assert_eq!(
            crate::source::Code::new("src/main.rs", "fn main() {}")
                .findings()
                .unwrap(),
            Vec::new()
        );
    }

    #[test]
    fn rejects_public_fields() {
        assert_eq!(rules("struct Cat { pub age: u8 }"), ["EO006"]);
    }

    #[test]
    fn rejects_global_state() {
        assert_eq!(rules("static COUNT: u8 = 0;"), ["EO007"]);
    }

    #[test]
    fn rejects_static_methods_and_busy_constructors() {
        let code = "struct Cat { age: u8 }
            impl Cat {
                fn helper(x: u8) -> u8 { x }
                fn new(age: u8) -> Self { let age = age + 1; Self { age } }
                fn try_new(age: u8) -> Result<Self, String> { Ok(Self { age }) }
            }";
        assert_eq!(rules(code), ["EO008", "EO008"]);
    }

    #[test]
    fn rejects_reflection() {
        assert_eq!(
            rules(
                "struct Cat; impl std::fmt::Debug for Cat { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { let x = f.downcast_ref::<u8>(); Ok(()) } }"
            ),
            ["EO009"]
        );
    }

    #[test]
    fn reports_violations_despite_waivers() {
        let code = r#"
            #[cfg_attr(eo, allow(EO006, reason = "plain data transfer object"))]
            struct Row { pub id: u8 }
            struct Cat;
            #[cfg_attr(eo, allow(EO004, reason = "required by the framework"))]
            impl Cat { pub fn meow(&self) {} }
            struct Dog;
            impl Dog {
                #[cfg_attr(eo, allow(EO004, reason = "required by the framework"))]
                pub fn bark(&self) {}
            }
        "#;
        assert_eq!(rules(code), ["EO006", "EO004", "EO004"]);
    }

    #[test]
    fn rejects_public_fields_of_tuple_structs() {
        assert_eq!(rules("struct Id(pub u8);"), ["EO006"]);
    }

    #[test]
    fn rejects_static_methods_returning_other_types() {
        let code = "struct Cat; impl Cat { fn sound() -> &'static str { \"meow\" } }";
        assert_eq!(rules(code), ["EO008"]);
    }

    #[test]
    fn ignores_associated_constants_and_types() {
        let code = "trait Shape { const SIDES: u8; type Unit; }
            struct Cat; impl Cat { const LIVES: u8 = 9; }";
        assert_eq!(rules(code), Vec::<String>::new());
    }

    #[test]
    fn ignores_test_code_outside_test_modules() {
        let code = "#[test] fn works() {}
            struct Cat;
            #[cfg(test)] impl Cat { pub fn meow(&mut self) {} }";
        assert_eq!(rules(code), Vec::<String>::new());
    }

    #[test]
    fn checks_nested_modules() {
        assert_eq!(rules("mod inner { fn helper() {} }"), ["EO005"]);
    }

    #[test]
    fn accepts_constructors_that_name_their_type() {
        let code = "struct Cat; impl Cat { fn new() -> Cat { Cat } }";
        assert_eq!(rules(code), Vec::<String>::new());
    }

    #[test]
    fn reports_multiline_item_docs() {
        assert_eq!(rules("/// One.\n/// Two.\nstruct Cat;"), ["EO012"]);
    }

    #[test]
    fn reports_multiline_module_docs_once() {
        assert_eq!(
            rules("//! One.\n//! Two.\n//! Three.\n/// Cat.\nstruct Cat;"),
            ["EO012"]
        );
    }

    #[test]
    fn checks_methods_of_trait_objects() {
        let code = "trait Shape {} impl dyn Shape { pub fn area(&self) {} }";
        assert_eq!(rules(code), ["EO004"]);
    }

    #[test]
    fn rejects_io_crates_in_domain() {
        let code = "use std::future::Future;
            use tokio::sync::Mutex;
            use tonic::Status;
            use axum::Router;
            use sqlx::PgPool;
            use async_nats::Client;
            use prost::Message;
            use contracts::hello::v1::Greeted;";
        assert_eq!(rules_of("src/domain/name.rs", code), ["EO010"; 7]);
    }

    #[test]
    fn rejects_full_paths_to_io_crates_in_domain() {
        let code =
            "struct Cell { lock: tokio::sync::Mutex<u8>, data: Option<prost::bytes::Bytes> }";
        assert_eq!(rules_of("src/domain/cell.rs", code), ["EO010", "EO010"]);
    }

    #[test]
    fn rejects_outer_layers_in_domain() {
        let code = "use crate::api::Endpoint;
            use crate::infra::Nats;
            use crate::domain::name::Name;";
        assert_eq!(rules_of("src/domain/greeting.rs", code), ["EO010", "EO010"]);
    }
}
