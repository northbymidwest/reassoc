//! `#[derive(Passthrough)]` — opt a type into the dispatch layer at its
//! definition, rather than with a separate `passthrough!` invocation.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Ident};

/// One arithmetic operator, in every spelling this macro needs.
struct Op {
    /// How the user names it in `#[passthrough(..)]`.
    name: &'static str,
    /// The dispatch trait in `reassoc::traits`.
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
        trait_name: "AlgAdd",
        method: "alg_add",
        bound: "Add",
        token: "+",
    },
    Op {
        name: "sub",
        trait_name: "AlgSub",
        method: "alg_sub",
        bound: "Sub",
        token: "-",
    },
    Op {
        name: "mul",
        trait_name: "AlgMul",
        method: "alg_mul",
        bound: "Mul",
        token: "*",
    },
    Op {
        name: "div",
        trait_name: "AlgDiv",
        method: "alg_div",
        bound: "Div",
        token: "/",
    },
    Op {
        name: "rem",
        trait_name: "AlgRem",
        method: "alg_rem",
        bound: "Rem",
        token: "%",
    },
];

pub fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let (selected, with_refs) = selected_ops(&input)?;

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let krate = crate::krate::path();

    // The right-hand operand is a type parameter bounded by `Operand`, not a
    // second impl per reference combination. That keeps one `Alg*` candidate
    // per operand type, which is what lets rustc name both types when they do
    // not match — see `reassoc::traits::Operand`. The name is deliberately
    // unlikely to collide with a generic parameter the user already has.
    let mut rhs_generics = input.generics.clone();
    rhs_generics.params.insert(
        0,
        syn::parse_quote!(__ReassocRhs: #krate::traits::Operand<#name #ty_generics>),
    );
    let (rhs_impl_generics, _, _) = rhs_generics.split_for_impl();

    let bounds = |extra: TokenStream| match where_clause {
        Some(existing) => quote! { #existing, #extra },
        None => quote! { where #extra },
    };
    // `RefOperand` rather than a bare `Copy` bound: it carries an
    // `on_unimplemented` message naming `no_refs`, where `Copy` alone would
    // produce an unexplained "cannot move out of a shared reference" pointing
    // into this expansion.
    let ref_bound = quote! { #name #ty_generics: #krate::traits::RefOperand };

    // Emitted once for the type, not once per operator.
    let mut operand_impls = quote! {
        impl #impl_generics #krate::traits::Operand<#name #ty_generics>
            for #name #ty_generics
        #where_clause
        {
            #[inline(always)]
            fn reassoc_operand(self) -> #name #ty_generics { self }
        }
    };
    if with_refs {
        let ref_bounds = bounds(ref_bound.clone());
        operand_impls.extend(quote! {
            impl #impl_generics #krate::traits::Operand<#name #ty_generics>
                for &#name #ty_generics
            #ref_bounds
            {
                #[inline(always)]
                fn reassoc_operand(self) -> #name #ty_generics {
                    #krate::traits::RefOperand::reassoc_dup(self)
                }
            }
        });
    }

    let impls = selected.into_iter().map(|op| {
        let trait_ident = Ident::new(op.trait_name, Span::call_site());
        let method_ident = Ident::new(op.method, Span::call_site());
        let bound_ident = Ident::new(op.bound, Span::call_site());
        let op_token: TokenStream = op.token.parse().expect("operator token must parse");

        // The `where` bound defers to the type's own `core::ops` impl, which is
        // what lets this work for generic types: without it the body would have
        // no operator to call.
        let op_bound = quote! {
            #name #ty_generics: ::core::ops::#bound_ident<Output = #name #ty_generics>
        };
        let by_value_bounds = bounds(quote! { #op_bound });

        let by_value = quote! {
            impl #rhs_impl_generics
                #krate::traits::#trait_ident<__ReassocRhs, #name #ty_generics>
                for #name #ty_generics
            #by_value_bounds
            {
                #[inline(always)]
                fn #method_ident(self, rhs: __ReassocRhs) -> #name #ty_generics {
                    self #op_token #krate::traits::Operand::reassoc_operand(rhs)
                }
            }
        };

        if !with_refs {
            return by_value;
        }

        // A reference on the left, so an opted-in type behaves like a built-in
        // one in iterator code. This dereferences, hence the `RefOperand`
        // bound; a type that is not `Copy` opts out with
        // `#[passthrough(no_refs)]`.
        let ref_bounds = bounds(quote! { #ref_bound, #op_bound });
        quote! {
            #by_value

            impl #rhs_impl_generics
                #krate::traits::#trait_ident<__ReassocRhs, #name #ty_generics>
                for &#name #ty_generics
            #ref_bounds
            {
                #[inline(always)]
                fn #method_ident(self, rhs: __ReassocRhs) -> #name #ty_generics {
                    #krate::traits::RefOperand::reassoc_dup(self)
                        #op_token #krate::traits::Operand::reassoc_operand(rhs)
                }
            }
        }
    });

    Ok(quote! { #operand_impls #(#impls)* })
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
