//! `#[passthrough]`: the one opt-in, on whichever item introduces the type.
//!
//! On a definition it is a marker impl with the type's generics. On a `use`
//! or a `type` alias, which is how a type from another crate is named, it
//! is the block the orphan rule needs: a private tag type, `OptInTag` for it,
//! and the marker under that tag, so that this crate's traits are implemented
//! for the foreign type with a local type in the header. On an `impl` of an
//! `#[algebraic_float]` trait it is that block plus the marker impl the
//! trait's bound requires, naming the trait's hidden tag by the trait's path.
//! A primitive on the left of a foreign type (`f32 * Vec3`) is the one pair
//! the blankets cannot reach (they are keyed on the default tag, which a
//! foreign type never has), so it is named in the attribute's arguments and
//! emitted under the block's tag.

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{Token, spanned::Spanned};

/// The hidden tag `#[algebraic_float]` emits beside a trait and the impl form
/// names: one per marked trait, derived from the trait's name.
pub fn trait_tag(trait_ident: &syn::Ident) -> syn::Ident {
    format_ident!("__ReassocTag_{}", trait_ident)
}

/// `A op B`, `A op B => O`, or `A op= B`: one dispatch impl, written out.
struct Pair {
    lhs: syn::Type,
    op: syn::BinOp,
    rhs: syn::Type,
    out: Option<syn::Type>,
}

impl Parse for Pair {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // `without_plus`: a bare `A + B` would otherwise parse as a trait object.
        let lhs = input.call(syn::Type::without_plus)?;
        let op: syn::BinOp = input.parse()?;
        let rhs = input.call(syn::Type::without_plus)?;
        let out = if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Pair { lhs, op, rhs, out })
    }
}

impl Pair {
    /// The dispatch impl for this pair under `tag`.
    fn emit(&self, krate: &syn::Ident, tag: &syn::Ident) -> syn::Result<TokenStream> {
        use syn::BinOp::*;
        let (trait_, method, std, assign) = match self.op {
            Add(_) => ("AddRhs", "add_rhs", "Add", false),
            Sub(_) => ("SubRhs", "sub_rhs", "Sub", false),
            Mul(_) => ("MulRhs", "mul_rhs", "Mul", false),
            Div(_) => ("DivRhs", "div_rhs", "Div", false),
            Rem(_) => ("RemRhs", "rem_rhs", "Rem", false),
            AddAssign(_) => ("AddAssignRhs", "add_assign_rhs", "AddAssign", true),
            SubAssign(_) => ("SubAssignRhs", "sub_assign_rhs", "SubAssign", true),
            MulAssign(_) => ("MulAssignRhs", "mul_assign_rhs", "MulAssign", true),
            DivAssign(_) => ("DivAssignRhs", "div_assign_rhs", "DivAssign", true),
            RemAssign(_) => ("RemAssignRhs", "rem_assign_rhs", "RemAssign", true),
            _ => {
                return Err(syn::Error::new(
                    self.op.span(),
                    "a `#[passthrough(..)]` pair is one of `+ - * / %`, or `+= -= *= /= %=`",
                ));
            }
        };
        let trait_ = syn::Ident::new(trait_, self.op.span());
        let method = syn::Ident::new(method, self.op.span());
        let std = syn::Ident::new(std, self.op.span());
        let (
            Pair {
                lhs: a,
                op,
                rhs: b,
                out,
            },
            ..,
        ) = (self,);
        if assign {
            if let Some(out) = out {
                return Err(syn::Error::new(
                    out.span(),
                    "an in-place pair (`A op= B`) has no output; drop the `=> O`",
                ));
            }
            return Ok(quote! {
                impl ::#krate::__private::traits::#trait_<#a, #tag> for #b {
                    #[inline(always)]
                    #[track_caller]
                    fn #method(self, lhs: &mut #a) { *lhs #op self; }
                }
            });
        }
        // The output is whatever the type's own impl says it is, which the
        // projection resolves; `=> O` is accepted for the reader's sake and
        // has to agree, since the body is checked against it.
        let out = match out {
            Some(out) => out.to_token_stream(),
            None => quote!(<#a as ::core::ops::#std<#b>>::Output),
        };
        Ok(quote! {
            impl ::#krate::__private::traits::#trait_<#a, #out, #tag> for #b {
                #[inline(always)]
                #[track_caller]
                fn #method(self, lhs: #a) -> #out { lhs #op self }
            }
        })
    }
}

/// The names a `use` brings into scope, each a type to opt in.
fn use_leaves(tree: &syn::UseTree, out: &mut Vec<syn::Ident>) -> syn::Result<()> {
    match tree {
        syn::UseTree::Path(p) => use_leaves(&p.tree, out),
        syn::UseTree::Name(n) => {
            if n.ident == "self" {
                return Err(syn::Error::new(
                    n.ident.span(),
                    "`self` in a `use` names a module, not a type",
                ));
            }
            out.push(n.ident.clone());
            Ok(())
        }
        syn::UseTree::Rename(r) => {
            out.push(r.rename.clone());
            Ok(())
        }
        syn::UseTree::Glob(g) => Err(syn::Error::new(
            g.star_token.span(),
            "`#[passthrough]` opts in the types a `use` names; a glob names none. List them",
        )),
        syn::UseTree::Group(g) => {
            for t in &g.items {
                use_leaves(t, out)?;
            }
            Ok(())
        }
    }
}

/// The opt-in block for a type from another crate: a private tag, and the
/// marker under it, plus the named pairs. `const _` so the tag is unnameable
/// and each expansion is its own.
fn foreign_block(
    krate: &syn::Ident,
    ty: &TokenStream,
    pairs: &[Pair],
    extra: TokenStream,
) -> syn::Result<TokenStream> {
    let tag = syn::Ident::new("__ReassocOptIn", Span::call_site());
    let pair_impls = pairs
        .iter()
        .map(|p| p.emit(krate, &tag))
        .collect::<syn::Result<Vec<_>>>()?;
    Ok(quote! {
        const _: () = {
            pub struct #tag;
            impl ::#krate::__private::traits::OptInTag for #tag {}
            impl ::#krate::__private::traits::Passthrough<#tag> for #ty {}
            #(#pair_impls)*
            #extra
        };
    })
}

pub fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let krate = syn::Ident::new(&crate::krate::name(), Span::call_site());
    let pairs: Vec<Pair> = if attr.is_empty() {
        Vec::new()
    } else {
        Punctuated::<Pair, Token![,]>::parse_terminated
            .parse2(attr)?
            .into_iter()
            .collect()
    };
    let no_pairs = |what: &str| -> syn::Result<()> {
        match pairs.first() {
            Some(p) => Err(syn::Error::new(
                p.lhs.span(),
                format!(
                    "a `#[passthrough(..)]` pair is for a primitive on the left of a type from \
                     another crate; {what}"
                ),
            )),
            None => Ok(()),
        }
    };

    let item: syn::Item = syn::parse2(item.clone()).map_err(|_| {
        syn::Error::new(
            Span::call_site(),
            "`#[passthrough]` goes on the item that introduces a type: its definition, the \
             `use` or `type` alias that names one from another crate, or its `impl` of an \
             `#[algebraic_float]` trait",
        )
    })?;

    match &item {
        // A type of yours: the marker, with the type's generics.
        syn::Item::Struct(syn::ItemStruct {
            ident, generics, ..
        })
        | syn::Item::Enum(syn::ItemEnum {
            ident, generics, ..
        })
        | syn::Item::Union(syn::ItemUnion {
            ident, generics, ..
        }) => {
            no_pairs("a type of yours has that through the blankets already")?;
            let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
            Ok(quote! {
                #item
                impl #impl_generics ::#krate::__private::traits::Passthrough for #ident #ty_generics #where_clause {}
            })
        }

        // A type from another crate, by the name the `use` gives it.
        syn::Item::Use(u) => {
            let mut leaves = Vec::new();
            use_leaves(&u.tree, &mut leaves)?;
            if leaves.len() > 1 && !pairs.is_empty() {
                return Err(syn::Error::new(
                    pairs[0].lhs.span(),
                    "pairs name one type; this `use` brings in several. Split it",
                ));
            }
            let blocks = leaves
                .iter()
                .map(|leaf| foreign_block(&krate, &leaf.to_token_stream(), &pairs, quote!()))
                .collect::<syn::Result<Vec<_>>>()?;
            Ok(quote! { #item #(#blocks)* })
        }

        // An instantiation of a generic foreign type, or any foreign type
        // under a name of yours.
        syn::Item::Type(t) => {
            if !t.generics.params.is_empty() {
                return Err(syn::Error::new(
                    t.generics.span(),
                    "`#[passthrough]` on a `type` alias opts in the one type it names; an alias \
                     with parameters names none. Write one alias per instantiation",
                ));
            }
            let block = foreign_block(&krate, &t.ty.to_token_stream(), &pairs, quote!())?;
            Ok(quote! { #item #block })
        }

        // A type into an `#[algebraic_float]` trait: the foreign block, plus
        // the marker impl under the trait's tag, which the impl form reaches
        // through the trait's own path.
        syn::Item::Impl(i) => {
            let Some((path, _)) = &i.trait_ else {
                return Err(syn::Error::new_spanned(
                    &i.self_ty,
                    "`#[passthrough]` on an `impl` opts the type into an `#[algebraic_float]` \
                     trait, and this `impl` block implements no trait",
                ));
            };
            let mut tag_path = path.clone();
            let last = tag_path.segments.last_mut().expect("a path has a segment");
            last.ident = syn::Ident::new(&trait_tag(&last.ident).to_string(), last.ident.span());
            last.arguments = syn::PathArguments::None;
            let (impl_generics, _, where_clause) = i.generics.split_for_impl();
            let self_ty = &i.self_ty;
            let tag = syn::Ident::new("__ReassocOptIn", Span::call_site());
            let pair_impls = pairs
                .iter()
                .map(|p| p.emit(&krate, &tag))
                .collect::<syn::Result<Vec<_>>>()?;
            Ok(quote! {
                const _: () = {
                    pub struct #tag;
                    impl ::#krate::__private::traits::OptInTag for #tag {}
                    impl #impl_generics ::#krate::__private::traits::Passthrough<#tag> for #self_ty #where_clause {}
                    impl #impl_generics ::#krate::__private::AlgebraicFloat<#tag_path> for #self_ty #where_clause {
                        type Tag = #tag;
                    }
                    #(#pair_impls)*
                };
                #item
            })
        }

        syn::Item::Trait(t) => Err(syn::Error::new(
            t.ident.span(),
            "`#[passthrough]` opts a type in; a float trait is marked with `#[algebraic_float]`",
        )),
        other => Err(syn::Error::new(
            other.span(),
            "`#[passthrough]` goes on the item that introduces a type: its definition, the \
             `use` or `type` alias that names one from another crate, or its `impl` of an \
             `#[algebraic_float]` trait",
        )),
    }
}
