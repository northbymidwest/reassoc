mod krate;
mod passthrough;
mod rewrite;
mod scope;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::visit_mut::VisitMut;

/// Rewrite arithmetic operators in a single expression to algebraic dispatch.
#[proc_macro]
pub fn alg(input: TokenStream) -> TokenStream {
    let mut expr = match syn::parse::<syn::Expr>(input) {
        Ok(expr) => expr,
        Err(err) => return err.to_compile_error().into(),
    };
    rewrite::Rewriter::expression_scope().visit_expr_mut(&mut expr);
    expr.to_token_stream().into()
}

/// Rewrite arithmetic operators throughout a function body.
#[proc_macro_attribute]
pub fn algebraic(attr: TokenStream, item: TokenStream) -> TokenStream {
    let scope = match scope::Scope::parse(attr.into()) {
        Ok(scope) => scope,
        Err(err) => return err.to_compile_error().into(),
    };

    let mut func = match syn::parse::<syn::ItemFn>(item.clone()) {
        Ok(func) => func,
        Err(err) => {
            // Parsing as `ItemFn` fails for two very different reasons: the
            // item genuinely isn't a function (e.g. `#[algebraic] impl
            // Foo { .. }`), or it is a function with a real syntax error in
            // it. Only the first case gets an authored message; for the
            // second, syn's own error already points at the actual
            // problem, and overriding it would make that diagnosis worse.
            return match syn::parse::<syn::Item>(item) {
                Ok(syn::Item::Fn(_)) | Err(_) => err,
                Ok(_) => syn::Error::new(
                    err.span(),
                    "`#[algebraic]` applies to functions; it cannot be applied to this item",
                ),
            }
            .to_compile_error()
            .into();
        }
    };

    // The dispatch functions `#[algebraic]` generates calls into
    // (`reassoc::ops::*`) are not `const fn`, so rewriting a `const fn`'s
    // body would fail with E0015 blamed on the attribute rather than on
    // anything the user wrote. Reject it up front with a message that
    // actually explains why.
    if let Some(const_token) = func.sig.constness {
        return syn::Error::new_spanned(
            const_token,
            "`#[algebraic]` cannot be applied to a `const fn`: the dispatch \
             functions it generates (`reassoc::ops::*`) are not `const fn`",
        )
        .to_compile_error()
        .into();
    }

    // `#[algebraic(skip)]` on a nested item is consumed by the enclosing
    // rewriter. Reaching here means it was used at the top level, where it
    // simply means "do nothing".
    if !scope.skip {
        rewrite::Rewriter::from_scope(scope).visit_item_fn_mut(&mut func);
    }

    func.to_token_stream().into()
}

/// Opt a type into `reassoc`'s dispatch layer at its definition.
///
/// Equivalent to `passthrough!(Ty)`, but written where the type is declared.
/// Defaults to all five operators; a type implementing only some of them names
/// the ones it has with `#[passthrough(add, sub, mul)]`.
///
/// See [`reassoc::Passthrough`] for a worked example — this crate cannot depend
/// on `reassoc`, so the example lives there where it can actually be compiled.
#[proc_macro_derive(Passthrough, attributes(passthrough))]
pub fn derive_passthrough(input: TokenStream) -> TokenStream {
    let input = match syn::parse::<syn::DeriveInput>(input) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error().into(),
    };
    match passthrough::expand(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
