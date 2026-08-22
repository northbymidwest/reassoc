mod build;
mod krate;
mod passthrough;
mod rewrite;
mod scope;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse::Parser;
use syn::visit_mut::VisitMut;

/// Declare what an operator yields for a left-hand type, when it is not that
/// type itself. Not part of the public API: `passthrough!` calls it.
#[doc(hidden)]
#[proc_macro]
pub fn declare_output(input: TokenStream) -> TokenStream {
    match passthrough::expand_declare_output(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Rewrite arithmetic operators in a single expression to algebraic dispatch.
#[proc_macro]
pub fn alg(input: TokenStream) -> TokenStream {
    // An expression first, so the single-expression form behaves exactly as it
    // always has and never picks up braces it did not ask for.
    if let Ok(mut expr) = syn::parse::<syn::Expr>(input.clone()) {
        rewrite::Rewriter::expression_scope().visit_expr_mut(&mut expr);
        return expr.to_token_stream().into();
    }

    // Otherwise a sequence of statements. Note the braces of `alg! { .. }` are
    // the macro's own delimiters and never reach us, so this parses the body
    // without them (`Block::parse_within`) and supplies the braces on output —
    // which also keeps the result usable anywhere a value is expected.
    match syn::Block::parse_within.parse(input) {
        Ok(stmts) => {
            let mut block = syn::Block {
                brace_token: syn::token::Brace::default(),
                stmts,
            };
            rewrite::Rewriter::expression_scope().visit_block_mut(&mut block);
            block.to_token_stream().into()
        }
        Err(err) => err.to_compile_error().into(),
    }
}

/// Rewrite arithmetic operators throughout a function body — or throughout
/// every member body of an `impl` block, an inline module or a trait.
#[proc_macro_attribute]
pub fn algebraic(attr: TokenStream, item: TokenStream) -> TokenStream {
    let scope = match scope::Scope::parse(attr.into()) {
        Ok(scope) => scope,
        Err(err) => return err.to_compile_error().into(),
    };

    // A function first: the common case, and a method's tokens (`fn f(&self)
    // ..`) parse as one.
    match syn::parse::<syn::ItemFn>(item.clone()) {
        Ok(mut func) => {
            // `ops::*` are not `const fn`; rejecting up front beats an E0015
            // blamed on the attribute.
            if let Some(const_token) = func.sig.constness {
                return syn::Error::new_spanned(
                    const_token,
                    "`#[algebraic]` cannot be applied to a `const fn`: the dispatch \
                     functions it generates (`reassoc::ops::*`) are not `const fn`",
                )
                .to_compile_error()
                .into();
            }
            // `skip` at the top level simply means "do nothing".
            if scope.skip {
                return func.to_token_stream().into();
            }
            let mut rewriter = rewrite::Rewriter::from_scope(scope);
            rewriter.visit_item_fn_mut(&mut func);
            if let Some(span) = scope.items_span {
                // Inside the body: a method has no room for a sibling item.
                let warn = items_deprecation(span);
                func.block.stmts.insert(0, syn::parse_quote!(#warn;));
            }
            with_errors(func.to_token_stream(), rewriter.errors).into()
        }
        Err(err) => {
            // A trait method without a body gets an authored message.
            if let Ok(f) = syn::parse::<syn::TraitItemFn>(item.clone())
                && f.default.is_none()
            {
                return syn::Error::new_spanned(
                    f.sig.ident,
                    "`#[algebraic]` applies to functions with a body; this trait method has \
                     none to rewrite",
                )
                .to_compile_error()
                .into();
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
                    if scope.skip {
                        return container.to_token_stream().into();
                    }
                    let mut rewriter = rewrite::Rewriter::from_scope(scope);
                    rewriter.visit_item_mut(&mut container);
                    let mut tokens = container.to_token_stream();
                    if let Some(span) = scope.items_span {
                        let warn = items_deprecation(span);
                        tokens.extend(quote::quote!(const _: () = #warn;));
                    }
                    with_errors(tokens, rewriter.errors).into()
                }
                Ok(_) => syn::Error::new(
                    err.span(),
                    "`#[algebraic]` applies to functions, `impl` blocks, inline modules and \
                     traits; it cannot be applied to this item",
                )
                .to_compile_error()
                .into(),
            }
        }
    }
}

/// A use of a `#[deprecated]` constant, spanned at the `items` parameter, so
/// rustc's own `deprecated` lint reports it there: a stable proc macro has no
/// way to emit a warning directly.
fn items_deprecation(span: proc_macro2::Span) -> proc_macro2::TokenStream {
    let name = syn::Ident::new("items_parameter_of_algebraic_is_deprecated", span);
    quote::quote_spanned! {span=>
        {
            #[deprecated(
                note = "nested items are entered by default; `items` is slated for removal. \
                        To leave an item alone, put `#[algebraic(skip)]` on it"
            )]
            #[allow(non_upper_case_globals)]
            const items_parameter_of_algebraic_is_deprecated: () = ();
            #name
        }
    }
}

/// The rewritten item, followed by any errors the rewriter collected — the
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

/// Opt a type into `reassoc`'s dispatch layer at its definition.
///
/// Equivalent to `passthrough!(Ty)`, but written where the type is declared.
/// Defaults to all five operators; a type implementing only some of them names
/// the ones it has with `#[passthrough(add, sub, mul)]`.
///
/// See `reassoc::Passthrough` for a worked example — this crate cannot depend
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
