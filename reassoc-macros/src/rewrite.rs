//! The rewriter shared by `alg!` and `#[algebraic]`.
//!
//! Every special case below is a measured decision; the history and the
//! alternatives that were tried live in `docs/design.md`.

use proc_macro2::Span;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit_mut::{self, VisitMut};
use syn::{Attribute, BinOp, Expr, UnOp};

use crate::build;

pub struct Rewriter {
    /// Descend into closure bodies.
    pub closures: bool,
    /// Enter the arguments of the std macros whose arguments are expressions.
    pub macros: bool,
    /// Errors to report alongside the rewritten item: a `const fn` whose
    /// arithmetic would have been rewritten.
    pub errors: Vec<syn::Error>,
    /// The facade crate's name in the consumer, looked up once per
    /// expansion: with `resolve-crate-name` the lookup reads the manifest,
    /// and it used to run once per operator.
    krate: String,
    /// Operators rewritten so far (binary and compound), for `REASSOC_TRACE`.
    pub ops: usize,
    /// `ops::fast::*` instead of `ops::*`: the `unsafe_fast` scope.
    pub fast: bool,
    /// Set while visiting the parts of a `const fn` body that a `const fn`
    /// actually evaluates. An operator met there cannot become a call
    /// (`ops::*` are not `const fn`), so it is left as written and recorded
    /// in `const_arith` instead. Cleared again on entry to a nested item or
    /// a closure body, which are ordinary runtime code.
    const_context: bool,
    /// Whether the `const fn` currently being visited has an operator of its
    /// own that would have been rewritten. Saved and restored per `const fn`,
    /// so a nested one's arithmetic is reported against it and not against
    /// the function that holds it.
    const_arith: bool,
}

impl Rewriter {
    /// Scope used by `alg!`: closures and nested items in, matching
    /// `#[algebraic]`'s default.
    pub fn expression_scope(fast: bool) -> Self {
        Rewriter {
            closures: true,
            macros: true,
            errors: Vec::new(),
            krate: crate::krate::name(),
            ops: 0,
            fast,
            const_context: false,
            const_arith: false,
        }
    }

    pub fn from_scope(scope: crate::scope::Scope) -> Self {
        Rewriter {
            closures: scope.closures,
            macros: scope.macros,
            errors: Vec::new(),
            krate: crate::krate::name(),
            ops: 0,
            fast: scope.fast,
            const_context: false,
            const_arith: false,
        }
    }

    /// `::reassoc::ops::<func>`, every token at `span` except the function
    /// name, which keeps the call site's.
    fn ops_fn(&self, span: Span, func: &str) -> Expr {
        let mut segments = vec![
            syn::Ident::new(&self.krate, span),
            syn::Ident::new("ops", span),
        ];
        // `unit` is shared: it wraps the compound statement in both modes.
        if self.fast && func != "unit" {
            segments.push(syn::Ident::new("fast", span));
        }
        segments.push(syn::Ident::new(func, Span::call_site()));
        build::path(span, segments)
    }

    /// A `const fn` met in an algebraic scope. `ops::*` are not `const fn`,
    /// so its own arithmetic cannot be rewritten; a `const fn` with none is
    /// skipped in silence, one with some is an error naming the way out,
    /// never a member left strict without a word.
    ///
    /// The body is entered rather than cloned and compared, because a
    /// `const fn` body is not one indivisible region: it is const context
    /// with runtime islands in it. A nested `fn`, `impl`, `mod` or `trait`,
    /// and a closure body, are ordinary runtime code, are rewritten like any
    /// other, and are none of this function's business. `const_context` says
    /// which of the two the visitor is in; `visit_expr_mut` records rather
    /// than rewrites while it is set, so the literal rule, `strict!` and
    /// const positions all still count exactly as they do elsewhere.
    ///
    /// Both flags are saved and restored, so a `const fn` nested in this one
    /// is reported against itself and does not also condemn its parent.
    fn const_fn(
        &mut self,
        const_token: syn::token::Const,
        name: &syn::Ident,
        body: &mut syn::Block,
    ) {
        crate::trace::record("const fn", name.span(), &name.to_string(), 0);
        let outer_context = core::mem::replace(&mut self.const_context, true);
        let outer_arith = core::mem::replace(&mut self.const_arith, false);
        self.visit_block_mut(body);
        self.const_context = outer_context;
        if core::mem::replace(&mut self.const_arith, outer_arith) {
            self.errors.push(syn::Error::new_spanned(
                const_token,
                "`#[algebraic]` cannot rewrite the arithmetic in this `const fn`: the dispatch \
                 functions it would call (`reassoc::ops::*`) are not `const fn`. Mark it \
                 `#[algebraic(skip)]` to leave it as written, or drop `const`",
            ));
        }
    }

    /// Visit something that is runtime code however it is nested: a closure
    /// body, or the inside of an item. `const_context` is what separates a
    /// `const fn`'s own expressions from the ordinary code written inside it.
    fn in_runtime_context(&mut self, visit: impl FnOnce(&mut Self)) {
        let outer = core::mem::replace(&mut self.const_context, false);
        visit(self);
        self.const_context = outer;
    }
}

impl VisitMut for Rewriter {
    // Every function body entered is one `REASSOC_TRACE` line, with the
    // number of operators rewritten inside it (nested items included).
    fn visit_item_fn_mut(&mut self, f: &mut syn::ItemFn) {
        let before = self.ops;
        visit_mut::visit_item_fn_mut(self, f);
        crate::trace::record(
            "fn",
            f.sig.ident.span(),
            &f.sig.ident.to_string(),
            self.ops - before,
        );
    }

    fn visit_impl_item_fn_mut(&mut self, f: &mut syn::ImplItemFn) {
        let before = self.ops;
        visit_mut::visit_impl_item_fn_mut(self, f);
        crate::trace::record(
            "fn",
            f.sig.ident.span(),
            &f.sig.ident.to_string(),
            self.ops - before,
        );
    }

    fn visit_trait_item_fn_mut(&mut self, f: &mut syn::TraitItemFn) {
        let before = self.ops;
        visit_mut::visit_trait_item_fn_mut(self, f);
        if f.default.is_some() {
            crate::trace::record(
                "fn",
                f.sig.ident.span(),
                &f.sig.ident.to_string(),
                self.ops - before,
            );
        }
    }

    fn visit_expr_closure_mut(&mut self, closure: &mut syn::ExprClosure) {
        if self.closures {
            // A closure body runs when the closure is called, which a `const
            // fn` cannot do, so it is runtime code even inside one.
            self.in_runtime_context(|this| visit_mut::visit_expr_closure_mut(this, closure));
        }
    }

    // Const positions are never rewritten: `ops::*` are not `const fn`, so a
    // call there is E0015. Array-repeat and type-array lengths, const generic
    // arguments, enum discriminants, inline `const {}` blocks, and nested
    // `const`/`static` items and `const fn` bodies are all reachable from any
    // function body.

    fn visit_expr_repeat_mut(&mut self, expr_repeat: &mut syn::ExprRepeat) {
        self.visit_expr_mut(&mut expr_repeat.expr);
    }

    fn visit_type_array_mut(&mut self, type_array: &mut syn::TypeArray) {
        self.visit_type_mut(&mut type_array.elem);
    }

    fn visit_const_param_mut(&mut self, param: &mut syn::ConstParam) {
        // `const N: usize = { A * B }`: the default is evaluated at compile
        // time. The type may carry an array length, already handled.
        self.visit_type_mut(&mut param.ty);
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
        match item_attrs_mut(item).map(claim) {
            Some(Claim::Theirs) => return,
            Some(Claim::Skipped) => return StripSkip.visit_item_mut(item),
            _ => {}
        }
        match item {
            syn::Item::Const(_) | syn::Item::Static(_) => {}
            syn::Item::Fn(f) if f.sig.constness.is_some() && cfg!(not(feature = "const-fn")) => {
                self.const_fn(f.sig.constness.unwrap(), &f.sig.ident, &mut f.block);
            }
            // Every other item begins a scope of its own, and an ordinary
            // `fn`, `impl`, `mod` or `trait` written inside a `const fn` body
            // is runtime code. This is the one way into a container's
            // members, so clearing here covers `visit_impl_item_mut` and
            // `visit_trait_item_mut` too.
            _ => self.in_runtime_context(|this| visit_mut::visit_item_mut(this, item)),
        }
    }

    fn visit_impl_item_mut(&mut self, item: &mut syn::ImplItem) {
        match impl_item_attrs_mut(item).map(claim) {
            Some(Claim::Theirs) => return,
            Some(Claim::Skipped) => return StripSkip.visit_impl_item_mut(item),
            _ => {}
        }
        match item {
            syn::ImplItem::Const(_) => {}
            syn::ImplItem::Fn(f) => match f.sig.constness {
                Some(c) if cfg!(not(feature = "const-fn")) => {
                    self.const_fn(c, &f.sig.ident, &mut f.block)
                }
                _ => self.visit_impl_item_fn_mut(f),
            },
            _ => visit_mut::visit_impl_item_mut(self, item),
        }
    }

    fn visit_trait_item_mut(&mut self, item: &mut syn::TraitItem) {
        match trait_item_attrs_mut(item).map(claim) {
            Some(Claim::Theirs) => return,
            Some(Claim::Skipped) => return StripSkip.visit_trait_item_mut(item),
            _ => {}
        }
        match item {
            syn::TraitItem::Const(_) => {}
            syn::TraitItem::Fn(f) => {
                // A required method has no body: nothing to rewrite, and
                // nothing to warn about.
                match (f.sig.constness, &mut f.default) {
                    (Some(c), Some(body)) if cfg!(not(feature = "const-fn")) => {
                        self.const_fn(c, &f.sig.ident, body)
                    }
                    _ => self.visit_trait_item_fn_mut(f),
                }
            }
            _ => visit_mut::visit_trait_item_mut(self, item),
        }
    }

    fn visit_stmt_mut(&mut self, stmt: &mut syn::Stmt) {
        match stmt {
            syn::Stmt::Macro(m) => self.macro_arguments(&mut m.mac),
            _ => visit_mut::visit_stmt_mut(self, stmt),
        }
    }

    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // A macro's body is an opaque token stream and is never entered,
        // which is exactly what makes `strict!(..)` an escape hatch, except
        // for the std macros whose arguments are known to be expressions. An
        // inline `const {}` block is a const position.
        match expr {
            Expr::Macro(m) => return self.macro_arguments(&mut m.mac),
            Expr::Const(_) => return,
            _ => {}
        }

        // Children first, so nested operators are already calls by the time
        // this node is rebuilt.
        visit_mut::visit_expr_mut(self, expr);
        reparen_tight_positions(expr);

        let Expr::Binary(binary) = expr else { return };
        let Some(func) = dispatch_fn(&binary.op) else {
            return;
        };

        // A non-float literal on either side proves this is not float
        // arithmetic (Rust never converts an integer to a float), so it stays
        // native: rustc's `arithmetic_overflow` lint keeps seeing it, and
        // integer counters and indices never enter dispatch at all.
        if is_non_float_constant(&binary.left) || is_non_float_constant(&binary.right) {
            return;
        }

        // Const context: `ops::*` are not `const fn`, so this operator stays
        // as written. `const_fn` turns the flag into one error for the whole
        // function, the remedy (`skip`, or dropping `const`) being per
        // function rather than per operator.
        if self.const_context {
            self.const_arith = true;
            return;
        }

        // Spanned at the operator so errors point there; the crate path
        // too, since left at the call site it would anchor "required by a
        // bound introduced by this call" on the `#[algebraic]` attribute.
        //
        // The replacement is built as syntax tree, not as tokens re-parsed:
        // the operands are moved out of the old node and into the new one,
        // never re-printed. With `quote!` + `parse2` every nesting level
        // re-printed and re-parsed its whole subtree, and the proc macro ran
        // that unoptimized; `scripts/compile-bench.sh` has the numbers.
        let span = binary.op.span();
        let Expr::Binary(binary) = core::mem::replace(expr, Expr::PLACEHOLDER) else {
            unreachable!("matched `Expr::Binary` above");
        };
        let left = unparen(*binary.left);
        let right = unparen(*binary.right);
        // `binary.attrs` is dropped with the node, and is empty. syn descends
        // the left spine when it places attributes, so they land on the
        // leftmost leaf and travel into the call with it, which is where rustc
        // reads them too: `#[allow(..)] a + b` becomes
        // `ops::add(#[allow(..)] a, b)`, the same attribute on the same
        // expression. Measured over every shape and every entry point rather
        // than read off syn's source, and the measurement is a test, so a syn
        // release that changes it fails rather than silently losing the
        // attributes: `tests/rewrite.rs::syn_never_puts_attributes_on_a_binary_node`.
        // Noted because the empty vectors below otherwise read like a leak.

        self.ops += 1;
        let assign = match func {
            Dispatch::Binary(name) => {
                *expr = build::call(span, self.ops_fn(span, name), [left, right], Vec::new());
                return;
            }
            Dispatch::Compound(assign) => assign,
        };

        if !is_place_expr(&left) {
            // Mirror rustc's E0067; letting `a + b += x` through would mutate
            // a discarded temporary.
            let err =
                syn::Error::new_spanned(&left, "invalid left-hand side of compound assignment");
            *expr = Expr::Verbatim(err.to_compile_error());
            return;
        }

        // RHS first, bound through a `match` (native order; native temporary
        // lifetime):
        //
        //     match (rhs,) { (__r,) => { ops::add_assign(&mut place, __r); } }
        //
        // The scrutinee is a one-tuple because a bare struct literal is not
        // allowed as a scrutinee, and `unparen` has already removed any parens
        // the user put around it. The binding resolves at the call site with a
        // nonsense suffix, not with `Span::mixed_site()` hygiene: rustc
        // re-anchors a span from an external macro's context at the
        // invocation, so a mixed-site binding moves the caret of an
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
        //
        // The whole `match` is then passed to `ops::unit`, an identity on
        // `()`, so that the statement is a *call*, not a block-like
        // expression. Left bare, the user's `;` after it trips clippy's
        // pedantic `unnecessary_semicolon`, and dropping that `;` instead
        // trips `semicolon_if_nothing_returned` whenever the statement is the
        // last of a block (the tokens sit at the operator's span, so clippy
        // reads the snippet `+=` and does not see a block); the wrapper is
        // clean under both and leaves every token of the user's alone.
        let rhs = syn::Ident::new("__reassoc_rhs_9f2c1a", span);
        let assign = build::call(
            span,
            self.ops_fn(span, assign),
            [build::ref_mut(span, left), build::ident(rhs.clone())],
            vec![build::allow(span, "static_mut_refs")],
        );
        let matched = build::match1(
            span,
            build::tuple1(span, right),
            build::pat_tuple1(span, build::bind(rhs)),
            build::block1(span, assign),
        );
        *expr = build::call(span, self.ops_fn(span, "unit"), [matched], Vec::new());
    }
}

impl Rewriter {
    /// Enters the arguments of a listed std macro. Only a macro whose last
    /// path segment is on the list, and only when its tokens parse as
    /// comma-separated expressions (or `vec!`'s `elem; len`, or `matches!`'s
    /// `expr, pattern`); anything else (a user macro, a listed name carrying
    /// a different grammar, `strict!`) is left untouched, tokens and all.
    fn macro_arguments(&mut self, mac: &mut syn::Macro) {
        if !self.macros {
            return;
        }
        // `matches!(expr, pattern)`: the scrutinee is an expression, the rest
        // is a pattern with an optional guard, left as written.
        if mac
            .path
            .segments
            .last()
            .is_some_and(|s| s.ident == "matches")
        {
            if let Ok(mut m) = syn::parse2::<MatchesArgs>(mac.tokens.clone()) {
                self.visit_expr_mut(&mut m.scrutinee);
                let MatchesArgs {
                    scrutinee,
                    comma,
                    rest,
                } = m;
                mac.tokens = quote::quote!(#scrutinee #comma #rest);
            }
            return;
        }
        if !is_listed_macro(&mac.path) {
            return;
        }
        if mac.path.segments.last().is_some_and(|s| s.ident == "vec")
            && let Ok(mut repeat) = syn::parse2::<VecRepeat>(mac.tokens.clone())
        {
            self.visit_expr_mut(&mut repeat.elem);
            self.visit_expr_mut(&mut repeat.len);
            let VecRepeat { elem, semi, len } = repeat;
            mac.tokens = quote::quote!(#elem #semi #len);
            return;
        }
        let parser = syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated;
        if let Ok(mut args) = syn::parse::Parser::parse2(parser, mac.tokens.clone()) {
            for arg in args.iter_mut() {
                self.visit_expr_mut(arg);
            }
            mac.tokens = args.to_token_stream();
        }
    }
}

/// `matches!(scrutinee, pattern [if guard])`.
struct MatchesArgs {
    scrutinee: Expr,
    comma: syn::Token![,],
    rest: proc_macro2::TokenStream,
}

impl syn::parse::Parse for MatchesArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(MatchesArgs {
            scrutinee: input.parse()?,
            comma: input.parse()?,
            rest: input.parse()?,
        })
    }
}

/// `vec![elem; len]`.
struct VecRepeat {
    elem: Expr,
    semi: syn::Token![;],
    len: Expr,
}

impl syn::parse::Parse for VecRepeat {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(VecRepeat {
            elem: input.parse()?,
            semi: input.parse()?,
            len: input.parse()?,
        })
    }
}

/// The std macros whose arguments are expressions separated by commas (format
/// strings being string-literal expressions, and `name = value` an assignment
/// expression that re-emits unchanged), matched on the last path segment so
/// `std::println!` counts. `strict!` is deliberately not here, nor is anything
/// whose arguments are token soup (`cfg!`, `stringify!`, `concat!`).
/// `matches!` is handled apart: its first argument is an expression, the rest
/// a pattern.
const LISTED_MACROS: [&str; 20] = [
    "assert",
    "assert_eq",
    "assert_ne",
    "debug_assert",
    "debug_assert_eq",
    "debug_assert_ne",
    "panic",
    "unreachable",
    "todo",
    "unimplemented",
    "print",
    "println",
    "eprint",
    "eprintln",
    "format",
    "format_args",
    "write",
    "writeln",
    "dbg",
    "vec",
];

fn is_listed_macro(path: &syn::Path) -> bool {
    path.segments
        .last()
        .is_some_and(|s| LISTED_MACROS.iter().any(|m| s.ident == m))
}

/// The dispatch function for an arithmetic operator: `ops::add` for `+`,
/// `ops::add_assign` for `+=`. `None` for everything the rewriter leaves
/// alone: comparison, logical, bitwise, shifts.
#[derive(Clone, Copy)]
enum Dispatch {
    Binary(&'static str),
    Compound(&'static str),
}

fn dispatch_fn(op: &BinOp) -> Option<Dispatch> {
    use Dispatch::{Binary, Compound};
    Some(match op {
        BinOp::Add(_) => Binary("add"),
        BinOp::Sub(_) => Binary("sub"),
        BinOp::Mul(_) => Binary("mul"),
        BinOp::Div(_) => Binary("div"),
        BinOp::Rem(_) => Binary("rem"),
        BinOp::AddAssign(_) => Compound("add_assign"),
        BinOp::SubAssign(_) => Compound("sub_assign"),
        BinOp::MulAssign(_) => Compound("mul_assign"),
        BinOp::DivAssign(_) => Compound("div_assign"),
        BinOp::RemAssign(_) => Compound("rem_assign"),
        _ => return None,
    })
}

/// A `$e:expr` fragment arrives in an invisible group, and rustc does not
/// honour that grouping once a proc macro has re-emitted the tokens: a
/// closure passed as `$call` and invoked as `$call(x)` reads back as
/// `|..| body(x)`. Any attribute macro on such a function breaks it; this one
/// gives a grouped low-precedence expression real parentheses wherever the
/// position binds tighter (callee, receiver, base of a field, index, `?` or
/// `.await`, operand of a cast, unary or `&`). Nothing else is touched: the
/// operands the rewriter itself consumes go through `unparen`.
///
/// `&mut $e` and `&raw const $e` need no arm of their own: both require a
/// place expression, and every place binds tighter than `&` already.
fn reparen_tight_positions(expr: &mut Expr) {
    let slot = match expr {
        Expr::Call(c) => &mut *c.func,
        Expr::MethodCall(m) => &mut *m.receiver,
        Expr::Field(f) => &mut *f.base,
        Expr::Index(i) => &mut *i.expr,
        Expr::Try(t) => &mut *t.expr,
        Expr::Await(a) => &mut *a.base,
        Expr::Cast(c) => &mut *c.expr,
        Expr::Unary(u) => &mut *u.expr,
        Expr::Reference(r) => &mut *r.expr,
        _ => return,
    };
    if !matches!(slot, Expr::Group(_)) {
        return;
    }
    let inner = ungroup(core::mem::replace(slot, Expr::PLACEHOLDER));
    let low = matches!(
        inner,
        Expr::Closure(_)
            | Expr::Binary(_)
            | Expr::Unary(_)
            | Expr::Cast(_)
            | Expr::Range(_)
            | Expr::Assign(_)
            | Expr::Let(_)
            | Expr::Return(_)
            | Expr::Break(_)
            | Expr::Continue(_)
            | Expr::Yield(_)
            | Expr::Reference(_)
            | Expr::RawAddr(_)
    );
    *slot = if low {
        Expr::Paren(syn::ExprParen {
            attrs: Vec::new(),
            paren_token: syn::token::Paren::default(),
            expr: Box::new(inner),
        })
    } else {
        inner
    };
}

/// Strips invisible groups, what a `macro_rules!` `$e:expr` arrives in, and
/// then exactly one layer of parentheses. One, because that layer is the one
/// the call's own delimiters make redundant; any further layers were already
/// redundant in the source and are left for `unused_parens` to report. By
/// value: the operand is moved into the replacement, not copied.
///
/// A layer carrying attributes is kept instead. `x * #[allow(..)] (y + z)`
/// puts them on the parentheses, and dropping the layer would drop them in
/// silence; they cannot be moved onto the expression inside, syn's
/// `replace_attrs` being private, and enumerating every `Expr` variant to do
/// it by hand is a poor trade for a shape only `stmt_expr_attributes`
/// (nightly) can write. Keeping the layer costs the parentheses the user
/// wrote, which `unused_parens` may then call redundant; a warning it does
/// not deserve beats an attribute that vanishes.
fn unparen(expr: Expr) -> Expr {
    let expr = ungroup(expr);
    match expr {
        Expr::Paren(inner) if inner.attrs.is_empty() => ungroup(*inner.expr),
        expr => expr,
    }
}

fn ungroup(mut expr: Expr) -> Expr {
    while let Expr::Group(inner) = expr {
        if !inner.attrs.is_empty() {
            return Expr::Group(inner);
        }
        expr = *inner.expr;
    }
    expr
}

/// A plausible left-hand side for compound assignment. Deliberately permissive:
/// a macro may expand to a place, and a false negative here merely falls
/// through to rustc's own check, while a false positive would reject valid
/// code.
fn is_place_expr(expr: &Expr) -> bool {
    let mut expr = expr;
    while let Expr::Group(inner) = expr {
        expr = &inner.expr;
    }
    match expr {
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
/// here, so `((200u8)) + ((100u8))` is still constant, where the emitter strips
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
            matches!(dispatch_fn(&binary.op), Some(Dispatch::Binary(_)))
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
        // The `qself` guard cannot be observed from stable Rust and is kept
        // deliberately. `<S as T>::u8` is the only qualified form rustc
        // accepts, and its path carries the trait segment too, so `get_ident`
        // already answers `None`; the single-segment `<S>::u8` that would
        // reach `INTS` is rejected as an ambiguous associated type (E0223).
        // If inherent associated types ever stabilise it becomes writable,
        // and an associated type named `u8` would otherwise be read as the
        // primitive. `scripts/mutants.sh` reports this guard as a survivor
        // for that reason: it is equivalent today, not untested.
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

pub fn is_algebraic_attr(attr: &Attribute) -> bool {
    attr.path()
        .segments
        .last()
        .is_some_and(|s| s.ident == "algebraic")
}

fn is_skip_attr(attr: &Attribute) -> bool {
    is_algebraic_attr(attr) && attr.parse_args::<syn::Ident>().is_ok_and(|i| i == "skip")
}

/// `#[algebraic(..)]` other than `skip`: the item is governed by that
/// attribute's own expansion.
fn is_own_algebraic_attr(attr: &Attribute) -> bool {
    is_algebraic_attr(attr) && !is_skip_attr(attr)
}

/// Removes `#[algebraic(skip)]` so it never reaches rustc: inside a `mod` the
/// attribute is not even in scope; returns whether there was one.
fn strip_skip(attrs: &mut Vec<Attribute>) -> bool {
    let before = attrs.len();
    attrs.retain(|attr| !is_skip_attr(attr));
    attrs.len() != before
}

/// Who handles a nested item, read off its attributes.
enum Claim {
    /// No `#[algebraic(..)]` of its own: this visitor enters it.
    Ours,
    /// Carries its own `#[algebraic(..)]`, which governs it alone (rewriting
    /// it here first would apply the outer scope and silently override the
    /// inner parameters), and whose expansion strips its own nested `skip`s.
    Theirs,
    /// Marked `#[algebraic(skip)]` (now removed): not entered, and every
    /// `skip` inside it must be stripped by the caller, since nothing else
    /// will see them.
    Skipped,
}

fn claim(attrs: &mut Vec<Attribute>) -> Claim {
    if attrs.iter().any(is_own_algebraic_attr) {
        Claim::Theirs
    } else if strip_skip(attrs) {
        Claim::Skipped
    } else {
        Claim::Ours
    }
}

/// Removes `#[algebraic(skip)]` from every item inside a skipped item, which
/// the rewriter does not enter and so would otherwise leave for rustc to meet,
/// and inside a `mod` the attribute is not even in scope. Stops at items that
/// carry their own `#[algebraic(..)]`, whose expansion does the same for them.
struct StripSkip;

impl VisitMut for StripSkip {
    fn visit_item_mut(&mut self, item: &mut syn::Item) {
        if let Some(attrs) = item_attrs_mut(item) {
            if attrs.iter().any(is_own_algebraic_attr) {
                return;
            }
            strip_skip(attrs);
        }
        visit_mut::visit_item_mut(self, item);
    }

    fn visit_impl_item_mut(&mut self, item: &mut syn::ImplItem) {
        if let Some(attrs) = impl_item_attrs_mut(item) {
            if attrs.iter().any(is_own_algebraic_attr) {
                return;
            }
            strip_skip(attrs);
        }
        visit_mut::visit_impl_item_mut(self, item);
    }

    fn visit_trait_item_mut(&mut self, item: &mut syn::TraitItem) {
        if let Some(attrs) = trait_item_attrs_mut(item) {
            if attrs.iter().any(is_own_algebraic_attr) {
                return;
            }
            strip_skip(attrs);
        }
        visit_mut::visit_trait_item_mut(self, item);
    }
}

fn item_attrs_mut(item: &mut syn::Item) -> Option<&mut Vec<Attribute>> {
    use syn::Item::*;
    match item {
        Const(i) => Some(&mut i.attrs),
        Enum(i) => Some(&mut i.attrs),
        ExternCrate(i) => Some(&mut i.attrs),
        Fn(i) => Some(&mut i.attrs),
        ForeignMod(i) => Some(&mut i.attrs),
        Impl(i) => Some(&mut i.attrs),
        Macro(i) => Some(&mut i.attrs),
        Mod(i) => Some(&mut i.attrs),
        Static(i) => Some(&mut i.attrs),
        Struct(i) => Some(&mut i.attrs),
        Trait(i) => Some(&mut i.attrs),
        TraitAlias(i) => Some(&mut i.attrs),
        Type(i) => Some(&mut i.attrs),
        Union(i) => Some(&mut i.attrs),
        Use(i) => Some(&mut i.attrs),
        _ => None,
    }
}

fn impl_item_attrs_mut(item: &mut syn::ImplItem) -> Option<&mut Vec<Attribute>> {
    use syn::ImplItem::*;
    match item {
        Const(i) => Some(&mut i.attrs),
        Fn(i) => Some(&mut i.attrs),
        Type(i) => Some(&mut i.attrs),
        Macro(i) => Some(&mut i.attrs),
        _ => None,
    }
}

fn trait_item_attrs_mut(item: &mut syn::TraitItem) -> Option<&mut Vec<Attribute>> {
    use syn::TraitItem::*;
    match item {
        Const(i) => Some(&mut i.attrs),
        Fn(i) => Some(&mut i.attrs),
        Type(i) => Some(&mut i.attrs),
        Macro(i) => Some(&mut i.attrs),
        _ => None,
    }
}
