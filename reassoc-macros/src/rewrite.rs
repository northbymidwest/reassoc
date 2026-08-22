//! The rewriter shared by `alg!` and `#[algebraic]`.
//!
//! Every special case below is a measured decision; the history and the
//! alternatives that were tried live in `docs/design.md`.

use proc_macro2::Span;
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::visit_mut::{self, VisitMut};
use syn::{Attribute, BinOp, Expr, UnOp};

pub struct Rewriter {
    /// Descend into closure bodies.
    pub closures: bool,
    /// Descend into nested `fn` / `impl` / `mod` items.
    pub items: bool,
}

impl Rewriter {
    /// Scope used by `alg!`: closures in, nested items out, matching
    /// `#[algebraic]`'s default.
    pub fn expression_scope() -> Self {
        Rewriter {
            closures: true,
            items: false,
        }
    }

    pub fn from_scope(scope: crate::scope::Scope) -> Self {
        Rewriter {
            closures: scope.closures,
            items: scope.items,
        }
    }
}

impl VisitMut for Rewriter {
    fn visit_expr_closure_mut(&mut self, closure: &mut syn::ExprClosure) {
        if self.closures {
            visit_mut::visit_expr_closure_mut(self, closure);
        }
    }

    // Const positions are never rewritten: `ops::*` are not `const fn`, so a
    // call there is E0015. Array-repeat and type-array lengths, const generic
    // arguments, enum discriminants, and inline `const {}` blocks are reachable
    // from any function body; `const`/`static` items and `const fn` bodies
    // through `items = true`.

    fn visit_expr_repeat_mut(&mut self, expr_repeat: &mut syn::ExprRepeat) {
        self.visit_expr_mut(&mut expr_repeat.expr);
    }

    fn visit_type_array_mut(&mut self, type_array: &mut syn::TypeArray) {
        self.visit_type_mut(&mut type_array.elem);
    }

    fn visit_generic_argument_mut(&mut self, arg: &mut syn::GenericArgument) {
        if !matches!(arg, syn::GenericArgument::Const(_)) {
            visit_mut::visit_generic_argument_mut(self, arg);
        }
    }

    fn visit_variant_mut(&mut self, variant: &mut syn::Variant) {
        self.visit_attributes_mut(&mut variant.attrs);
        self.visit_ident_mut(&mut variant.ident);
        self.visit_fields_mut(&mut variant.fields);
    }

    fn visit_item_mut(&mut self, item: &mut syn::Item) {
        if !self.items {
            return;
        }
        if let Some(attrs) = item_attrs_mut(item) {
            // A nested item with its own `#[algebraic(..)]` is governed by that
            // attribute alone; rewriting it here first would apply the outer
            // scope and silently override the inner parameters.
            if strip_skip(attrs) || attrs.iter().any(is_algebraic_attr) {
                return;
            }
        }
        let is_const = match item {
            syn::Item::Const(_) | syn::Item::Static(_) => true,
            syn::Item::Fn(f) => f.sig.constness.is_some(),
            _ => false,
        };
        if !is_const {
            visit_mut::visit_item_mut(self, item);
        }
    }

    fn visit_impl_item_mut(&mut self, item: &mut syn::ImplItem) {
        match item {
            syn::ImplItem::Const(_) => {}
            syn::ImplItem::Fn(f) => {
                if !leave_fn_alone(&mut f.attrs, f.sig.constness.is_some()) {
                    visit_mut::visit_impl_item_fn_mut(self, f);
                }
            }
            _ => visit_mut::visit_impl_item_mut(self, item),
        }
    }

    fn visit_trait_item_mut(&mut self, item: &mut syn::TraitItem) {
        match item {
            syn::TraitItem::Const(_) => {}
            syn::TraitItem::Fn(f) => {
                if !leave_fn_alone(&mut f.attrs, f.sig.constness.is_some()) {
                    visit_mut::visit_trait_item_fn_mut(self, f);
                }
            }
            _ => visit_mut::visit_trait_item_mut(self, item),
        }
    }

    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // A macro's body is an opaque token stream and is never entered; this
        // is also exactly what makes `strict!(..)` an escape hatch. An inline
        // `const {}` block is a const position.
        if matches!(expr, Expr::Macro(_) | Expr::Const(_)) {
            return;
        }

        // Children first, so nested operators are already calls by the time
        // this node is rebuilt.
        visit_mut::visit_expr_mut(self, expr);

        let Expr::Binary(binary) = expr else { return };
        let Some((name, assign)) = dispatch_fn(&binary.op) else {
            return;
        };

        // A non-float literal on either side proves this is not float
        // arithmetic (Rust never converts an integer to a float), so it stays
        // native: rustc's `arithmetic_overflow` lint keeps seeing it, and
        // integer counters and indices never enter dispatch at all.
        if is_non_float_constant(&binary.left) || is_non_float_constant(&binary.right) {
            return;
        }

        // Spanned at the operator so errors point there. The crate path is
        // respanned too; left at the call site it would anchor "required by a
        // bound introduced by this call" on the `#[algebraic]` attribute.
        let span = binary.op.span();
        let krate = crate::krate::path_spanned(span);
        let func = syn::Ident::new(name, Span::call_site());
        let left = unparen(&binary.left);
        let right = unparen(&binary.right);

        if !assign {
            *expr = syn::parse2(quote_spanned! {span=>
                #krate::ops::#func(#left, #right)
            })
            .expect("generated dispatch call must parse");
            return;
        }

        if !is_place_expr(left) {
            // Mirror rustc's E0067; letting `a + b += x` through would mutate
            // a discarded temporary.
            let err =
                syn::Error::new_spanned(left, "invalid left-hand side of compound assignment");
            *expr = syn::parse2(err.to_compile_error()).expect("compile_error! must parse");
            return;
        }

        // RHS first, bound through a `match` (native order; native temporary
        // lifetime). The scrutinee is a one-tuple because a bare struct
        // literal is not allowed as a scrutinee, and `unparen` has already
        // removed any parens the user put around it. The binding resolves at
        // the call site with a nonsense suffix, not with `Span::mixed_site()`
        // hygiene: rustc re-anchors a span from an external macro's context
        // at the invocation, so a mixed-site binding moves the caret of an
        // unsatisfied `+=` from the operator to the `#[algebraic]` attribute
        // (measured; `tests/ui/compound_assign_not_opted_in.rs` pins it). A
        // user binding of the same name is a loud error, never a misresolve.
        //
        // The place is then borrowed, for every shape: a closure-captured
        // non-`Copy` local stays `FnMut` (assigning through by name moved it
        // out), a type with only an in-place form works (by name needed `+`),
        // and a temporary behind a deref lives through the call. Native `+=`
        // on a primitive `static mut` takes no reference, and edition 2024
        // denies `&mut` on one; the allow keeps that case compiling.
        let func_assign = syn::Ident::new(&format!("{name}_assign"), Span::call_site());
        let rhs = syn::Ident::new("__reassoc_rhs_9f2c1a", span);
        *expr = syn::parse2(quote_spanned! {span=>
            match (#right,) {
                (#rhs,) => {
                    #[allow(static_mut_refs)]
                    #krate::ops::#func_assign(&mut #left, #rhs);
                }
            }
        })
        .expect("generated compound assignment must parse");
    }
}

/// The dispatch function for an arithmetic operator, and whether the operator
/// is the compound-assignment form. `None` for everything the rewriter leaves
/// alone: comparison, logical, bitwise, shifts.
fn dispatch_fn(op: &BinOp) -> Option<(&'static str, bool)> {
    Some(match op {
        BinOp::Add(_) => ("add", false),
        BinOp::Sub(_) => ("sub", false),
        BinOp::Mul(_) => ("mul", false),
        BinOp::Div(_) => ("div", false),
        BinOp::Rem(_) => ("rem", false),
        BinOp::AddAssign(_) => ("add", true),
        BinOp::SubAssign(_) => ("sub", true),
        BinOp::MulAssign(_) => ("mul", true),
        BinOp::DivAssign(_) => ("div", true),
        BinOp::RemAssign(_) => ("rem", true),
        _ => return None,
    })
}

/// Strips invisible groups — what a `macro_rules!` `$e:expr` arrives in — and
/// then exactly one layer of parentheses. One, because that layer is the one
/// the call's own delimiters make redundant; any further layers were already
/// redundant in the source and are left for `unused_parens` to report.
fn unparen(expr: &Expr) -> &Expr {
    match ungroup(expr) {
        Expr::Paren(inner) => ungroup(&inner.expr),
        expr => expr,
    }
}

fn ungroup(mut expr: &Expr) -> &Expr {
    while let Expr::Group(inner) = expr {
        expr = &inner.expr;
    }
    expr
}

/// A plausible left-hand side for compound assignment. Deliberately permissive:
/// a macro may expand to a place, and a false negative here merely falls
/// through to rustc's own check, while a false positive would reject valid
/// code.
fn is_place_expr(expr: &Expr) -> bool {
    match ungroup(expr) {
        Expr::Path(_) | Expr::Field(_) | Expr::Index(_) | Expr::Macro(_) => true,
        Expr::Unary(unary) => matches!(unary.op, UnOp::Deref(_)),
        Expr::Paren(inner) => is_place_expr(&inner.expr),
        _ => false,
    }
}

/// A compile-time proof that this is not float arithmetic: any non-float
/// literal, a cast to an integer type, a minus over one, or arithmetic over
/// such. A denylist rather than an integer allowlist, so byte literals (which
/// overflow like `u8`) and any literal kind added later are exempt from
/// rewriting by default. `2f64` has no decimal point and reaches syn as
/// `Lit::Int`, hence the suffix check. Every paren layer is looked through
/// here — `((200u8)) + ((100u8))` is still constant — where the emitter strips
/// only the one its own delimiters make redundant.
fn is_non_float_constant(expr: &Expr) -> bool {
    match unparen_all(expr) {
        Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Float(_) => false,
            syn::Lit::Int(int) => !is_float_suffix(int.suffix()),
            _ => true,
        },
        Expr::Cast(cast) => is_integer_type(&cast.ty),
        Expr::Unary(unary) => {
            matches!(unary.op, UnOp::Neg(_)) && is_non_float_constant(&unary.expr)
        }
        Expr::Binary(binary) => {
            dispatch_fn(&binary.op).is_some_and(|(_, assign)| !assign)
                && is_non_float_constant(&binary.left)
                && is_non_float_constant(&binary.right)
        }
        _ => false,
    }
}

fn unparen_all(mut expr: &Expr) -> &Expr {
    loop {
        expr = match expr {
            Expr::Group(inner) => &inner.expr,
            Expr::Paren(inner) => &inner.expr,
            expr => return expr,
        };
    }
}

/// A primitive integer type, named plainly. A cast to one proves the operand
/// is an integer exactly as an integer literal does; `as f32` proves nothing
/// and a path that is not one of these (an alias, say) is not assumed either.
fn is_integer_type(ty: &syn::Type) -> bool {
    const INTS: [&str; 12] = [
        "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
    ];
    let mut ty = ty;
    loop {
        ty = match ty {
            syn::Type::Group(inner) => &inner.elem,
            syn::Type::Paren(inner) => &inner.elem,
            _ => break,
        };
    }
    match ty {
        syn::Type::Path(path) if path.qself.is_none() => path
            .path
            .get_ident()
            .is_some_and(|i| INTS.iter().any(|n| i == n)),
        _ => false,
    }
}

/// `f32`, `f64`, and any future `f<N>`, matched by shape.
fn is_float_suffix(suffix: &str) -> bool {
    suffix
        .strip_prefix('f')
        .is_some_and(|width| !width.is_empty() && width.bytes().all(|b| b.is_ascii_digit()))
}

fn is_algebraic_attr(attr: &Attribute) -> bool {
    attr.path()
        .segments
        .last()
        .is_some_and(|s| s.ident == "algebraic")
}

fn is_skip_attr(attr: &Attribute) -> bool {
    is_algebraic_attr(attr) && attr.parse_args::<syn::Ident>().is_ok_and(|i| i == "skip")
}

/// Removes `#[algebraic(skip)]` so it never reaches rustc as an unknown
/// attribute; returns whether there was one.
fn strip_skip(attrs: &mut Vec<Attribute>) -> bool {
    let before = attrs.len();
    attrs.retain(|attr| !is_skip_attr(attr));
    attrs.len() != before
}

/// A nested `fn` the rewriter must not enter: `const fn` (a const position),
/// `#[algebraic(skip)]`, or one carrying its own `#[algebraic(..)]`.
fn leave_fn_alone(attrs: &mut Vec<Attribute>, is_const: bool) -> bool {
    strip_skip(attrs) || is_const || attrs.iter().any(is_algebraic_attr)
}

fn item_attrs_mut(item: &mut syn::Item) -> Option<&mut Vec<Attribute>> {
    match item {
        syn::Item::Fn(f) => Some(&mut f.attrs),
        syn::Item::Impl(i) => Some(&mut i.attrs),
        syn::Item::Mod(m) => Some(&mut m.attrs),
        _ => None,
    }
}
