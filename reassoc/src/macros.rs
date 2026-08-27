/// One dispatch impl written out, for the standard types this crate opts in
/// with a foreign right operand (`u8 / NonZero<u8>`), where the marker
/// blankets do not reach. Internal: a user's equivalent is a pair in
/// `#[passthrough(..)]`'s arguments.
macro_rules! pair {
    ($rhs:ident, $method:ident, $op:tt, $a:ty, $b:ty => $o:ty) => {
        impl $crate::__private::traits::$rhs<$a, $o> for $b {
            #[inline(always)]
            #[track_caller]
            fn $method(self, lhs: $a) -> $o { lhs $op self }
        }
    };
    ($assign:ident, $method:ident, $op:tt, $a:ty, $b:ty) => {
        impl $crate::__private::traits::$assign<$a> for $b {
            #[inline(always)]
            #[track_caller]
            fn $method(self, lhs: &mut $a) { *lhs $op self }
        }
    };
}

/// Marks an expression as strictly IEEE, using ordinary operators instead of
/// algebraic dispatch.
///
/// An identity macro, taking an expression or a brace-delimited statement
/// sequence. It works as an escape hatch inside `alg!` and `#[algebraic]`
/// because the rewriter never descends into a macro's token stream unless the
/// macro is one of the std ones whose arguments are expressions (`assert!`,
/// `println!`, `vec!`, ..), and `strict!` is not, so it is opaque even as an
/// argument of those. Like any macro it must be in scope: `use
/// reassoc::strict;` or `reassoc::strict!(..)`.
///
/// This exists to protect algorithms that depend on exact rounding, most
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
