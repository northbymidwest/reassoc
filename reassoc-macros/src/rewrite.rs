use proc_macro2::Span;
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::visit_mut::{self, VisitMut};
use syn::{BinOp, Expr};

pub struct Rewriter {
    /// Descend into closure bodies.
    pub closures: bool,
    /// Descend into nested `fn` / `impl` / `mod` items.
    pub items: bool,
}

impl Rewriter {
    /// Scope used by `alg!`, which only ever sees a single expression.
    pub fn expression_scope() -> Self {
        Rewriter { closures: true, items: true }
    }
}

/// Maps a binary operator to the dispatch function that replaces it.
/// Returns `None` for operators we do not touch (comparison, logical,
/// bitwise, shifts).
fn dispatch_fn(op: &BinOp) -> Option<&'static str> {
    match op {
        BinOp::Add(_) => Some("add"),
        BinOp::Sub(_) => Some("sub"),
        BinOp::Mul(_) => Some("mul"),
        BinOp::Div(_) => Some("div"),
        BinOp::Rem(_) => Some("rem"),
        _ => None,
    }
}

impl VisitMut for Rewriter {
    fn visit_expr_closure_mut(&mut self, closure: &mut syn::ExprClosure) {
        if self.closures {
            visit_mut::visit_expr_closure_mut(self, closure);
        }
    }

    fn visit_item_mut(&mut self, item: &mut syn::Item) {
        if self.items {
            visit_mut::visit_item_mut(self, item);
        }
    }

    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // Rewrite children first, so nested operators are already converted
        // by the time we rebuild this node.
        visit_mut::visit_expr_mut(self, expr);

        if let Expr::Binary(binary) = expr
            && let Some(name) = dispatch_fn(&binary.op)
        {
            let func = syn::Ident::new(name, Span::call_site());
            let left = &binary.left;
            let right = &binary.right;
            // Span the call at the operator so type errors point there.
            let span = binary.op.span();
            *expr = syn::parse2(quote_spanned! {span=>
                ::reassoc::ops::#func(#left, #right)
            })
            .expect("generated dispatch call must parse");
        }
    }
}
