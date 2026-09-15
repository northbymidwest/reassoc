//! The name-matched methods. `.sum()` and `.product()` inside an algebraic
//! scope go through the dispatch layer: a float output folds with the
//! algebraic operators, every other output through its own
//! `core::iter::Sum` / `Product`. `Summed` implements the two dispatch
//! traits and not `core::iter::Sum`, so each call on it below compiles only
//! because the call was rewritten; the opt-out direction is
//! `tests/ui/reductions_false_opts_out.rs`. `.powi(n)` on a float is
//! square-and-multiply with the algebraic multiply; its trait is sealed to
//! the floats, so the rewrite is observable through the codegen matrix
//! (`sugar_powi_f32` against its strict control) and
//! `tests/ui/powi_on_own_method.rs`, and the tests here pin values and
//! receiver shapes.
#![allow(clippy::all)]

use core::num::Wrapping;
use core::time::Duration;
use reassoc::__private::traits::{MulRhs, ProductOf, SumOf};
use reassoc::{alg, algebraic, passthrough};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Summed(f32);
impl SumOf<Summed> for Summed {
    fn sum_of<I: Iterator<Item = Summed>>(iter: I) -> Summed {
        Summed(iter.map(|s| s.0).sum())
    }
}
impl<'a> SumOf<&'a Summed> for Summed {
    fn sum_of<I: Iterator<Item = &'a Summed>>(iter: I) -> Summed {
        Summed(iter.map(|s| s.0).sum())
    }
}
impl ProductOf<Summed> for Summed {
    fn product_of<I: Iterator<Item = Summed>>(iter: I) -> Summed {
        Summed(iter.map(|s| s.0).product())
    }
}
impl<'a> ProductOf<&'a Summed> for Summed {
    fn product_of<I: Iterator<Item = &'a Summed>>(iter: I) -> Summed {
        Summed(iter.map(|s| s.0).product())
    }
}
impl MulRhs<Summed, Summed> for Summed {
    fn mul_rhs(self, lhs: Summed) -> Summed {
        Summed(lhs.0 * self.0)
    }
}

#[test]
fn sum_actually_dispatches() {
    let v = [Summed(1.0), Summed(2.0), Summed(4.0)];
    assert_eq!(alg!(v.iter().sum::<Summed>()), Summed(7.0));
    assert_eq!(alg!(v.iter().copied().sum::<Summed>()), Summed(7.0));
    let annotated: Summed = alg!(v.iter().sum());
    assert_eq!(annotated, Summed(7.0));
}

#[test]
fn product_actually_dispatches() {
    let v = [Summed(1.0), Summed(2.0), Summed(4.0)];
    assert_eq!(alg!(v.iter().product::<Summed>()), Summed(8.0));
    assert_eq!(alg!(v.iter().copied().product::<Summed>()), Summed(8.0));
    let annotated: Summed = alg!(v.iter().product());
    assert_eq!(annotated, Summed(8.0));
}

#[algebraic]
fn total(v: &[Summed]) -> Summed {
    v.iter().sum()
}

#[algebraic]
fn scaled_total(v: &[Summed], k: Summed) -> Summed {
    // The receiver is rewritten before the call: the `*` inside the closure
    // has no `std::ops` to fall back on either.
    v.iter().map(|&x| x * k).sum()
}

#[test]
fn the_attribute_form_and_a_rewritten_receiver() {
    let v = [Summed(1.0), Summed(2.0), Summed(4.0)];
    assert_eq!(total(&v), Summed(7.0));
    assert_eq!(scaled_total(&v, Summed(2.0)), Summed(14.0));
}

#[test]
fn float_values_and_identities_match_core() {
    let v = [1.5f32, 2.5, 4.0];
    assert_eq!(alg!(v.iter().sum::<f32>()), 8.0);
    assert_eq!(alg!(v.iter().copied().sum::<f32>()), 8.0);
    assert_eq!(alg!(v.iter().product::<f32>()), 15.0);
    let w = [1.5f64, 2.5, 4.0];
    assert_eq!(alg!(w.iter().sum::<f64>()), 8.0);
    assert_eq!(alg!(w.into_iter().product::<f64>()), 15.0);
    // core's identities: `-0.0` for a sum, so a sum of negative zeros is
    // negative zero, and `1.0` for a product.
    let empty_sum: f32 = alg!(core::iter::empty::<f32>().sum());
    assert!(empty_sum == 0.0 && empty_sum.is_sign_negative());
    let neg: f32 = alg!([-0.0f32, -0.0].iter().sum());
    assert!(neg.is_sign_negative());
    let empty_product: f64 = alg!(core::iter::empty::<f64>().product());
    assert_eq!(empty_product, 1.0);
}

#[test]
fn the_output_type_is_inferred_as_natively() {
    fn from_return(v: &[f32]) -> f32 {
        alg!(v.iter().sum())
    }
    assert_eq!(from_return(&[1.0, 2.0]), 3.0);
    // An unsuffixed literal iterator resolves from the annotated output.
    let s: f64 = alg!([1.0, 2.0, 3.0].into_iter().sum());
    assert_eq!(s, 6.0);
    let s: f64 = alg!([1.0, 2.0, 3.0].iter().sum());
    assert_eq!(s, 6.0);
    // The sum then takes part in further inference like any operand.
    let t = alg!([1.0f32, 2.0].iter().sum::<f32>() * 2.0);
    assert_eq!(t, 6.0);
}

#[test]
fn integers_and_std_types_use_their_own_sum_and_product() {
    let v = [1i32, 2, 3];
    assert_eq!(alg!(v.iter().sum::<i32>()), 6);
    assert_eq!(alg!(v.iter().copied().product::<i32>()), 6);
    let big = vec![u64::MAX / 2, 1];
    assert_eq!(alg!(big.into_iter().sum::<u64>()), u64::MAX / 2 + 1);
    let n: usize = alg!((0..10).sum());
    assert_eq!(n, 45);
    let w = [Wrapping(200u8), Wrapping(100)];
    assert_eq!(alg!(w.iter().sum::<Wrapping<u8>>()), Wrapping(44));
    let d = [Duration::from_millis(500), Duration::from_millis(750)];
    assert_eq!(
        alg!(d.iter().sum::<Duration>()),
        Duration::from_millis(1250)
    );
    // `Option` and `Result`, which core sums into, short-circuiting.
    let some = [Some(1.0f32), Some(2.0)];
    assert_eq!(alg!(some.iter().copied().sum::<Option<f32>>()), Some(3.0));
    let none = [Some(1.0f32), None];
    assert_eq!(alg!(none.into_iter().sum::<Option<f32>>()), None);
    let ok: Result<i32, &str> = alg!([Ok(2), Ok(3)].into_iter().product());
    assert_eq!(ok, Ok(6));
    let err: Result<i32, &str> = alg!([Ok(2), Err("no")].into_iter().sum());
    assert_eq!(err, Err("no"));
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[passthrough]
struct Money(i64);
impl core::iter::Sum for Money {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Money {
        Money(iter.map(|m| m.0).sum())
    }
}
impl<'a> core::iter::Sum<&'a Money> for Money {
    fn sum<I: Iterator<Item = &'a Money>>(iter: I) -> Money {
        Money(iter.map(|m| m.0).sum())
    }
}

#[test]
fn an_opted_in_type_uses_its_own_sum() {
    let v = [Money(5), Money(7)];
    assert_eq!(alg!(v.iter().sum::<Money>()), Money(12));
    assert_eq!(alg!(v.into_iter().sum::<Money>()), Money(12));
}

#[test]
fn a_sum_method_with_arguments_is_not_iterator_sum() {
    struct Grid(Vec<f32>);
    impl Grid {
        fn sum(&self, from: usize) -> f32 {
            self.0[from..].iter().sum()
        }
    }
    let g = Grid(vec![1.0, 2.0, 3.0]);
    assert_eq!(alg!(g.sum(1)), 5.0);
}

/// Method syntax reborrows a `&mut` iterator receiver; a function argument
/// would move it. Both shapes compile in plain Rust, so both must here.
#[test]
fn a_mutable_iterator_receiver_is_reborrowed_not_moved() {
    let mut it = [1.0f32, 2.0, 4.0].into_iter();
    let r = &mut it;
    let first: f32 = alg!(r.sum());
    let rest: f32 = alg!(r.sum());
    assert_eq!((first, rest), (7.0, 0.0));

    struct Stream<'a> {
        it: &'a mut core::slice::Iter<'a, f32>,
    }
    impl Stream<'_> {
        #[algebraic]
        fn drain(&mut self) -> f32 {
            self.it.sum()
        }
    }
    let v = [1.0f32, 2.0];
    let mut it = v.iter();
    let mut s = Stream { it: &mut it };
    assert_eq!(s.drain(), 3.0);
    assert_eq!(s.drain(), 0.0);
}

#[test]
fn powi_values_match_core() {
    assert_eq!(alg!(3.0f32.powi(3)), 27.0);
    assert_eq!(alg!(2.0f64.powi(-2)), 0.25);
    assert_eq!(alg!(2.0f32.powi(10)), 1024.0);
    assert_eq!(alg!(0.0f32.powi(0)), 1.0);
    assert_eq!(alg!(f32::NAN.powi(0)), 1.0);
    assert_eq!(alg!(2.0f32.powi(i32::MIN)), 0.0);
    assert_eq!(alg!(1.5f64.powi(1)), 1.5);
    let n = 5;
    assert_eq!(alg!(2.0f32.powi(n)), 32.0);
}

#[test]
fn powi_auto_derefs_its_receiver_as_natively() {
    let x = 3.0f32;
    let r = &x;
    let rr = &r;
    assert_eq!(alg!(r.powi(2)), 9.0);
    assert_eq!(alg!(rr.powi(2)), 9.0);
    let mut y = 2.0f64;
    let m = &mut y;
    assert_eq!(alg!(m.powi(3)), 8.0);
    // The everyday shape: a borrowed item inside `map`, then a sum.
    let v = [1.0f32, 2.0, 3.0];
    assert_eq!(alg!(v.iter().map(|x| x.powi(2)).sum::<f32>()), 14.0);
}

/// `powi = false` is its own switch: with it, a `powi` of a type's own is
/// called as written (with the rule on, this does not compile:
/// `tests/ui/powi_on_own_method.rs`), while the sum beside it is still
/// rewritten.
#[test]
fn powi_false_leaves_a_types_own_powi_alone() {
    #[derive(Clone, Copy, Debug, PartialEq)]
    #[passthrough]
    struct Gain(f32);
    impl Gain {
        fn powi(self, n: i32) -> Gain {
            Gain(self.0.powi(n))
        }
    }
    #[algebraic(powi = false)]
    fn f(g: Gain, v: &[Summed]) -> (Gain, Summed) {
        (g.powi(2), v.iter().sum())
    }
    assert_eq!(
        f(Gain(3.0), &[Summed(1.0), Summed(2.0)]),
        (Gain(9.0), Summed(3.0))
    );
}

#[test]
fn a_powi_with_another_arity_is_not_f32_powi() {
    struct Poly(Vec<f32>);
    impl Poly {
        fn powi(&self, n: i32, at: f32) -> f32 {
            self.0.iter().sum::<f32>().powi(n) * at
        }
    }
    let p = Poly(vec![1.0, 2.0]);
    assert_eq!(alg!(p.powi(2, 0.5)), 4.5);
}
