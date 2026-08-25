//! Resolving the path the generated code should use to reach `reassoc`.

use proc_macro2::{Span, TokenStream};
use quote::quote;

/// The name the facade crate goes by in the consumer.
///
/// By default simply `reassoc`, which is correct unless the consumer renamed
/// the dependency in their `Cargo.toml`. A proc macro cannot see the path it
/// was invoked through, so the only way to learn a rename is to read the
/// consumer's manifest, which is what `proc-macro-crate` does, at the cost of
/// pulling in a TOML parser. That is gated behind `resolve-crate-name` rather
/// than imposed on everyone, since renaming is rare.
pub fn name() -> String {
    #[cfg(feature = "resolve-crate-name")]
    {
        use proc_macro_crate::{FoundCrate, crate_name};
        match crate_name("reassoc") {
            // `Itself` means we are expanding somewhere in the `reassoc`
            // package, but its examples, tests, benches and doctests are
            // each their own crate, linking the library by name, so `crate::`
            // there resolves to the wrong root. The library itself never
            // invokes these macros, so naming the crate is right for every
            // case that actually occurs.
            Ok(FoundCrate::Itself) => return "reassoc".to_owned(),
            Ok(FoundCrate::Name(name)) => return name,
            // Not resolvable (e.g. expanding outside a cargo build); the
            // unrenamed name is the best guess and matches the default.
            Err(_) => {}
        }
    }
    "reassoc".to_owned()
}

/// The absolute path to the facade crate as tokens, for `passthrough!`'s
/// derive output.
pub fn path() -> TokenStream {
    let ident = syn::Ident::new(&name(), Span::call_site());
    quote!(::#ident)
}
