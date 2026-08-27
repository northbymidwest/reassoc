//! The proc macros behind [`reassoc`](https://docs.rs/reassoc): `alg!`,
//! `#[algebraic]`, `#[algebraic_float]` and `#[passthrough]`. A proc-macro
//! crate can export nothing else, so the traits and impls they expand into
//! live in `reassoc`, which re-exports these. Depend on `reassoc`; this
//! crate's version is pinned exactly by it and has no surface of its own.

// The rewriter reads tokens and writes tokens; it has never needed `unsafe`
// and should not start. `forbid` rather than `deny` so a future `allow`
// cannot quietly reopen it.
#![forbid(unsafe_code)]
// As in `reassoc`: whatever docs.rs shows says what it is.
#![warn(missing_docs)]

mod build;
mod krate;
mod opt_in;
mod rewrite;
mod scope;
mod trace;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse::Parser;
use syn::visit_mut::VisitMut;

/// Rewrite arithmetic operators in a single expression to algebraic dispatch.
#[proc_macro]
pub fn alg(input: TokenStream) -> TokenStream {
    // An expression first, so the single-expression form behaves exactly as it
    // always has and never picks up braces it did not ask for.
    if let Ok(mut expr) = syn::parse::<syn::Expr>(input.clone()) {
        let mut rewriter = rewrite::Rewriter::expression_scope();
        rewriter.visit_expr_mut(&mut expr);
        trace::record("alg", proc_macro2::Span::call_site(), "-", rewriter.ops);
        return expr.to_token_stream().into();
    }

    // Otherwise a sequence of statements. Note the braces of `alg! { .. }` are
    // the macro's own delimiters and never reach us, so this parses the body
    // without them (`Block::parse_within`) and supplies the braces on output,
    // which also keeps the result usable anywhere a value is expected.
    match syn::Block::parse_within.parse(input) {
        Ok(stmts) => {
            let mut block = syn::Block {
                brace_token: syn::token::Brace::default(),
                stmts,
            };
            let mut rewriter = rewrite::Rewriter::expression_scope();
            rewriter.visit_block_mut(&mut block);
            trace::record("alg", proc_macro2::Span::call_site(), "-", rewriter.ops);
            block.to_token_stream().into()
        }
        Err(err) => err.to_compile_error().into(),
    }
}

/// Rewrite arithmetic operators throughout a function body, or throughout
/// every member body of an `impl` block, an inline module or a trait.
#[proc_macro_attribute]
pub fn algebraic(attr: TokenStream, item: TokenStream) -> TokenStream {
    let scope = match scope::Scope::parse(attr.into()) {
        Ok(scope) => scope,
        Err(err) => return err.to_compile_error().into(),
    };

    // `skip` means "leave this alone", whatever it is on: a `const fn`, a
    // `const`, a `struct`: anything a reader might mark to be explicit. A
    // container strips it from its members before rustc sees it; this is the
    // path for the item rustc does hand us.
    if scope.skip {
        return item;
    }

    // A function first: the common case, and a method's tokens (`fn f(&self)
    // ..`) parse as one.
    match syn::parse::<syn::ItemFn>(item.clone()) {
        Ok(mut func) => {
            // A second `#[algebraic(..)]` below this one governs instead, as
            // it does for a container member: rewriting here first would
            // apply this scope and silently override the inner parameters.
            if func.attrs.iter().any(rewrite::is_algebraic_attr) {
                return func.to_token_stream().into();
            }
            // `ops::*` are not `const fn`; rejecting up front beats an E0015
            // blamed on the attribute.
            // The function is still emitted, unrewritten, so that its callers
            // resolve and the user reads one error rather than a cascade.
            if let Some(const_token) = func.sig.constness
                && cfg!(not(feature = "const-fn"))
            {
                let err = syn::Error::new_spanned(
                    const_token,
                    "`#[algebraic]` cannot be applied to a `const fn`: the dispatch \
                     functions it generates (`reassoc::ops::*`) are not `const fn`",
                );
                return with_errors(func.to_token_stream(), vec![err]).into();
            }
            let mut rewriter = rewrite::Rewriter::from_scope(scope);
            rewriter.visit_item_fn_mut(&mut func);
            with_errors(func.to_token_stream(), rewriter.errors).into()
        }
        Err(err) => {
            // A trait method without a body gets an authored message.
            if let Ok(f) = syn::parse::<syn::TraitItemFn>(item.clone())
                && f.default.is_none()
            {
                let err = syn::Error::new_spanned(
                    &f.sig.ident,
                    "`#[algebraic]` applies to functions with a body; this trait method has \
                     none to rewrite",
                );
                // The method stays in the trait: removing it would fail every impl.
                return with_errors(f.to_token_stream(), vec![err]).into();
            }
            match syn::parse::<syn::Item>(item) {
                // A function with a syntax error keeps syn's, which points
                // at the problem.
                Ok(syn::Item::Fn(_)) | Err(_) => err.to_compile_error().into(),
                Ok(syn::Item::Mod(m)) if m.content.is_none() => syn::Error::new_spanned(
                    m.ident,
                    "`#[algebraic]` cannot see the body of an out-of-line module; put it on \
                     the items inside that file, or on an inline `mod name { .. }`",
                )
                .to_compile_error()
                .into(),
                Ok(
                    mut container @ (syn::Item::Impl(_) | syn::Item::Mod(_) | syn::Item::Trait(_)),
                ) => {
                    let mut rewriter = rewrite::Rewriter::from_scope(scope);
                    rewriter.visit_item_mut(&mut container);
                    with_errors(container.to_token_stream(), rewriter.errors).into()
                }
                Ok(other) => {
                    let err = syn::Error::new(
                        err.span(),
                        "`#[algebraic]` applies to functions, `impl` blocks, inline modules and \
                         traits; it cannot be applied to this item",
                    );
                    with_errors(other.to_token_stream(), vec![err]).into()
                }
            }
        }
    }
}

/// The rewritten item, followed by any errors the rewriter collected. The
/// item is still emitted so that everything else in it is checked too.
fn with_errors(
    mut tokens: proc_macro2::TokenStream,
    errors: Vec<syn::Error>,
) -> proc_macro2::TokenStream {
    for err in errors {
        tokens.extend(err.to_compile_error());
    }
    tokens
}

/// Mark a user's own float trait so that generic code over it is rewritten.
///
/// A crate that is generic over "some float" defines a trait implemented for
/// `f32` and `f64` and writes every function against it. Dispatch is by
/// trait, so a type parameter has only the bounds it is given, and none of
/// them says "a float the macros can rewrite". This attribute adds that
/// bound to the trait: one line, in one place, and every generic function
/// in the crate is rewritable without touching a signature. The primitive
/// floats need nothing more; any other implementor, a bignum from another
/// crate say, is opted in with `#[passthrough]` on its `impl` of the trait.
///
/// ```text
/// #[reassoc::algebraic_float]
/// pub trait Float: num_traits::Float + AddAssign + Copy {}
/// impl Float for f32 {}
/// impl Float for f64 {}
///
/// #[reassoc::passthrough]
/// impl Float for rug::Float {}                     // a foreign bignum
///
/// #[reassoc::algebraic]
/// fn dot<T: Float>(a: &[T], b: &[T]) -> T { .. }   // rewritten, at all three
/// ```
///
/// What the attribute writes into the trait is not a surface and may
/// change; the attribute is the contract. See `reassoc`'s crate docs for the
/// compiled example and the limits. This crate cannot depend on `reassoc`,
/// so the example lives there.
#[proc_macro_attribute]
pub fn algebraic_float(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::Error::new(
            proc_macro2::TokenStream::from(attr)
                .into_iter()
                .next()
                .map(|t| t.span())
                .unwrap_or_else(proc_macro2::Span::call_site),
            "`#[algebraic_float]` takes no parameters",
        )
        .to_compile_error()
        .into();
    }
    let krate = syn::Ident::new(&krate::name(), proc_macro2::Span::call_site());

    if let Ok(mut item_trait) = syn::parse::<syn::ItemTrait>(item.clone()) {
        // The orphan slot: a type local to the trait's crate, named in the
        // marker's parameters, so that an `impl` of this trait for a type
        // from another crate may implement the marker too (Rust's orphan
        // rule admits a foreign trait for a foreign type only then). Beside
        // the trait, with the trait's visibility, since `#[passthrough]` on
        // an impl names it by the trait's path.
        let tag = opt_in::trait_tag(&item_trait.ident);
        let vis = &item_trait.vis;
        let bound: syn::TypeParamBound =
            syn::parse_quote!(::#krate::__private::AlgebraicFloat<#tag>);
        item_trait.supertraits.push(bound);
        return quote::quote! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #vis struct #tag;
            #item_trait
        }
        .into();
    }
    let msg = if syn::parse::<syn::ItemImpl>(item).is_ok() {
        "`#[algebraic_float]` goes on the trait; a type's `impl` of a marked trait is opted in \
         with `#[passthrough]`"
    } else {
        "`#[algebraic_float]` applies to a trait: put it on the trait your generic code is \
         written against, the one implemented for `f32` and `f64`"
    };
    syn::Error::new(proc_macro2::Span::call_site(), msg)
        .to_compile_error()
        .into()
}

/// Opt a type into `reassoc`'s dispatch layer.
///
/// On whichever item introduces the type. Every operator the type implements
/// (`+ - * / %` with any right-hand type and any output, the `op=` forms, and
/// references wherever the type implements them) is dispatched from then on,
/// exactly as `std::ops` defines it; nothing is listed.
///
/// ```text
/// #[passthrough]                          // a type of yours: its definition
/// struct Vec3(f32, f32, f32);
///
/// #[passthrough]                          // a type from another crate: the `use`
/// use glam::Vec3;
///
/// #[passthrough(f32 * Vec3)]      // a primitive on the *left* of a foreign type
/// use glam::Vec3;                         // is the one pair that has to be named
///
/// #[passthrough]                          // an instantiation of a generic foreign type
/// type C64 = num_complex::Complex<f64>;
///
/// #[passthrough]                          // a bignum into an `#[algebraic_float]` trait
/// impl Float for rug::Float { .. }
/// ```
///
/// A foreign type is opted in **once** per dependency tree: two opt-ins are
/// two dispatch impls, and every concrete operator on the type is then an
/// ambiguity error. The pairs (`A op B`, or `A op= B`, one per operator) are
/// for a primitive on the left of a foreign type only; a type of yours has
/// that automatically. The output is the type's own, resolved for you;
/// `A op B => O` spells it out and has to agree. See `reassoc`'s crate docs for the compiled examples.
#[proc_macro_attribute]
pub fn passthrough(attr: TokenStream, item: TokenStream) -> TokenStream {
    match opt_in::expand(attr.into(), item.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
