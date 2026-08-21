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

    let mut func = match syn::parse::<syn::ItemFn>(item) {
        Ok(func) => func,
        Err(err) => return err.to_compile_error().into(),
    };

    // `#[algebraic(skip)]` on a nested item is consumed by the enclosing
    // rewriter. Reaching here means it was used at the top level, where it
    // simply means "do nothing".
    if !scope.skip {
        rewrite::Rewriter::from_scope(scope).visit_item_fn_mut(&mut func);
    }

    func.to_token_stream().into()
}
