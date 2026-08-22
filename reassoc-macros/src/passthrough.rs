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
    /// The marker saying `+=` may be formed from `+` by reading the place.
    synth: &'static str,
    /// The in-place compound form, named `add_assign` etc. in the attribute.
    assign: AssignOp,
}

struct AssignOp {
    name: &'static str,
    trait_name: &'static str,
    method: &'static str,
    bound: &'static str,
    token: &'static str,
}

const OPS: [Op; 5] = [
    Op {
        name: "add",
        trait_name: "AddRhs",
        method: "add_rhs",
        bound: "Add",
        token: "+",
        synth: "SynthAddAssign",
        assign: AssignOp {
            name: "add_assign",
            trait_name: "AddAssignRhs",
            method: "add_assign_rhs",
            bound: "AddAssign",
            token: "+=",
        },
    },
    Op {
        name: "sub",
        trait_name: "SubRhs",
        method: "sub_rhs",
        bound: "Sub",
        token: "-",
        synth: "SynthSubAssign",
        assign: AssignOp {
            name: "sub_assign",
            trait_name: "SubAssignRhs",
            method: "sub_assign_rhs",
            bound: "SubAssign",
            token: "-=",
        },
    },
    Op {
        name: "mul",
        trait_name: "MulRhs",
        method: "mul_rhs",
        bound: "Mul",
        token: "*",
        synth: "SynthMulAssign",
        assign: AssignOp {
            name: "mul_assign",
            trait_name: "MulAssignRhs",
            method: "mul_assign_rhs",
            bound: "MulAssign",
            token: "*=",
        },
    },
    Op {
        name: "div",
        trait_name: "DivRhs",
        method: "div_rhs",
        bound: "Div",
        token: "/",
        synth: "SynthDivAssign",
        assign: AssignOp {
            name: "div_assign",
            trait_name: "DivAssignRhs",
            method: "div_assign_rhs",
            bound: "DivAssign",
            token: "/=",
        },
    },
    Op {
        name: "rem",
        trait_name: "RemRhs",
        method: "rem_rhs",
        bound: "Rem",
        token: "%",
        synth: "SynthRemAssign",
        assign: AssignOp {
            name: "rem_assign",
            trait_name: "RemAssignRhs",
            method: "rem_assign_rhs",
            bound: "RemAssign",
            token: "%=",
        },
    },
];

pub fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let (selected, assigns, with_refs) = selected_ops(&input)?;

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let krate = crate::krate::path();
    let ty = quote! { #name #ty_generics };

    // Appended as a predicate, not as tokens after the existing clause: the
    // existing clause may end in a trailing comma (rustfmt writes one for any
    // multi-line bound list), and `#existing, #extra` then reads `, ,`.
    let bounds = |extra: syn::WherePredicate| {
        let mut clause = where_clause.cloned().unwrap_or_else(|| syn::WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        });
        clause.predicates.push(extra);
        clause
    };
    // `RefOperand` rather than a bare `Copy` bound: it carries an
    // `on_unimplemented` message naming `no_refs`, where `Copy` alone would
    // produce an unexplained "cannot move out of a shared reference" pointing
    // into this expansion.
    let ref_bound: syn::WherePredicate = syn::parse_quote! { #ty: #krate::traits::RefOperand };

    let impls = selected.iter().map(|op| {
        let trait_ident = Ident::new(op.trait_name, Span::call_site());
        let method_ident = Ident::new(op.method, Span::call_site());
        let bound_ident = Ident::new(op.bound, Span::call_site());
        let synth_ident = Ident::new(op.synth, Span::call_site());
        let op_token: TokenStream = op.token.parse().expect("operator token must parse");
        // A `Copy` type's `+=` is formed from `+` unless it asked for the
        // in-place form, whose impl would overlap the blanket one.
        let in_place = assigns.iter().any(|a| a.name == op.assign.name);

        // The `where` bound defers to the type's own `core::ops` impl, which is
        // what lets this work for generic types: without it the body would have
        // no operator to call.
        let op_bound: syn::WherePredicate =
            syn::parse_quote! { #ty: ::core::ops::#bound_ident<Output = #ty> };
        let by_value_bounds = bounds(op_bound.clone());

        // Opting in means implementing the right-operand trait, keyed on the
        // left type. The blanket impl in `reassoc::traits` turns that into the
        // operator. Two opt-ins for the same type — say same-type `Mul` and a
        // scalar `Mul` — add two of these and never overlap.
        // The marker's supertrait must be provable for a generic type. Bounded
        // on `RefOperand` (`Copy` under a name that carries the `no_refs` note)
        // so a non-`Copy` type sees that note, not a bare `Copy` error.
        let copy_bounds = bounds(ref_bound.clone());
        let synth_value = (with_refs && !in_place).then(|| {
            quote! {
                impl #impl_generics #krate::traits::#synth_ident<#ty> for #ty #copy_bounds {}
            }
        });
        let by_value = quote! {
            #synth_value
            impl #impl_generics #krate::traits::#trait_ident<#ty, #ty> for #ty
            #by_value_bounds
            {
                #[inline(always)]
                #[track_caller]
                fn #method_ident(self, lhs: #ty) -> #ty { lhs #op_token self }
            }
        };

        if !with_refs {
            return by_value;
        }

        let synth_ref = (!in_place).then(|| {
            quote! {
                impl #impl_generics #krate::traits::#synth_ident<&#ty> for #ty #copy_bounds {}
            }
        });
        let mut ref_bounds = bounds(ref_bound.clone());
        ref_bounds.predicates.push(op_bound);
        quote! {
            #by_value
            #synth_ref

            impl #impl_generics #krate::traits::#trait_ident<#ty, #ty> for &#ty
            #ref_bounds
            {
                #[inline(always)]
                #[track_caller]
                fn #method_ident(self, lhs: #ty) -> #ty {
                    lhs #op_token #krate::traits::RefOperand::reassoc_dup(self)
                }
            }

            impl #impl_generics #krate::traits::#trait_ident<&#ty, #ty> for #ty
            #ref_bounds
            {
                #[inline(always)]
                #[track_caller]
                fn #method_ident(self, lhs: &#ty) -> #ty {
                    #krate::traits::RefOperand::reassoc_dup(lhs) #op_token self
                }
            }

            impl #impl_generics #krate::traits::#trait_ident<&#ty, #ty> for &#ty
            #ref_bounds
            {
                #[inline(always)]
                #[track_caller]
                fn #method_ident(self, lhs: &#ty) -> #ty {
                    #krate::traits::RefOperand::reassoc_dup(lhs)
                        #op_token #krate::traits::RefOperand::reassoc_dup(self)
                }
            }
        }
    });

    // In-place compound forms, through the type's own `AddAssign` etc.
    let assign_impls = assigns.iter().map(|op| {
        let trait_ident = Ident::new(op.trait_name, Span::call_site());
        let method_ident = Ident::new(op.method, Span::call_site());
        let bound_ident = Ident::new(op.bound, Span::call_site());
        let op_token: TokenStream = op.token.parse().expect("operator token must parse");
        let op_bound: syn::WherePredicate =
            syn::parse_quote! { #ty: ::core::ops::#bound_ident<#ty> };
        let by_value_bounds = bounds(op_bound.clone());
        let by_value = quote! {
            impl #impl_generics #krate::traits::#trait_ident<#ty> for #ty
            #by_value_bounds
            {
                #[inline(always)]
                #[track_caller]
                fn #method_ident(self, lhs: &mut #ty) { *lhs #op_token self }
            }
        };
        if !with_refs {
            return by_value;
        }
        let mut ref_bounds = bounds(ref_bound.clone());
        ref_bounds.predicates.push(op_bound);
        quote! {
            #by_value

            impl #impl_generics #krate::traits::#trait_ident<#ty> for &#ty
            #ref_bounds
            {
                #[inline(always)]
                #[track_caller]
                fn #method_ident(self, lhs: &mut #ty) {
                    *lhs #op_token #krate::traits::RefOperand::reassoc_dup(self)
                }
            }
        }
    });

    Ok(quote! { #(#impls)* #(#assign_impls)* })
}

/// Reads an optional `#[passthrough(add, mul, add_assign, no_refs)]` attribute.
///
/// Naming nothing means all five binary operators and no in-place forms. A
/// type implementing only some of them must name the ones it has: an impl
/// whose `where` bound is known unsatisfiable for a concrete type is a hard
/// error at the definition rather than a lazily-checked one, so generating
/// all five unconditionally would not compile. Naming only in-place forms
/// means only those — a type with `AddAssign` and no `Add` is ordinary.
/// `add_assign` etc. are never assumed, since a `Copy` type gets `+=` from
/// `+` without them.
type Selected = (Vec<&'static Op>, Vec<&'static AssignOp>, bool);

fn selected_ops(input: &DeriveInput) -> syn::Result<Selected> {
    let mut chosen: Vec<&'static Op> = Vec::new();
    let mut assigns: Vec<&'static AssignOp> = Vec::new();
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
            if let Some(op) = OPS.iter().find(|op| meta.path.is_ident(op.name)) {
                if !chosen.iter().any(|already| already.name == op.name) {
                    chosen.push(op);
                }
                return Ok(());
            }
            if let Some(op) = OPS.iter().find(|op| meta.path.is_ident(op.assign.name)) {
                if !assigns.iter().any(|already| already.name == op.assign.name) {
                    assigns.push(&op.assign);
                }
                return Ok(());
            }
            Err(meta.error(
                "expected `no_refs`, one or more of `add`, `sub`, `mul`, `div`, `rem`, \
                 or the in-place forms `add_assign` .. `rem_assign`",
            ))
        })?;
    }

    if chosen.is_empty() && assigns.is_empty() {
        chosen = OPS.iter().collect();
    }
    Ok((chosen, assigns, with_refs))
}

/// `declare_output!($crate, MulOut, refs|no_refs, A, B, O)` — state that
/// `A`'s operator with `B` on the right yields `O`, but only when `O` is not
/// `A` as written: the blanket "yields the left type" impl already covers that
/// case and a specific impl would collide with it. `macro_rules!` cannot
/// compare two `$ty` fragments, hence a proc macro. `no_refs` emits the value
/// pair; `refs` emits only the `&B` combinations, since the reference form of
/// `passthrough!` has already expanded `no_refs`. The crate path is an
/// argument because a proc macro cannot write `$crate` and `passthrough!` is
/// invoked from inside `reassoc` itself. The comparison is syntactic, so an
/// alias of the left type (`=> V3` for `type V3 = Vec3`) reads as different
/// and collides — the error lands on the `passthrough!` line.
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
