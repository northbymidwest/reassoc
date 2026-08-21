mod krate;
mod passthrough;
mod rewrite;
mod scope;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse::Parser;
use syn::visit_mut::VisitMut;

/// Declare what an operator yields for a left-hand type, when it is not that
/// type itself. Not part of the public API: `passthrough!` calls it.
#[doc(hidden)]
#[proc_macro]
pub fn declare_output(input: TokenStream) -> TokenStream {
    match passthrough::expand_declare_output(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Rewrite arithmetic operators in a single expression to algebraic dispatch.
#[proc_macro]
pub fn alg(input: TokenStream) -> TokenStream {
    // An expression first, so the single-expression form behaves exactly as it
    // always has and never picks up braces it did not ask for.
    if let Ok(mut expr) = syn::parse::<syn::Expr>(input.clone()) {
        rewrite::Rewriter::expression_scope().visit_expr_mut(&mut expr);
        return expr.to_token_stream().into();
    }

    // Otherwise a sequence of statements. Note the braces of `alg! { .. }` are
    // the macro's own delimiters and never reach us, so this parses the body
    // without them (`Block::parse_within`) and supplies the braces on output —
    // which also keeps the result usable anywhere a value is expected.
    match syn::Block::parse_within.parse(input) {
        Ok(stmts) => {
            let mut block = syn::Block {
                brace_token: syn::token::Brace::default(),
                stmts,
            };
            rewrite::Rewriter::expression_scope().visit_block_mut(&mut block);
            block.to_token_stream().into()
        }
        Err(err) => err.to_compile_error().into(),
    }
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
            // Not a function at all gets an authored message; a function
            // with a syntax error keeps syn's, which points at the problem.
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

    // `ops::*` are not `const fn`; rejecting up front beats an E0015 blamed
    // on the attribute.
    if let Some(const_token) = func.sig.constness {
        return syn::Error::new_spanned(
            const_token,
            "`#[algebraic]` cannot be applied to a `const fn`: the dispatch \
             functions it generates (`reassoc::ops::*`) are not `const fn`",
        )
        .to_compile_error()
        .into();
    }

    // `skip` at the top level simply means "do nothing".
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
/// See `reassoc::Passthrough` for a worked example — this crate cannot depend
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
