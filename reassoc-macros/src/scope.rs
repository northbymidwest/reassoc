use proc_macro2::TokenStream;

/// Which parts of an item `#[algebraic]` descends into.
#[derive(Clone, Copy)]
pub struct Scope {
    pub closures: bool,
    /// Enter the arguments of the std macros whose arguments are
    /// expressions (`assert!`, `println!`, `vec!`, ..).
    pub macros: bool,
    pub skip: bool,
    /// Route the float operators to the `f*_fast` intrinsics: `ops::fast::*`.
    pub fast: bool,
}

impl Default for Scope {
    fn default() -> Self {
        // Everything lexically inside the annotated scope is algebraic:
        // closures and nested items alike. Nested items used to be out by
        // default on the reading that a nested `fn` is "a standalone item",
        // which left a helper silently strict inside a body that looked
        // covered and, once containers propagated all the way down, made a
        // function body the one place nesting stopped. Opting out is
        // `#[algebraic(skip)]` on the item.
        Scope {
            closures: true,
            macros: true,
            skip: false,
            fast: false,
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
            } else if meta.path.is_ident("macros") {
                scope.macros = meta.value()?.parse::<syn::LitBool>()?.value();
                Ok(())
            } else if meta.path.is_ident("skip") {
                scope.skip = true;
                Ok(())
            } else if meta.path.is_ident("unsafe_fast") {
                if cfg!(feature = "unstable-fast-math") {
                    scope.fast = true;
                    Ok(())
                } else {
                    Err(meta.error(
                        "`unsafe_fast` needs the `unstable-fast-math` feature of `reassoc` (nightly): \
                         the scope's float operators become `f*_fast` intrinsics, undefined \
                         behaviour on a NaN or infinity",
                    ))
                }
            } else if meta.path.is_ident("items") {
                // Deprecated in 0.4.0, removed in 0.8.0. An authored error
                // rather than "unknown parameter", since old code has it.
                Err(meta.error(
                    "`items` was removed: nested items are always entered. To leave one \
                     alone, put `#[algebraic(skip)]` on it",
                ))
            } else {
                Err(meta.error(
                    "unknown `#[algebraic]` parameter; expected `closures`, `macros`, `skip`, or `unsafe_fast`",
                ))
            }
        });

        syn::parse::Parser::parse2(parser, attr)?;
        Ok(scope)
    }
}
