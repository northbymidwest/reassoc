use proc_macro2::{Span, TokenStream};

/// Which parts of an item `#[algebraic]` descends into.
#[derive(Clone, Copy)]
pub struct Scope {
    pub closures: bool,
    pub items: bool,
    /// Enter the arguments of the std macros whose arguments are
    /// expressions (`assert!`, `println!`, `vec!`, ..).
    pub macros: bool,
    pub skip: bool,
    /// Where the deprecated `items` parameter was written, if it was: the
    /// expansion emits a deprecation warning there.
    pub items_span: Option<Span>,
}

impl Default for Scope {
    fn default() -> Self {
        // Everything lexically inside the annotated scope is algebraic:
        // closures and nested items alike. Nested items used to be out by
        // default on the reading that a nested `fn` is "a standalone item",
        // which left a helper silently strict inside a body that looked
        // covered — and, once containers propagated all the way down, made a
        // function body the one place nesting stopped. Opting out is
        // `#[algebraic(skip)]` on the item.
        Scope {
            closures: true,
            items: true,
            macros: true,
            skip: false,
            items_span: None,
        }
    }
}

impl Scope {
    pub fn parse(attr: TokenStream) -> syn::Result<Self> {
        let mut scope = Scope::default();
        if attr.is_empty() {
            return Ok(scope);
        }

        let parser = syn::meta::parser(|meta| {
            if meta.path.is_ident("closures") {
                scope.closures = meta.value()?.parse::<syn::LitBool>()?.value();
                Ok(())
            } else if meta.path.is_ident("items") {
                // Deprecated: nested items are entered by default. Kept so
                // `items = false` still restores the old boundary; slated for
                // removal.
                scope.items = meta.value()?.parse::<syn::LitBool>()?.value();
                scope.items_span = Some(meta.path.segments[0].ident.span());
                Ok(())
            } else if meta.path.is_ident("macros") {
                scope.macros = meta.value()?.parse::<syn::LitBool>()?.value();
                Ok(())
            } else if meta.path.is_ident("skip") {
                scope.skip = true;
                Ok(())
            } else {
                Err(meta.error(
                    "unknown `#[algebraic]` parameter; expected `closures`, `macros`, or `skip` \
                     (`items` is deprecated)",
                ))
            }
        });

        syn::parse::Parser::parse2(parser, attr)?;
        Ok(scope)
    }
}
