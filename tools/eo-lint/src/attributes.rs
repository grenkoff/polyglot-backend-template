//! Attributes that mark test code and waive rules.

use syn::punctuated::Punctuated;
use syn::{Attribute, Expr, ExprLit, Lit, Meta, Token};

/// Comma-separated arguments of an attribute.
type Metas = Punctuated<Meta, Token![,]>;

/// One `allow(...)` entry inside `cfg_attr(eo, ...)`.
#[derive(Debug, Clone, Copy)]
pub struct Allowance<'a> {
    /// The entry itself.
    meta: &'a Meta,
}

impl<'a> Allowance<'a> {
    /// Wraps one attribute argument.
    #[must_use]
    pub const fn new(meta: &'a Meta) -> Self {
        Self { meta }
    }
}

/// Tells whether an entry waives a rule.
pub trait Exemption {
    /// Returns true when the entry allows the rule and gives a reason.
    fn covers(&self, rule: &str) -> bool;
}

impl Exemption for Allowance<'_> {
    fn covers(&self, rule: &str) -> bool {
        let Meta::List(list) = self.meta else {
            return false;
        };
        list.path.is_ident("allow")
            && list
                .parse_args_with(Metas::parse_terminated)
                .is_ok_and(|items| {
                    let named = items.iter().any(|item| item.path().is_ident(rule));
                    let reasoned = items.iter().any(|item| {
                        matches!(item, Meta::NameValue(pair)
                            if pair.path.is_ident("reason")
                                && matches!(&pair.value, Expr::Lit(ExprLit { lit: Lit::Str(text), .. })
                                    if !text.value().trim().is_empty()))
                    });
                    named && reasoned
                })
    }
}

/// Questions about the attributes of an item.
pub trait Attributes {
    /// Returns true when the attributes waive the rule.
    fn waives(&self, rule: &str) -> bool;

    /// Returns true when the item exists only for tests.
    fn testing(&self) -> bool;
}

/// Attributes of one item.
#[derive(Debug, Clone, Copy)]
pub struct Marks<'a> {
    /// The attributes themselves.
    attrs: &'a [Attribute],
}

impl<'a> Marks<'a> {
    /// Wraps the attributes of an item.
    #[must_use]
    pub const fn new(attrs: &'a [Attribute]) -> Self {
        Self { attrs }
    }
}

impl Attributes for Marks<'_> {
    fn waives(&self, rule: &str) -> bool {
        self.attrs.iter().any(|attr| {
            attr.path().is_ident("cfg_attr")
                && attr
                    .parse_args_with(Metas::parse_terminated)
                    .is_ok_and(|metas| {
                        let mut parts = metas.iter();
                        parts
                            .next()
                            .is_some_and(|predicate| predicate.path().is_ident("eo"))
                            && parts.any(|meta| Allowance::new(meta).covers(rule))
                    })
        })
    }

    fn testing(&self) -> bool {
        self.attrs.iter().any(|attr| {
            let test = attr
                .path()
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "test");
            let cfg = attr.path().is_ident("cfg")
                && attr
                    .parse_args::<syn::Ident>()
                    .is_ok_and(|ident| ident == "test");
            test || cfg
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Attributes, Marks};

    #[test]
    fn finds_waiver_with_reason() {
        let item: syn::ItemStruct = syn::parse_quote! {
            #[cfg_attr(eo, allow(EO006, reason = "plain data for serde"))]
            struct Row;
        };
        assert!(Marks::new(&item.attrs).waives("EO006"));
        assert!(!Marks::new(&item.attrs).waives("EO001"));
    }

    #[test]
    fn refuses_waiver_without_reason() {
        let item: syn::ItemStruct = syn::parse_quote! {
            #[cfg_attr(eo, allow(EO006))]
            #[cfg_attr(eo, allow(EO006, reason = "   "))]
            #[cfg_attr(eo, allow(EO006, reason = 1))]
            #[cfg_attr(eo, allow(EO006, because = "wrong key"))]
            #[cfg_attr(eo, inline)]
            struct Row;
        };
        assert!(!Marks::new(&item.attrs).waives("EO006"));
    }

    #[test]
    fn ignores_other_attributes() {
        let item: syn::ItemStruct = syn::parse_quote! {
            #[cfg_attr(test, allow(EO006, reason = "not the eo predicate"))]
            #[cfg_attr(eo, deny(EO006, reason = "not an allow"))]
            #[allow(dead_code, reason = "not cfg_attr")]
            #[doc = "eo:allow EO006 old comment syntax"]
            #[derive(Debug)]
            struct Row;
        };
        assert!(!Marks::new(&item.attrs).waives("EO006"));
    }

    #[test]
    fn recognizes_test_code() {
        let unit: syn::ItemFn = syn::parse_quote! { #[tokio::test] async fn works() {} };
        let module: syn::ItemMod = syn::parse_quote! { #[cfg(test)] mod tests {} };
        let plain: syn::ItemFn = syn::parse_quote! { #[inline] fn works() {} };
        let platform: syn::ItemFn = syn::parse_quote! { #[cfg(unix)] fn works() {} };
        assert!(Marks::new(&unit.attrs).testing());
        assert!(Marks::new(&module.attrs).testing());
        assert!(!Marks::new(&plain.attrs).testing());
        assert!(!Marks::new(&platform.attrs).testing());
    }
}
