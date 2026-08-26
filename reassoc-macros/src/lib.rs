// The rewriter reads tokens and writes tokens; it has never needed `unsafe`
// and should not start. `forbid` rather than `deny` so a future `allow`
// cannot quietly reopen it.
#![forbid(unsafe_code)]

mod build;
mod krate;
mod passthrough;
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
/// `f32` and `f64` only and writes every function against it. Dispatch is by
/// trait, so a type parameter has only the bounds it is given, and none of
/// them says "a float the macros can rewrite". This attribute adds that
/// bound to the trait: one line, in one place, and every generic function
/// in the crate is rewritable without touching a signature.
///
/// ```text
/// #[reassoc::algebraic_float]
/// pub trait Float: num_traits::Float + AddAssign + Copy {}
/// impl Float for f32 {}
/// impl Float for f64 {}
///
/// #[reassoc::algebraic]
/// fn dot<T: Float>(a: &[T], b: &[T]) -> T { .. }   // rewritten
/// ```
///
/// The bound is sealed: a trait carrying it can only be implemented for the
/// primitive floats, which is what such a trait is for. What it expands to
/// is not a surface and may change; the attribute is the contract.
///
/// See `reassoc`'s crate docs for the compiled example. This crate cannot
/// depend on `reassoc`, so the example lives there where it can be run.
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
    let mut item_trait = match syn::parse::<syn::ItemTrait>(item.clone()) {
        Ok(t) => t,
        Err(_) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                "`#[algebraic_float]` applies to a trait: put it on the trait your \
                 generic code is written against, the one implemented for `f32` and `f64`",
            )
            .to_compile_error()
            .into();
        }
    };
    let krate = syn::Ident::new(&krate::name(), proc_macro2::Span::call_site());
    let bound: syn::TypeParamBound = syn::parse_quote!(::#krate::__private::AlgebraicFloat);
    item_trait.supertraits.push(bound);
    item_trait.to_token_stream().into()
}

/// Opt a type into `reassoc`'s dispatch layer at its definition.
///
/// Equivalent to `passthrough!(Ty)`, but written where the type is declared:
/// every operator the type implements is dispatched. A generic type is opted
/// in for every instantiation, with the operators each instantiation has.
///
/// See `reassoc::passthrough!` for a worked example. This crate cannot depend
/// on `reassoc`, so the example lives there where it can actually be compiled.
#[proc_macro_derive(Passthrough, attributes(passthrough))]
pub fn derive_passthrough(input: TokenStream) -> TokenStream {
    let input = match syn::parse::<syn::DeriveInput>(input) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error().into(),
    };
    match passthrough::expand(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
