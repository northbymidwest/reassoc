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
        if has_algebraic_attribute(item) {
            // The item carries its own `#[algebraic(..)]`. Leave it entirely
            // alone and let that attribute govern it: rewriting here first
            // would apply the OUTER scope to it, and the inner attribute
            // would then run over already-rewritten code — silently ignoring
            // its own parameters. `#[algebraic(closures = false)]` on a
            // nested fn used to be overridden this way.
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

    fn visit_impl_item_mut(&mut self, item: &mut syn::ImplItem) {
        // `Item`-level checks miss everything inside an `impl` block, because
        // an associated const or a method is an `ImplItem`, not an `Item`.
        // Associated consts and `const fn` methods are const contexts (E0015),
        // and `#[algebraic(skip)]` on a method used to be silently ignored.
        match item {
            syn::ImplItem::Const(_) => return,
            syn::ImplItem::Fn(f)
                if f.sig.constness.is_some()
                    || attrs_skip(&f.attrs)
                    || attrs_algebraic(&f.attrs) =>
            {
                strip_skip_attrs(&mut f.attrs);
                return;
            }
            _ => {}
        }
        visit_mut::visit_impl_item_mut(self, item);
    }

    fn visit_trait_item_mut(&mut self, item: &mut syn::TraitItem) {
        match item {
            syn::TraitItem::Const(_) => return,
            syn::TraitItem::Fn(f)
                if f.sig.constness.is_some()
                    || attrs_skip(&f.attrs)
                    || attrs_algebraic(&f.attrs) =>
            {
                strip_skip_attrs(&mut f.attrs);
                return;
            }
            _ => {}
        }
        visit_mut::visit_trait_item_mut(self, item);
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

            if is_non_float_literal(&binary.left) && is_non_float_literal(&binary.right) {
                // Every literal EXCEPT a float one — a denylist, not an
                // allowlist, and deliberately so.
                //
                // `arithmetic_overflow` and `unconditional_panic` are
                // deny-by-default and fire on compile-time-evaluable INTEGER
                // arithmetic. Rewriting to a call hides the constants, so
                // `255u8 + 1` would compile and wrap to 0 instead of being
                // rejected. Integers dispatch to plain operators anyway, so
                // skipping them forfeits nothing.
                //
                // Floats are a different matter. Neither lint applies to them
                // (`1.0/0.0` is inf, not a panic), and the algebraic operators
                // are meaningfully non-deterministic even on constants — that
                // freedom is the whole point of them, and the reference
                // explicitly declines to guarantee anything about the returned
                // value. So `2.0 * 3.0` is still rewritten.
                //
                // Phrasing it as "not a float literal" rather than "is an
                // integer literal" matters: byte literals are `u8` and overflow
                // exactly like integers (`b'\xff' + b'\x01'`), and an
                // allowlist silently missed them. Any literal kind added later
                // is likewise excluded by default, which fails safe — the worst
                // case is forfeiting dispatch on a constant expression, rather
                // than hiding a deny-by-default lint.
                return;
            }

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
                    // The generated bindings carry a nonsense suffix because
                    // a stable proc macro has no def-site hygiene: these
                    // identifiers resolve at the call site and would shadow a
                    // user binding of the same name. The suffix makes that
                    // collision implausible rather than impossible.
                    //
                    // `match` rather than `let`: a `let` drops the RHS's
                    // temporaries at the end of its own statement, i.e.
                    // before the place is even evaluated, whereas native
                    // `+=` keeps them alive to the end of the full
                    // statement. Binding through a `match` scrutinee
                    // extends them across the whole expansion, restoring
                    // native drop timing for a Drop-carrying RHS.
                    *expr = syn::parse2(quote_spanned! {span=>
                        match #right {
                            __reassoc_rhs_9f2c1a => {
                                let __reassoc_place_9f2c1a = &mut #left;
                                *__reassoc_place_9f2c1a =
                                    ::reassoc::ops::#func(*__reassoc_place_9f2c1a, __reassoc_rhs_9f2c1a);
                            }
                        }
                    })
                    .expect("generated compound assignment must parse");
                }
            }
        }
    }
}

/// True for any literal that is not a float literal, ignoring parentheses and
/// a leading unary minus.
///
/// Stated as a denylist on purpose. The one literal kind that must keep being
/// rewritten is the float one, because algebraic operators are meaningfully
/// non-deterministic even on constants. Everything else either cannot do
/// arithmetic at all, or is integer-like and must stay visible to rustc's
/// deny-by-default `arithmetic_overflow` / `unconditional_panic` lints —
/// byte literals being the case an integer-only allowlist missed.
fn is_non_float_literal(expr: &Expr) -> bool {
    match unparen(expr) {
        Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Float(_) => false,
            // `2f64` and `2f32` are float literals that syn reports as
            // `Lit::Int`, because they have no decimal point — only the
            // suffix distinguishes them. Missing that would silently skip
            // dispatch for a perfectly ordinary spelling of a float constant.
            syn::Lit::Int(int) => !is_float_suffix(int.suffix()),
            _ => true,
        },
        Expr::Unary(unary) => {
            matches!(unary.op, syn::UnOp::Neg(_)) && is_non_float_literal(&unary.expr)
        }
        _ => false,
    }
}

/// `f32`, `f64`, and any future `f<N>` — matched by shape rather than by a
/// fixed list, so a new float width does not silently fall through to the
/// integer path.
fn is_float_suffix(suffix: &str) -> bool {
    suffix
        .strip_prefix('f')
        .is_some_and(|width| !width.is_empty() && width.bytes().all(|b| b.is_ascii_digit()))
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
fn is_algebraic_attr(attr: &syn::Attribute) -> bool {
    attr.path()
        .segments
        .last()
        .is_some_and(|s| s.ident == "algebraic")
}

fn attrs_skip(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        is_algebraic_attr(attr)
            && attr
                .parse_args::<syn::Ident>()
                .map(|ident| ident == "skip")
                .unwrap_or(false)
    })
}

/// Any `#[algebraic(..)]`, including a bare one and any parameterised form.
fn attrs_algebraic(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(is_algebraic_attr)
}

fn strip_skip_attrs(attrs: &mut Vec<syn::Attribute>) {
    attrs.retain(|attr| !(is_algebraic_attr(attr) && attrs_skip(core::slice::from_ref(attr))));
}

fn has_skip_attribute(item: &syn::Item) -> bool {
    item_attrs(item).is_some_and(|attrs| attrs_skip(attrs))
}

fn has_algebraic_attribute(item: &syn::Item) -> bool {
    item_attrs(item).is_some_and(|attrs| attrs_algebraic(attrs))
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
