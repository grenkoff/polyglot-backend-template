use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Expr, Fields, GenericArgument, ImplItem, ImplItemFn, ItemEnum, ItemFn, ItemImpl, ItemMod,
    ItemStatic, ItemStruct, ItemTrait, PathArguments, Receiver, ReceiverKind, ReturnType, Stmt,
    TraitItem, Type, Visibility,
};

pub trait Mutation {
    fn mutates(&self) -> bool;
}

impl Mutation for Receiver {
    fn mutates(&self) -> bool {
        self.mutability.is_some() || matches!(self.kind, ReceiverKind::Reference(_, _, Some(_)))
    }
}

use crate::attributes::{Attributes, Marks};
use crate::finding::Finding;

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

#[derive(Debug)]
pub struct Inspection {
    file: String,
    entry: bool,
    findings: Vec<Finding>,
    doc: Option<usize>,
}

impl Inspection {
    #[must_use]
    pub fn new(file: impl Into<String>, entry: bool) -> Self {
        Self {
            file: file.into(),
            entry,
            findings: Vec::new(),
            doc: None,
        }
    }
}

impl From<Inspection> for Vec<Finding> {
    fn from(inspection: Inspection) -> Self {
        inspection.findings
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TypeName<'a> {
    ident: &'a syn::Ident,
}

impl<'a> TypeName<'a> {
    #[must_use]
    pub const fn new(ident: &'a syn::Ident) -> Self {
        Self { ident }
    }
}

pub trait Agency {
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

#[derive(Debug, Clone, Copy)]
pub struct Output<'a> {
    output: &'a ReturnType,
    owner: &'a str,
}

impl<'a> Output<'a> {
    #[must_use]
    pub const fn new(output: &'a ReturnType, owner: &'a str) -> Self {
        Self { output, owner }
    }
}

pub trait Product {
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

#[derive(Debug, Clone, Copy)]
pub struct Body<'a> {
    stmts: &'a [Stmt],
}

impl<'a> Body<'a> {
    #[must_use]
    pub const fn new(stmts: &'a [Stmt]) -> Self {
        Self { stmts }
    }
}

pub trait Assignment {
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

#[derive(Debug, Clone, Copy)]
pub struct Method<'a> {
    item: &'a ImplItemFn,
    owner: &'a str,
    outer: Marks<'a>,
}

impl<'a> Method<'a> {
    #[must_use]
    pub const fn new(item: &'a ImplItemFn, owner: &'a str, outer: Marks<'a>) -> Self {
        Self { item, owner, outer }
    }
}

pub trait Verdict {
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
        let own = Marks::new(&self.item.attrs);
        checks
            .into_iter()
            .filter(|(rule, broken, _)| *broken && !own.waives(rule) && !self.outer.waives(rule))
            .map(|(rule, _, message)| (rule, message))
            .collect()
    }
}

impl<'ast> Visit<'ast> for Inspection {
    fn visit_attribute(&mut self, node: &'ast syn::Attribute) {
        if node.path().is_ident("doc") {
            let line = node.span().start().line;
            if self.doc.is_some_and(|last| last + 1 == line) {
                self.findings.push(Finding::new(
                    &self.file,
                    line,
                    "EO012",
                    "documentation has more than one line, keep it to one",
                ));
            }
            self.doc = Some(line);
        }
    }

    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        if !Marks::new(&node.attrs).testing() {
            visit::visit_item_mod(self, node);
        }
    }

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        let marks = Marks::new(&node.attrs);
        if TypeName::new(&node.ident).doer() && !marks.waives("EO001") {
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
            if !matches!(field.vis, Visibility::Inherited) && !marks.waives("EO006") {
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
        if TypeName::new(&node.ident).doer() && !Marks::new(&node.attrs).waives("EO001") {
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
        let marks = Marks::new(&node.attrs);
        if TypeName::new(&node.ident).doer() && !marks.waives("EO001") {
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
                if (name.starts_with("get_") || name.starts_with("set_")) && !marks.waives("EO002")
                {
                    self.findings.push(Finding::new(
                        &self.file,
                        method.sig.ident.span().start().line,
                        "EO002",
                        format!("`{name}` is a getter or setter, name it after what it returns"),
                    ));
                }
                if method.sig.receiver().is_some_and(Mutation::mutates) && !marks.waives("EO003") {
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
        if !Marks::new(&node.attrs).waives("EO007") {
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
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let marks = Marks::new(&node.attrs);
        if marks.testing() {
            return;
        }
        let main = self.entry && node.sig.ident == "main";
        if !main && !marks.waives("EO005") {
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
        let marks = Marks::new(&node.attrs);
        if marks.testing() {
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
                    Method::new(method, &owner, marks)
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
        Code::new("lib.rs", code)
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
    fn respects_waivers() {
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
        assert_eq!(rules(code), Vec::<String>::new());
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
    fn checks_methods_of_trait_objects() {
        let code = "trait Shape {} impl dyn Shape { pub fn area(&self) {} }";
        assert_eq!(rules(code), ["EO004"]);
    }
}
