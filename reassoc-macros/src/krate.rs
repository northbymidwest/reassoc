//! Resolving the path the generated code should use to reach `reassoc`.

use proc_macro2::TokenStream;
use quote::quote;

/// The absolute path to the facade crate, as generated code should spell it.
///
/// By default this is simply `::reassoc`, which is correct unless the consumer
/// renamed the dependency in their `Cargo.toml`. A proc macro cannot see the
/// path it was invoked through, so the only way to learn a rename is to read
/// the consumer's manifest — which is what `proc-macro-crate` does, at the cost
/// of pulling in a TOML parser. That is gated behind `resolve-crate-name`
/// rather than imposed on everyone, since renaming is rare.
pub fn path() -> TokenStream {
    #[cfg(feature = "resolve-crate-name")]
    {
        use proc_macro_crate::{FoundCrate, crate_name};
        match crate_name("reassoc") {
            Ok(FoundCrate::Itself) => return quote!(crate),
            Ok(FoundCrate::Name(name)) => {
                let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
                return quote!(::#ident);
            }
            // Not resolvable (e.g. expanding outside a cargo build); the
            // unrenamed path is the best guess and matches the default.
            Err(_) => {}
        }
    }
    quote!(::reassoc)
}
