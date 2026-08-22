//! Expands every `#[reassoc::algebraic]` / `#[algebraic]` function in a file
//! with the crate's own rewriter and prints the result as source. Only the
//! plain-function form, default scope; enough for the benchmark and for
//! eyeballing an expansion.
#![allow(dead_code)]

mod krate {
    use proc_macro2::Span;
    pub fn ident(span: Span) -> syn::Ident {
        syn::Ident::new("reassoc", span)
    }
}
#[path = "../../../../reassoc-macros/src/scope.rs"]
mod scope;
#[path = "../../../../reassoc-macros/src/rewrite.rs"]
mod rewrite;

use quote::ToTokens;
use syn::visit_mut::VisitMut;

fn main() {
    let path = std::env::args().nth(1).expect("usage: expander FILE.rs");
    let src = std::fs::read_to_string(&path).expect("read input");
    let mut file: syn::File = syn::parse_str(&src).expect("parse input as a Rust file");
    let mut expanded = 0usize;
    for item in &mut file.items {
        if let syn::Item::Fn(f) = item {
            let before = f.attrs.len();
            f.attrs.retain(|a| {
                !a.path()
                    .segments
                    .last()
                    .is_some_and(|s| s.ident == "algebraic")
            });
            if f.attrs.len() != before {
                let mut rw = rewrite::Rewriter::from_scope(scope::Scope::default());
                rw.visit_item_fn_mut(f);
                assert!(rw.errors.is_empty(), "rewriter reported errors");
                expanded += 1;
            }
        }
    }
    eprintln!("expanded {expanded} functions");
    print!("{}", file.to_token_stream());
}
