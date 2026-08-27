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
    Rewriter::expression_scope(false).visit_item_fn_mut(&mut f);
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
    Rewriter::expression_scope(false).visit_item_fn_mut(&mut f);
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
