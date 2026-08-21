use proc_macro2::TokenStream;

/// Which parts of an item `#[algebraic]` descends into.
#[derive(Clone, Copy)]
pub struct Scope {
    pub closures: bool,
    pub items: bool,
    pub skip: bool,
}

impl Default for Scope {
    fn default() -> Self {
        // Closure bodies are usually where the hot kernel lives, so they are
        // in by default. A nested `fn` reads like a standalone item, so it
        // must opt in.
        Scope {
            closures: true,
            items: false,
            skip: false,
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
                scope.items = meta.value()?.parse::<syn::LitBool>()?.value();
                Ok(())
            } else if meta.path.is_ident("skip") {
                scope.skip = true;
                Ok(())
            } else {
                Err(meta.error(
                    "unknown `#[algebraic]` parameter; expected `closures`, `items`, or `skip`",
                ))
            }
        });

        syn::parse::Parser::parse2(parser, attr)?;
        Ok(scope)
    }
}
