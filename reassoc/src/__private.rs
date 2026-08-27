//! Not a surface. What the macros expand into: `ops`, the functions a
//! rewritten operator becomes; `traits`, the dispatch layer they call
//! through; and the bound `#[algebraic_float]` writes into a trait. All of
//! it is `pub` because generated code in another crate must name it, and
//! none of it is an API: the attributes and `alg!` are the contract, and
//! what they write here is free to change in any release. Under this name,
//! typing a path into it by hand looks like what it is.
//!
//! `ops` and `AlgebraicFloat` are `#[doc(hidden)]`; `traits` is not, and
//! that is measured, not an oversight. rustc stops trimming paths for items
//! under a hidden module: hide `traits` and every "other types implement"
//! line in a diagnostic reads `reassoc::__private::traits::AddRhs<..>` in
//! place of `AddRhs<..>` (`tests/ui`, and `docs/design.md`). The `ops`
//! functions and the marker were already spelled in full, so hiding those
//! changes no snapshot. Visible, `traits` also stays inside what
//! `cargo-semver-checks` compares.

// Hidden: nothing here is reached except through the macros, and the paths
// rustc prints for these were never trimmed (see the module doc).
#[doc(hidden)]
pub mod ops;
// Not hidden: see the module doc.
pub mod traits;

use traits::{
    AddAssignRhs, AddRhs, DivAssignRhs, DivRhs, FloatTag, MulAssignRhs, MulRhs, RemAssignRhs,
    RemRhs, SubAssignRhs, SubRhs,
};

/// The bound `#[algebraic_float]` appends to a user's float trait:
/// "some type the dispatch layer can rewrite arithmetic on". Generic code
/// bounded on that trait reaches every operator through the supertraits
/// here, with the dispatch tag carried as an associated type so that each
/// implementor names the impls it already has: the primitive floats their
/// algebraic ones under `FloatTag`, an opted-in type the marker blankets
/// under its own tag. Nothing is implemented for `f32` that it did not
/// have, which is what keeps concrete float code at exactly one candidate.
///
/// `X` is an orphan slot, never a dispatch tag. The attribute emits a
/// hidden type beside the trait it marks and writes `AlgebraicFloat<That>`
/// as the bound, so that the user's crate may implement this trait for a
/// type from another crate (a bignum), which the orphan rule permits only
/// when a local type appears in the trait's parameters. The primitive
/// impls are generic over `X` and serve every marked trait.
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a primitive float, and is not opted into this \
               `#[algebraic_float]` trait",
    label = "not `f32` or `f64`, and no `#[reassoc::passthrough]` on its `impl`",
    note = "a primitive float needs nothing; any other type is opted in by putting \
            `#[reassoc::passthrough]` on its `impl` of the marked trait, which needs the type \
            to have all five operators (`+ - * / %` and their `op=` forms)",
    note = "a type can implement one marked trait, and that `impl` is its one opt-in",
    note = "if the bound is `reassoc::AlgebraicFloat` itself, that is the primitive floats \
            only and cannot be extended: bound on a float trait of your own carrying \
            `#[algebraic_float]` instead"
)]
pub trait AlgebraicFloat<X = ()>:
    Sized
    + AddRhs<Self, Self, Self::Tag>
    + SubRhs<Self, Self, Self::Tag>
    + MulRhs<Self, Self, Self::Tag>
    + DivRhs<Self, Self, Self::Tag>
    + RemRhs<Self, Self, Self::Tag>
    + AddAssignRhs<Self, Self::Tag>
    + SubAssignRhs<Self, Self::Tag>
    + MulAssignRhs<Self, Self::Tag>
    + DivAssignRhs<Self, Self::Tag>
    + RemAssignRhs<Self, Self::Tag>
{
    type Tag;
}

impl<X> AlgebraicFloat<X> for f32 {
    type Tag = FloatTag;
}
impl<X> AlgebraicFloat<X> for f64 {
    type Tag = FloatTag;
}
#[cfg(feature = "f16")]
impl<X> AlgebraicFloat<X> for f16 {
    type Tag = FloatTag;
}
#[cfg(feature = "f128")]
impl<X> AlgebraicFloat<X> for f128 {
    type Tag = FloatTag;
}
