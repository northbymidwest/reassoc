use proc_macro2::Span;
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::visit_mut::{self, VisitMut};
use syn::{BinOp, Expr, UnOp};

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

/// Checks whether an expression is a plausible left-hand side for a
/// compound assignment.
///
/// Native Rust rejects `a + b += x` outright (`E0067`); without this check
/// our rewrite would silently accept it and mutate a discarded temporary
/// instead. This is deliberately permissive rather than a precise place-
/// expression checker: `Expr::Macro` is always accepted, since a macro can
/// expand to a place and we have no way to tell without expanding it — a
/// false negative here just falls through to the compiler's own place
/// check downstream, while a false positive would reject valid code.
fn is_place_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Path(_) | Expr::Field(_) | Expr::Index(_) | Expr::Macro(_) => true,
        Expr::Unary(unary) => matches!(unary.op, UnOp::Deref(_)),
        Expr::Paren(inner) => is_place_expr(&inner.expr),
        Expr::Group(inner) => is_place_expr(&inner.expr),
        _ => false,
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
                if !is_place_expr(&binary.left) {
                    // Mirror rustc's E0067: `a + b += x` is not a place,
                    // and letting it through would silently mutate a
                    // discarded temporary instead of erroring.
                    let err = syn::Error::new_spanned(
                        &*binary.left,
                        "invalid left-hand side of compound assignment",
                    );
                    *expr = syn::parse2(err.to_compile_error())
                        .expect("compile_error! must parse as an expression");
                } else {
                    let func = syn::Ident::new(name, Span::call_site());
                    // Strip parens from both operands: from the place so
                    // the `&mut` binding below doesn't wrap it in a
                    // redundant group, and from the RHS for the same
                    // reason as the non-assigning arm above.
                    let left = unparen(&binary.left);
                    let right = unparen(&binary.right);
                    // Bind the place through a `&mut` temporary so it is
                    // evaluated exactly once; a naive `place = f(place,
                    // rhs)` rewrite would evaluate `place` twice.
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
}
