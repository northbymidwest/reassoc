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
        Rewriter {
            closures: true,
            items: true,
        }
    }

    /// Scope used by `#[algebraic]`, configured by its `closures`/`items`
    /// parameters.
    pub fn from_scope(scope: crate::scope::Scope) -> Self {
        Rewriter {
            closures: scope.closures,
            items: scope.items,
        }
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

    // `[expr; len]`'s `len` is evaluated in a const context (it must be,
    // since it fixes the array's type); `ops::*` are not `const fn`, so
    // rewriting arithmetic there trades working code for E0015. This is the
    // default-scope case that needs no opt-in at all: `[0.0f32; 4 * 2]`
    // sits inside an ordinary function body, indistinguishable from any
    // other `Expr::Repeat` except for which of its two children is const.
    // Only the element expression is an ordinary runtime position.
    fn visit_expr_repeat_mut(&mut self, expr_repeat: &mut syn::ExprRepeat) {
        self.visit_expr_mut(&mut expr_repeat.expr);
    }

    // `[T; len]`'s `len` (a *type*, not an expression -- e.g. a return type
    // or a local binding's type annotation) is the same const context as
    // `Expr::Repeat`'s length above, reachable the same way with no opt-in:
    // `fn f() -> [f32; 4 * 2] { .. }`. Only the element type can contain
    // ordinary nested types worth visiting; array types have no other
    // expression-shaped field.
    fn visit_type_array_mut(&mut self, type_array: &mut syn::TypeArray) {
        self.visit_type_mut(&mut type_array.elem);
    }

    // A const-generic argument (`f::<{ 1 + 1 }>()`) is evaluated at const
    // time for the same reason as `Expr::Const` below: `ops::*` are not
    // `const fn`. Every other kind of generic argument (lifetimes, types,
    // associated-type/const bindings) may still contain ordinary runtime
    // expressions nested inside a type and must still be visited normally.
    fn visit_generic_argument_mut(&mut self, arg: &mut syn::GenericArgument) {
        if let syn::GenericArgument::Const(_) = arg {
            return;
        }
        visit_mut::visit_generic_argument_mut(self, arg);
    }

    // A `Variant`'s explicit discriminant (`Variant = 1 + 1`) is a const
    // context, reachable via `items = true` the same way as a nested
    // `Item::Const`/`Item::Static` (`is_const_context` below): `ops::*` are
    // not `const fn`. Only the discriminant is skipped; attributes, the
    // variant name, and its fields are ordinary positions.
    fn visit_variant_mut(&mut self, variant: &mut syn::Variant) {
        self.visit_attributes_mut(&mut variant.attrs);
        self.visit_ident_mut(&mut variant.ident);
        self.visit_fields_mut(&mut variant.fields);
    }

    fn visit_item_mut(&mut self, item: &mut syn::Item) {
        if !self.items {
            return;
        }
        if has_skip_attribute(item) {
            strip_skip_attribute(item);
            return;
        }
        // `const`/`static` initializers, and `const fn` bodies, are const
        // contexts; `ops::*` are not `const fn`, so rewriting inside any of
        // these would trade working code for E0015. Leave them untouched
        // rather than trying to distinguish which sub-expressions are
        // actually evaluated at const time.
        if is_const_context(item) {
            return;
        }
        visit_mut::visit_item_mut(self, item);
    }

    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if let Expr::Macro(_) = expr {
            // Do not rewrite inside a macro invocation: syn hands us a
            // macro's body as an opaque token stream, and we deliberately
            // never parse into it to look for arithmetic. A false positive
            // there (e.g. rewriting inside `format!`'s format string) would
            // be far worse than the false negative of leaving genuine
            // arithmetic inside an unrelated macro alone.
            //
            // `strict!` needs no special handling here: it is an ordinary
            // identity macro (`reassoc::macros::strict`), and this same
            // non-descent rule is exactly what leaves its contents with
            // native operator semantics instead of dispatch calls. Matching
            // it by name here would only add a name-collision hazard for no
            // benefit.
            return;
        }

        if let Expr::Const(_) = expr {
            // An inline `const { .. }` block's body is a const context,
            // just like a `const`/`static` item or a `const fn` body
            // (`is_const_context` below); `ops::*` are not `const fn`, so
            // rewriting inside it fails with E0015. Unlike a macro
            // invocation this is an ordinary `Expr` syn hands us in full,
            // but it still must not be descended into. Stable since Rust
            // 1.79, it sits in plain expression position and needs no
            // opt-in, so it is reachable from any function body.
            return;
        }

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
                    // Evaluate the RHS into a temporary *before* borrowing
                    // the place mutably, for two reasons:
                    //
                    // - Native `a += b` evaluates `b` before it evaluates
                    //   the place `a` (e.g. `v[idx()] += rhs()` calls
                    //   `rhs()` before `idx()`). Binding the place first, as
                    //   this used to, reversed that order relative to
                    //   native compound assignment.
                    // - If `#right` reads the place at all (`s += s * k`),
                    //   borrowing `&mut #left` first makes that read a
                    //   borrow-check error (E0503) even though the
                    //   equivalent native `+=` compiles fine. Evaluating
                    //   the RHS first, while the place is not yet borrowed,
                    //   avoids that.
                    //
                    // The place is still bound through a `&mut` temporary
                    // so it is evaluated exactly once; a naive `place =
                    // f(place, rhs)` rewrite would evaluate `place` twice.
                    *expr = syn::parse2(quote_spanned! {span=>
                        {
                            let __reassoc_rhs = #right;
                            let __reassoc_place = &mut #left;
                            *__reassoc_place = ::reassoc::ops::#func(*__reassoc_place, __reassoc_rhs);
                        }
                    })
                    .expect("generated compound assignment must parse");
                }
            }
        }
    }
}

/// True for a nested item whose body/initializer is a const context:
/// `const`, `static`, and `const fn`. `ops::*` are not `const fn`, so
/// rewriting inside any of these fails with E0015.
fn is_const_context(item: &syn::Item) -> bool {
    match item {
        syn::Item::Const(_) | syn::Item::Static(_) => true,
        syn::Item::Fn(item_fn) => item_fn.sig.constness.is_some(),
        _ => false,
    }
}

/// True when the item carries `#[algebraic(skip)]`.
fn has_skip_attribute(item: &syn::Item) -> bool {
    item_attrs(item).is_some_and(|attrs| {
        attrs.iter().any(|attr| {
            attr.path()
                .segments
                .last()
                .is_some_and(|s| s.ident == "algebraic")
                && attr
                    .parse_args::<syn::Ident>()
                    .map(|ident| ident == "skip")
                    .unwrap_or(false)
        })
    })
}

/// Remove `#[algebraic(skip)]` so it does not reach the compiler as an
/// unresolved attribute on a nested item.
fn strip_skip_attribute(item: &mut syn::Item) {
    if let Some(attrs) = item_attrs_mut(item) {
        attrs.retain(|attr| {
            !(attr
                .path()
                .segments
                .last()
                .is_some_and(|s| s.ident == "algebraic")
                && attr
                    .parse_args::<syn::Ident>()
                    .map(|ident| ident == "skip")
                    .unwrap_or(false))
        });
    }
}

fn item_attrs(item: &syn::Item) -> Option<&Vec<syn::Attribute>> {
    match item {
        syn::Item::Fn(f) => Some(&f.attrs),
        syn::Item::Impl(i) => Some(&i.attrs),
        syn::Item::Mod(m) => Some(&m.attrs),
        _ => None,
    }
}

fn item_attrs_mut(item: &mut syn::Item) -> Option<&mut Vec<syn::Attribute>> {
    match item {
        syn::Item::Fn(f) => Some(&mut f.attrs),
        syn::Item::Impl(i) => Some(&mut i.attrs),
        syn::Item::Mod(m) => Some(&mut m.attrs),
        _ => None,
    }
}
