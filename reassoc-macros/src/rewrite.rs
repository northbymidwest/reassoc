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

/// Strips one layer of grouping parens from an operand.
///
/// Exactly one layer, deliberately. The outermost layer is the one macro
/// expansion makes redundant: `(a + b) * c` needs those parens in source,
/// but the rewritten call delimits its own arguments, so they would trip
/// `unused_parens` in user code. Any FURTHER layers were already redundant
/// in the user's source, so they are left in place for rustc to lint —
/// stripping them too would silently swallow a real diagnostic.
fn unparen(expr: &Expr) -> &Expr {
    if let Expr::Paren(inner) = expr {
        return &inner.expr;
    }
    expr
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

/// Maps a compound-assignment operator to its dispatch function.
/// `a += b` parses as `Expr::Binary` with `BinOp::AddAssign`; syn has no
/// separate assignment-op node.
fn dispatch_fn_assign(op: &BinOp) -> Option<&'static str> {
    match op {
        BinOp::AddAssign(_) => Some("add"),
        BinOp::SubAssign(_) => Some("sub"),
        BinOp::MulAssign(_) => Some("mul"),
        BinOp::DivAssign(_) => Some("div"),
        BinOp::RemAssign(_) => Some("rem"),
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

        if let Expr::Binary(binary) = expr {
            // Span the call at the operator so type errors point there.
            let span = binary.op.span();

            if let Some(name) = dispatch_fn(&binary.op) {
                let func = syn::Ident::new(name, Span::call_site());
                let left = unparen(&binary.left);
                let right = unparen(&binary.right);
                *expr = syn::parse2(quote_spanned! {span=>
                    ::reassoc::ops::#func(#left, #right)
                })
                .expect("generated dispatch call must parse");
            } else if let Some(name) = dispatch_fn_assign(&binary.op) {
                let func = syn::Ident::new(name, Span::call_site());
                // Strip parens from both operands: from the place so the
                // `&mut` binding below doesn't wrap it in a redundant
                // group, and from the RHS for the same reason as the
                // non-assigning arm above.
                let left = unparen(&binary.left);
                let right = unparen(&binary.right);
                // Bind the place through a `&mut` temporary so it is
                // evaluated exactly once; a naive `place = f(place, rhs)`
                // rewrite would evaluate `place` twice.
                *expr = syn::parse2(quote_spanned! {span=>
                    {
                        let __reassoc_place = &mut #left;
                        *__reassoc_place = ::reassoc::ops::#func(*__reassoc_place, #right);
                    }
                })
                .expect("generated compound assignment must parse");
            }
        }
    }
}
