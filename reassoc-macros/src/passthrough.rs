//! `#[derive(Passthrough)]` — opt a type into the dispatch layer at its
//! definition, rather than with a separate `passthrough!` invocation.

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{DeriveInput, Ident};

/// One arithmetic operator, in every spelling this macro needs.
struct Op {
    /// How the user names it in `#[passthrough(..)]`.
    name: &'static str,
    /// The right-operand trait in `reassoc::traits`, where opting in happens.
    trait_name: &'static str,
    /// That trait's single method.
    method: &'static str,
    /// The `core::ops` trait the generated `where` bound defers to.
    bound: &'static str,
    /// The operator token the generated body uses.
    token: &'static str,
}

const OPS: [Op; 5] = [
    Op {
        name: "add",
        trait_name: "AddRhs",
        method: "add_rhs",
        bound: "Add",
        token: "+",
    },
    Op {
        name: "sub",
        trait_name: "SubRhs",
        method: "sub_rhs",
        bound: "Sub",
        token: "-",
    },
    Op {
        name: "mul",
        trait_name: "MulRhs",
        method: "mul_rhs",
        bound: "Mul",
        token: "*",
    },
    Op {
        name: "div",
        trait_name: "DivRhs",
        method: "div_rhs",
        bound: "Div",
        token: "/",
    },
    Op {
        name: "rem",
        trait_name: "RemRhs",
        method: "rem_rhs",
        bound: "Rem",
        token: "%",
    },
];

pub fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let (selected, with_refs) = selected_ops(&input)?;

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let krate = crate::krate::path();
    let ty = quote! { #name #ty_generics };

    let bounds = |extra: TokenStream| match where_clause {
        Some(existing) => quote! { #existing, #extra },
        None => quote! { where #extra },
    };
    // `RefOperand` rather than a bare `Copy` bound: it carries an
    // `on_unimplemented` message naming `no_refs`, where `Copy` alone would
    // produce an unexplained "cannot move out of a shared reference" pointing
    // into this expansion.
    let ref_bound = quote! { #ty: #krate::traits::RefOperand };

    let impls = selected.into_iter().map(|op| {
        let trait_ident = Ident::new(op.trait_name, Span::call_site());
        let method_ident = Ident::new(op.method, Span::call_site());
        let bound_ident = Ident::new(op.bound, Span::call_site());
        let op_token: TokenStream = op.token.parse().expect("operator token must parse");

        // The `where` bound defers to the type's own `core::ops` impl, which is
        // what lets this work for generic types: without it the body would have
        // no operator to call.
        let op_bound = quote! { #ty: ::core::ops::#bound_ident<Output = #ty> };
        let by_value_bounds = bounds(quote! { #op_bound });

        // Opting in means implementing the right-operand trait, keyed on the
        // left type. The blanket impl in `reassoc::traits` turns that into the
        // operator. Two opt-ins for the same type — say same-type `Mul` and a
        // scalar `Mul` — add two of these and never overlap.
        let by_value = quote! {
            impl #impl_generics #krate::traits::#trait_ident<#ty, #ty> for #ty
            #by_value_bounds
            {
                #[inline(always)]
                fn #method_ident(self, lhs: #ty) -> #ty { lhs #op_token self }
            }
        };

        if !with_refs {
            return by_value;
        }

        let ref_bounds = bounds(quote! { #ref_bound, #op_bound });
        quote! {
            #by_value

            impl #impl_generics #krate::traits::#trait_ident<#ty, #ty> for &#ty
            #ref_bounds
            {
                #[inline(always)]
                fn #method_ident(self, lhs: #ty) -> #ty {
                    lhs #op_token #krate::traits::RefOperand::reassoc_dup(self)
                }
            }

            impl #impl_generics #krate::traits::#trait_ident<&#ty, #ty> for #ty
            #ref_bounds
            {
                #[inline(always)]
                fn #method_ident(self, lhs: &#ty) -> #ty {
                    #krate::traits::RefOperand::reassoc_dup(lhs) #op_token self
                }
            }

            impl #impl_generics #krate::traits::#trait_ident<&#ty, #ty> for &#ty
            #ref_bounds
            {
                #[inline(always)]
                fn #method_ident(self, lhs: &#ty) -> #ty {
                    #krate::traits::RefOperand::reassoc_dup(lhs)
                        #op_token #krate::traits::RefOperand::reassoc_dup(self)
                }
            }
        }
    });

    Ok(quote! { #(#impls)* })
}

/// Reads an optional `#[passthrough(add, mul)]` attribute.
///
/// Defaults to all five operators. A type implementing only some of them must
/// name the ones it has: an impl whose `where` bound is known unsatisfiable for
/// a concrete type is a hard error at the definition rather than a
/// lazily-checked one, so generating all five unconditionally would not compile.
fn selected_ops(input: &DeriveInput) -> syn::Result<(Vec<&'static Op>, bool)> {
    let mut chosen: Vec<&'static Op> = Vec::new();
    let mut with_refs = true;

    for attr in &input.attrs {
        if !attr.path().is_ident("passthrough") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("no_refs") {
                with_refs = false;
                return Ok(());
            }
            match OPS.iter().find(|op| meta.path.is_ident(op.name)) {
                Some(op) => {
                    if !chosen.iter().any(|already| already.name == op.name) {
                        chosen.push(op);
                    }
                    Ok(())
                }
                None => Err(meta.error(
                    "expected `no_refs` or one or more of `add`, `sub`, `mul`, `div`, `rem`",
                )),
            }
        })?;
    }

    if chosen.is_empty() {
        Ok((OPS.iter().collect(), with_refs))
    } else {
        Ok((chosen, with_refs))
    }
}

/// `declare_output!($crate, MulOut, refs|no_refs, A, B, O)` — state that
/// `A`'s operator with a `B` on the right yields `O`, but only when `O` is
/// not `A` itself.
///
/// `passthrough!` cannot make this call: `macro_rules!` cannot compare two
/// `$ty` fragments, and emitting the impl unconditionally would collide with
/// the blanket `impl<A, B> MulOut<B, A> for A` every time the output *is* the
/// left operand — which is nearly always. So the decision moves here, where
/// the two types can be compared as written.
///
/// `no_refs` emits the value pair. `refs` emits only the `&B` combinations,
/// because the reference-emitting form of `passthrough!` expands the
/// `no_refs` form first and that call has already emitted the value pair.
///
/// Comparison is syntactic, which is exact for every spelling that reaches this
/// macro except an alias: `passthrough!(mul: Vec3, Vec3 => V3)` where
/// `type V3 = Vec3` reads as a differing output and emits an impl that collides
/// with the blanket. The error names the `passthrough!` line, and spelling the
/// output the same way as the left operand resolves it.
///
/// The crate path arrives as an argument because a proc macro cannot write
/// `$crate`, and `passthrough!` is invoked from inside `reassoc` itself.
pub fn expand_declare_output(input: TokenStream) -> syn::Result<TokenStream> {
    struct Args {
        krate: syn::Path,
        trait_ident: Ident,
        refs: bool,
        left: syn::Type,
        right: syn::Type,
        output: syn::Type,
    }

    impl syn::parse::Parse for Args {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let krate = input.parse()?;
            input.parse::<syn::Token![,]>()?;
            let trait_ident = input.parse()?;
            input.parse::<syn::Token![,]>()?;
            let mode: Ident = input.parse()?;
            let refs = match mode.to_string().as_str() {
                "refs" => true,
                "no_refs" => false,
                _ => return Err(syn::Error::new(mode.span(), "expected `refs` or `no_refs`")),
            };
            input.parse::<syn::Token![,]>()?;
            let left = input.parse()?;
            input.parse::<syn::Token![,]>()?;
            let right = input.parse()?;
            input.parse::<syn::Token![,]>()?;
            let output = input.parse()?;
            Ok(Args {
                krate,
                trait_ident,
                refs,
                left,
                right,
                output,
            })
        }
    }

    let Args {
        krate,
        trait_ident,
        refs,
        left,
        right,
        output,
    } = syn::parse2(input)?;
    if left.to_token_stream().to_string() == output.to_token_stream().to_string() {
        return Ok(TokenStream::new());
    }
    let right: syn::Type = if refs {
        syn::parse_quote!(&#right)
    } else {
        right
    };
    Ok(quote! {
        impl #krate::traits::#trait_ident<#right, #output> for #left {}
        impl #krate::traits::#trait_ident<#right, #output> for &#left {}
    })
}
