mod rewrite;

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
