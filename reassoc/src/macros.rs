/// Opt a type into `reassoc`'s dispatch layer.
///
/// One line per type. Every operator the type implements — `+ - * / %` with
/// any right-hand type and any output, the `op=` forms, and references
/// wherever the type implements them — is dispatched from then on, exactly
/// as `std::ops` defines it; nothing is listed:
///
/// ```
/// # #[derive(Debug, Clone, Copy, PartialEq)]
/// # struct Vec3(f32, f32, f32);
/// # impl core::ops::Add for Vec3 { type Output = Vec3; fn add(self, o: Vec3) -> Vec3 { Vec3(self.0 + o.0, self.1 + o.1, self.2 + o.2) } }
/// # impl core::ops::Mul<f32> for Vec3 { type Output = Vec3; fn mul(self, k: f32) -> Vec3 { Vec3(self.0 * k, self.1 * k, self.2 * k) } }
/// # impl core::ops::Mul<Vec3> for f32 { type Output = Vec3; fn mul(self, v: Vec3) -> Vec3 { v * self } }
/// use reassoc::{alg, passthrough};
///
/// passthrough!(Vec3);
///
/// let v = Vec3(1.0, 2.0, 3.0);
/// assert_eq!(alg!(v + v * 2.0), Vec3(3.0, 6.0, 9.0));   // Add, Mul<f32>
/// assert_eq!(alg!(0.5 * v), Vec3(0.5, 1.0, 1.5));      // Mul<Vec3> for f32
/// ```
///
/// Forms:
///
/// - `passthrough!(T)` — a type of this crate.
/// - `passthrough!(foreign T)` — a type from another crate (`glam::Vec3`,
///   say). Rust's orphan rule forbids implementing this crate's traits for a
///   foreign type unless the impl names a type of yours, so this form emits
///   one privately and carries it in the traits' tag parameter. Opt a foreign
///   type in **once**, in the binary or one shared crate: two crates opting
///   in the same type give every crate that depends on both an ambiguity
///   error at each use (`docs/limitations.md`). One thing the foreign form
///   cannot do automatically is a *float on the left* of the type (`2.0 *
///   v`); that pair is named explicitly, see the next form.
/// - `passthrough!(mul: f32, glam::Vec3 => glam::Vec3)` and the `foreign`
///   prefix of it — one operator for one pair, written out. Needed only for a
///   float on the left of a foreign type; everything else the first two forms
///   cover. `add_assign: A, B` names an in-place pair the same way.
///
/// The dispatch traits and `ops` functions these expand to are
/// implementation detail, not a surface to write against by hand.
#[macro_export]
macro_rules! passthrough {
    // ---- entry: a type from another crate ----
    //
    // The impls are emitted around a private type of the caller's and carry
    // it in the traits' trailing tag parameter (`traits.rs`). One block per
    // invocation; the name is unlikely rather than hygienic, since
    // `macro_rules!` items are not.
    (foreign $($form:tt)*) => {
        const _: () = {
            struct __ReassocOptIn;
            impl $crate::traits::OptInTag for __ReassocOptIn {}
            $crate::passthrough!(@tag __ReassocOptIn; $($form)*);
        };
    };

    // ---- the forms, with the tag they implement under ----
    (@tag $tag:ty; add: $a:ty, $b:ty => $o:ty) => { $crate::passthrough!(@pair AddRhs, add_rhs, +, $tag, $a, $b, $o); };
    (@tag $tag:ty; sub: $a:ty, $b:ty => $o:ty) => { $crate::passthrough!(@pair SubRhs, sub_rhs, -, $tag, $a, $b, $o); };
    (@tag $tag:ty; mul: $a:ty, $b:ty => $o:ty) => { $crate::passthrough!(@pair MulRhs, mul_rhs, *, $tag, $a, $b, $o); };
    (@tag $tag:ty; div: $a:ty, $b:ty => $o:ty) => { $crate::passthrough!(@pair DivRhs, div_rhs, /, $tag, $a, $b, $o); };
    (@tag $tag:ty; rem: $a:ty, $b:ty => $o:ty) => { $crate::passthrough!(@pair RemRhs, rem_rhs, %, $tag, $a, $b, $o); };

    (@tag $tag:ty; add_assign: $a:ty, $b:ty) => { $crate::passthrough!(@assign AddAssignRhs, add_assign_rhs, +=, $tag, $a, $b); };
    (@tag $tag:ty; sub_assign: $a:ty, $b:ty) => { $crate::passthrough!(@assign SubAssignRhs, sub_assign_rhs, -=, $tag, $a, $b); };
    (@tag $tag:ty; mul_assign: $a:ty, $b:ty) => { $crate::passthrough!(@assign MulAssignRhs, mul_assign_rhs, *=, $tag, $a, $b); };
    (@tag $tag:ty; div_assign: $a:ty, $b:ty) => { $crate::passthrough!(@assign DivAssignRhs, div_assign_rhs, /=, $tag, $a, $b); };
    (@tag $tag:ty; rem_assign: $a:ty, $b:ty) => { $crate::passthrough!(@assign RemAssignRhs, rem_assign_rhs, %=, $tag, $a, $b); };

    // The whole type: the marker, and the blanket impls do the rest.
    (@tag $tag:ty; $t:ty) => {
        impl $crate::traits::Passthrough<$tag> for $t {}
    };

    // Internal: one pair, written out. `#[track_caller]` is free once inlined.
    (@pair $rhs:ident, $method:ident, $op:tt, $tag:ty, $a:ty, $b:ty, $o:ty) => {
        impl $crate::traits::$rhs<$a, $o, $tag> for $b {
            #[inline(always)]
            #[track_caller]
            fn $method(self, lhs: $a) -> $o { lhs $op self }
        }
    };
    (@assign $assign:ident, $method:ident, $op:tt, $tag:ty, $a:ty, $b:ty) => {
        impl $crate::traits::$assign<$a, $tag> for $b {
            #[inline(always)]
            #[track_caller]
            fn $method(self, lhs: &mut $a) { *lhs $op self }
        }
    };

    // Anything else that reached the tagged arms is not a form: say so, rather
    // than let the catch-all below re-wrap it until the recursion limit.
    (@tag $tag:ty; $($form:tt)*) => {
        ::core::compile_error!(::core::concat!(
            "`passthrough!`: no such form: `",
            ::core::stringify!($($form)*),
            "`. The forms are `T`, `foreign T`, `OP: A, B => O`, `OP_assign: A, B` (each of \
             those two optionally prefixed `foreign`), with OP one of add, sub, mul, div, rem"
        ));
    };

    // ---- entry: a type of this crate — under the default tag ----
    ($($form:tt)*) => {
        $crate::passthrough!(@tag (); $($form)*);
    };
}

/// Marks an expression as strictly IEEE, using ordinary operators instead of
/// algebraic dispatch.
///
/// An identity macro, taking an expression or a brace-delimited statement
/// sequence. It works as an escape hatch inside `alg!` and `#[algebraic]`
/// because the rewriter never descends into a macro's token stream unless the
/// macro is one of the std ones whose arguments are expressions (`assert!`,
/// `println!`, `vec!`, ..) — and `strict!` is not, so it is opaque even as an
/// argument of those. Like any macro it must be in scope: `use
/// reassoc::strict;` or `reassoc::strict!(..)`.
///
/// This exists to protect algorithms that depend on exact rounding — most
/// importantly compensated summation, where `(t - sum) - y` is algebraically
/// zero and reassociation would delete it.
#[macro_export]
macro_rules! strict {
    ($e:expr) => {
        $e
    };
    // A statement sequence, with or without a tail expression:
    // `strict! { let y = term - c; let t = sum + y; .. }`. The braces are the
    // macro's own delimiters, so the body arrives as bare statements and is
    // given a block to live in. Tried after the expression arm, so a single
    // expression is never wrapped and `unused_braces` has nothing to say.
    ($($t:tt)*) => {
        { $($t)* }
    };
}
