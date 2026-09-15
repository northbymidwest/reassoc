//! The rewriter, driven directly on syn trees.
//!
//! Everything else about the macros is tested by compiling what they emit
//! (`reassoc/tests`), which is the stronger check and the one to reach for.
//! This file is for what cannot be reached that way: an attribute on an
//! expression needs `stmt_expr_attributes`, so a `.rs` file exercising one
//! would not build on the stable toolchain the suite runs on. syn parses the
//! shape regardless of the gate, and the shape is all the rewriter sees.
//!
//! The rewriter is included by `#[path]` rather than imported: a
//! `proc-macro = true` crate exports nothing but proc macros, so there is no
//! other way to reach `Rewriter` from a test.
//! `scripts/compile-bench/expander` does the same for the same reason, and
//! mirrors `krate` and `trace` by hand; nothing here needs that, since
//! `krate::name` falls back to the plain name off a cargo build and
//! `trace::record` is a no-op with `REASSOC_TRACE` unset.
#![allow(dead_code)]

// `trace.rs` asks `proc_macro::is_available()` before resolving a span, and
// that crate is linked automatically only for a `proc-macro = true` one. It
// answers `false` here, so nothing is written and no span is touched.
extern crate proc_macro;

#[path = "../src/build.rs"]
mod build;
#[path = "../src/krate.rs"]
mod krate;
#[path = "../src/rewrite.rs"]
mod rewrite;
#[path = "../src/scope.rs"]
mod scope;
#[path = "../src/trace.rs"]
mod trace;

use syn::parse::Parser;

use quote::ToTokens;
use rewrite::Rewriter;
use syn::Expr;
use syn::visit_mut::{self, VisitMut};

fn rewritten(src: &str) -> String {
    let mut f: syn::ItemFn = syn::parse_str(src).expect("parses");
    Rewriter::expression_scope().visit_item_fn_mut(&mut f);
    f.to_token_stream().to_string()
}

/// `unparen` used to drop the layer and the attributes with it, leaving no
/// trace of `#[allow(..)]` in the output. Plain Rust honours it, so the
/// rewrite must not swallow it.
#[test]
fn attributes_on_a_parenthesised_operand_are_not_dropped() {
    let out = rewritten("fn f(x: f32, y: f32, z: f32) -> f32 { x * #[allow(unused)] (y + z) }");
    assert!(out.contains("allow"), "attribute dropped: {out}");
    assert!(out.contains("ops :: mul"), "not rewritten: {out}");
}

/// The same for the invisible group a `macro_rules!` fragment arrives in.
#[test]
fn attributes_on_a_grouped_operand_are_not_dropped() {
    let mut f: syn::ItemFn =
        syn::parse_str("fn f(x: f32, y: f32, z: f32) -> f32 { x * (y + z) }").expect("parses");
    // Rebuild the right operand as an attributed group, which is what a
    // fragment carrying an attribute would look like.
    let syn::Stmt::Expr(syn::Expr::Binary(top), _) = &mut f.block.stmts[0] else {
        panic!("expected a binary tail expression")
    };
    let inner = core::mem::replace(&mut *top.right, Expr::PLACEHOLDER);
    *top.right = Expr::Group(syn::ExprGroup {
        attrs: vec![syn::parse_quote!(#[allow(unused)])],
        group_token: syn::token::Group::default(),
        expr: Box::new(inner),
    });
    Rewriter::expression_scope().visit_item_fn_mut(&mut f);
    let out = f.to_token_stream().to_string();
    assert!(out.contains("allow"), "attribute dropped: {out}");
}

/// An operand with no attributes still loses exactly one layer, which is
/// what keeps `unused_parens` quiet about the call's own delimiters. The
/// emitted calls bring parentheses of their own, so this looks for the
/// node rather than the token.
#[test]
fn an_unattributed_layer_is_still_stripped() {
    struct FindParen(bool);
    impl VisitMut for FindParen {
        fn visit_expr_paren_mut(&mut self, e: &mut syn::ExprParen) {
            self.0 = true;
            visit_mut::visit_expr_paren_mut(self, e);
        }
    }
    let out = rewritten("fn f(x: f32, y: f32, z: f32) -> f32 { x * (y + z) }");
    let mut f: syn::ItemFn = syn::parse_str(&out).expect("output parses");
    let mut find = FindParen(false);
    find.visit_item_fn_mut(&mut f);
    assert!(!find.0, "a redundant layer was kept: {out}");
}

/// `visit_expr_mut` discards a binary node's own `attrs` when it rebuilds the
/// node as a call, which is correct only while that field is empty. It is:
/// syn descends the left spine when it places attributes
/// (`Expr::Binary(e) => &mut e.left`, syn-3.0.4 `stmt.rs`), so they land on
/// the leftmost leaf and travel into the call with it, which is where rustc
/// reads them too. `#[allow(..)] a + b` becomes `ops::add(#[allow(..)] a, b)`:
/// the same attribute on the same expression.
///
/// Read off syn's source that is an inference about one version, and the
/// manifest asks for `syn = "3.0"`. This is the measurement instead, over
/// every shape the rewriter can be handed and every entry point it has:
/// `alg!` parses an `Expr` or a statement sequence, `#[algebraic]` an
/// `ItemFn`. Should a syn update ever start attaching attributes to the binary
/// node itself, this fails, and the emitter has to carry them onto the call
/// rather than drop them.
#[test]
fn syn_never_puts_attributes_on_a_binary_node() {
    /// Every shape that reaches a binary operator with an attribute in front
    /// of it. `(a + b)` is absent on purpose: there the attributes land on the
    /// parentheses, which is the case `unparen` has to keep.
    const SHAPES: [&str; 10] = [
        "#[allow(x)] a + b",
        "#[allow(x)] a += b",
        "#[allow(x)] a.b + c",
        "#[allow(x)] *a + b",
        "#[allow(x)] -a + b",
        "#[allow(x)] (a) + b",
        "#[allow(x)] a as f32 + b",
        "#[allow(x)] f() + b",
        "#[allow(x)] [a][0] + b",
        "#[allow(x)] a + b * c - d",
    ];

    /// The same, with a block-like left operand. These reach the rewriter only
    /// as expressions: in statement position a block-like expression ends the
    /// statement, so `if c { a } else { b } + d` is a parse error there in
    /// plain Rust too, and listing them apart is what keeps that from being a
    /// silent skip.
    const BLOCK_LIKE: [&str; 5] = [
        "#[allow(x)] if c { a } else { b } + d",
        "#[allow(x)] match c { _ => a } + d",
        "#[allow(x)] { a } + b",
        "#[allow(x)] loop { break a; } + b",
        "#[allow(x)] unsafe { a } + b",
    ];

    struct Assert(&'static str);
    impl VisitMut for Assert {
        fn visit_expr_binary_mut(&mut self, e: &mut syn::ExprBinary) {
            assert!(
                e.attrs.is_empty(),
                "syn now puts attributes on the binary node itself ({}): \
                 `visit_expr_mut` drops them and must stop doing so",
                self.0
            );
            visit_mut::visit_expr_binary_mut(self, e);
        }
    }

    // `alg!(expr)`, the entry point every shape reaches.
    for src in SHAPES.iter().chain(&BLOCK_LIKE) {
        let mut expr: Expr = syn::parse_str(src).expect(src);
        Assert(src).visit_expr_mut(&mut expr);
    }
    for src in SHAPES {
        // `alg! { stmts.. }`, with and without the trailing semicolon.
        for body in [src.to_owned(), format!("{src};")] {
            for stmt in &mut syn::Block::parse_within.parse_str(&body).expect(src) {
                Assert(src).visit_stmt_mut(stmt);
            }
        }
        // `#[algebraic] fn ..`.
        let mut f: syn::ItemFn = syn::parse_str(&format!("fn f() {{ {src}; }}")).expect(src);
        Assert(src).visit_item_fn_mut(&mut f);
    }
}

// ---- `.sum()`, `.product()` and `.powi(n)` ----
//
// The emission shape for the name-matched methods, which
// `reassoc/tests/methods.rs` then proves observable by compiling. Here: the
// exact tokens. Method syntax on a hidden extension trait brought in by a
// `use` inside a block, not a function call: a `&mut` iterator receiver is
// then reborrowed as native method syntax reborrows it, and a `&f32`
// receiver of `powi` auto-derefs; a turbofish travels as the method's first
// type argument.

#[test]
fn sum_and_product_calls_become_trait_method_calls() {
    let out = rewritten("fn f(v: &[f32]) -> f32 { v.iter().sum() }");
    assert!(out.contains("{ # [allow (unused_imports)] use :: reassoc :: __private :: ops :: Reduce as _ ; v . iter () . __reassoc_sum () }"), "{out}");
    let out = rewritten("fn f(v: &[f32]) -> f32 { v.iter().product() }");
    assert!(out.contains("{ # [allow (unused_imports)] use :: reassoc :: __private :: ops :: Reduce as _ ; v . iter () . __reassoc_product () }"), "{out}");
}

#[test]
fn powi_becomes_a_trait_method_call() {
    let out = rewritten("fn f(x: f32) -> f32 { x.powi(2) }");
    assert!(out.contains("{ # [allow (unused_imports)] use :: reassoc :: __private :: ops :: Powi as _ ; x . __reassoc_powi (2) }"), "{out}");
    // The exponent is an expression like any other, visited first.
    let out = rewritten("fn f(x: f32, n: i32, m: i32) -> f32 { x.powi(n * m) }");
    assert!(
        out.contains("x . __reassoc_powi (:: reassoc :: __private :: ops :: mul (n , m))"),
        "{out}"
    );
}

#[test]
fn a_turbofish_on_sum_becomes_the_first_type_argument() {
    let out = rewritten("fn f(v: &[f32]) -> f32 { v.iter().sum::<f32>() }");
    assert!(
        out.contains("v . iter () . __reassoc_sum :: < f32 , _ > ()"),
        "{out}"
    );
    let out = rewritten("fn f(v: &[f32]) -> f32 { v.iter().product::<_>() }");
    assert!(
        out.contains("v . iter () . __reassoc_product :: < _ , _ > ()"),
        "{out}"
    );
}

#[test]
fn other_shapes_of_the_same_names_are_left_alone() {
    // `Iterator::sum` takes nothing: a `sum(x)` is some other method.
    let out = rewritten("fn f(g: G) -> f32 { g.sum(1) }");
    assert!(!out.contains("__reassoc"), "{out}");
    // Two type arguments cannot be `Iterator::sum` either.
    let out = rewritten("fn f(g: G) -> f32 { g.sum::<f32, u8>() }");
    assert!(!out.contains("__reassoc"), "{out}");
    // `f32::powi` takes exactly one argument and no type argument.
    let out = rewritten("fn f(g: G) -> f32 { g.powi() + g.powi(1, 2) + g.powi::<u8>(2) }");
    assert!(!out.contains("__reassoc"), "{out}");
}

#[test]
fn the_receiver_is_rewritten_before_the_call() {
    let out = rewritten("fn f(v: &[f32], k: f32) -> f32 { v.iter().map(|x| x * k).sum() }");
    assert!(out.contains("v . iter () . map (| x | :: reassoc :: __private :: ops :: mul (x , k)) . __reassoc_sum ()"), "{out}");
}

#[test]
fn parentheses_around_the_receiver_are_kept() {
    // The receiver stays in receiver position, where its parentheses may be
    // load-bearing: `(0..10).sum()` and `(x as f32).powi(2)`.
    let out = rewritten("fn f(x: u8) -> f32 { (0..10).sum::<i32>() as f32 + (x as f32).powi(2) }");
    assert!(
        out.contains("(0 .. 10) . __reassoc_sum :: < i32 , _ > ()"),
        "{out}"
    );
    assert!(out.contains("(x as f32) . __reassoc_powi (2)"), "{out}");
}

#[test]
fn reductions_false_leaves_the_reductions_alone() {
    let scope = scope::Scope::parse(quote::quote!(reductions = false)).unwrap();
    let mut f: syn::ItemFn = syn::parse_str(
        "fn f(v: &[f32], x: f32) -> f32 { v.iter().sum::<f32>() + v.iter().product::<f32>() + x.powi(2) }",
    )
    .unwrap();
    Rewriter::from_scope(scope).visit_item_fn_mut(&mut f);
    let out = f.to_token_stream().to_string();
    assert!(out.contains("v . iter () . sum :: < f32 > ()"), "{out}");
    assert!(out.contains("v . iter () . product :: < f32 > ()"), "{out}");
    // `powi` has its own switch, and the `+`s are still rewritten: the
    // switch is about the two calls, nothing else.
    assert!(out.contains("x . __reassoc_powi (2)"), "{out}");
    assert!(out.contains("ops :: add"), "{out}");
}

#[test]
fn powi_false_leaves_powi_alone() {
    let scope = scope::Scope::parse(quote::quote!(powi = false)).unwrap();
    let mut f: syn::ItemFn =
        syn::parse_str("fn f(v: &[f32], x: f32) -> f32 { v.iter().sum::<f32>() + x.powi(2) }")
            .unwrap();
    Rewriter::from_scope(scope).visit_item_fn_mut(&mut f);
    let out = f.to_token_stream().to_string();
    assert!(out.contains("x . powi (2)"), "{out}");
    assert!(
        out.contains("v . iter () . __reassoc_sum :: < f32 , _ > ()"),
        "{out}"
    );
}

#[test]
fn a_const_fn_keeps_its_calls_as_written() {
    // Rustc rejects `Iterator::sum` and `f32::powi` in a `const fn` itself;
    // leaving the call untouched keeps that native error, and the rewriter
    // reports nothing of its own for it.
    // Nested, since a `const fn` is entered by the item visitor, which is
    // what sets the const context (`#[algebraic]` on one directly is refused
    // before the rewriter runs).
    let mut f: syn::ItemFn = syn::parse_str(
        "fn outer() { const fn f(v: &[f32], x: f32) -> f32 { let _s: f32 = v.iter().sum(); x.powi(2) } }",
    )
    .unwrap();
    let mut rewriter = Rewriter::expression_scope();
    rewriter.visit_item_fn_mut(&mut f);
    let out = f.to_token_stream().to_string();
    assert!(out.contains("v . iter () . sum () ; x . powi (2)"), "{out}");
    assert!(!out.contains("__reassoc"), "{out}");
    assert!(rewriter.errors.is_empty(), "{:?}", rewriter.errors);
}
