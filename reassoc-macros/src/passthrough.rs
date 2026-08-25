//! `#[derive(Passthrough)]`: opt a type into the dispatch layer at its
//! definition, rather than with a separate `passthrough!` invocation. The
//! whole of it is one marker impl; the blanket impls in `reassoc::traits`
//! route every `std::ops` operator the type has.

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    // `#[passthrough(add, mul, no_refs, ..)]` used to name which operators to
    // emit; nothing is emitted per operator any more, so the list has no
    // meaning, so say so rather than silently ignore it.
    if let Some(attr) = input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("passthrough"))
    {
        return Err(syn::Error::new_spanned(
            attr,
            "`#[passthrough(..)]` takes no parameters any more: every operator the type \
             implements is dispatched, references included wherever the type implements \
             them. Remove the attribute",
        ));
    }
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let krate = crate::krate::path();
    Ok(quote! {
        impl #impl_generics #krate::traits::Passthrough for #name #ty_generics #where_clause {}
    })
}
