//! Attributes that mark test code.

use syn::Attribute;

/// Questions about the attributes of an item.
pub trait Attributes {
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
