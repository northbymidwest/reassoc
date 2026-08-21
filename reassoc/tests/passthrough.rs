use core::time::Duration;
use reassoc::ops::{add, mul};
use reassoc::{passthrough, strict};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec3(f32, f32, f32);

impl core::ops::Add for Vec3 {
    type Output = Vec3;
    fn add(self, o: Vec3) -> Vec3 {
        Vec3(self.0 + o.0, self.1 + o.1, self.2 + o.2)
    }
}
impl core::ops::Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, o: Vec3) -> Vec3 {
        Vec3(self.0 - o.0, self.1 - o.1, self.2 - o.2)
    }
}
impl core::ops::Mul for Vec3 {
    type Output = Vec3;
    fn mul(self, o: Vec3) -> Vec3 {
        Vec3(self.0 * o.0, self.1 * o.1, self.2 * o.2)
    }
}
impl core::ops::Div for Vec3 {
    type Output = Vec3;
    fn div(self, o: Vec3) -> Vec3 {
        Vec3(self.0 / o.0, self.1 / o.1, self.2 / o.2)
    }
}
impl core::ops::Rem for Vec3 {
    type Output = Vec3;
    fn rem(self, o: Vec3) -> Vec3 {
        Vec3(self.0 % o.0, self.1 % o.1, self.2 % o.2)
    }
}

passthrough!(Vec3);

#[derive(Debug, Clone, Copy, PartialEq)]
struct Scaled(u32);

impl core::ops::Mul<u32> for Scaled {
    type Output = Scaled;
    fn mul(self, n: u32) -> Scaled {
        Scaled(self.0 * n)
    }
}

passthrough!(mul: Scaled, u32 => Scaled);

#[test]
fn same_type_passthrough_covers_all_five_operators() {
    let a = Vec3(1.0, 2.0, 3.0);
    assert_eq!(add(a, a), Vec3(2.0, 4.0, 6.0));
    assert_eq!(mul(a, a), Vec3(1.0, 4.0, 9.0));
}

#[test]
fn heterogeneous_passthrough_covers_one_operator() {
    assert_eq!(mul(Scaled(3), 4u32), Scaled(12));
}

#[test]
fn strict_is_an_identity_macro() {
    let (t, sum, y) = (3.0f32, 2.0f32, 1.0f32);
    assert_eq!(strict!((t - sum) - y), 0.0);
    assert_eq!(strict!(Duration::from_secs(1)), Duration::from_secs(1));
}

// ---- std wrappers, covered without any opt-in ----

#[test]
fn wrapping_and_saturating_need_no_opt_in() {
    use core::num::{Saturating, Wrapping};
    assert_eq!(add(Wrapping(250u8), Wrapping(10u8)), Wrapping(4u8)); // wraps
    assert_eq!(add(Saturating(250u8), Saturating(10u8)), Saturating(255u8)); // saturates
    assert_eq!(mul(Wrapping(3i64), Wrapping(4i64)), Wrapping(12i64));
    assert_eq!(
        reassoc::ops::rem(Wrapping(9u32), Wrapping(4u32)),
        Wrapping(1u32)
    );
    assert_eq!(
        reassoc::ops::div(Saturating(9i16), Saturating(2i16)),
        Saturating(4i16)
    );
}

// ---- #[derive(Passthrough)] ----

#[derive(Debug, Clone, Copy, PartialEq, reassoc::Passthrough)]
struct Derived(f32);

macro_rules! derived_ops {
    ($($t:ident, $m:ident, $op:tt);* $(;)?) => {$(
        impl core::ops::$t for Derived {
            type Output = Derived;
            fn $m(self, o: Derived) -> Derived { Derived(self.0 $op o.0) }
        }
    )*};
}
derived_ops!(Add, add, +; Sub, sub, -; Mul, mul, *; Div, div, /; Rem, rem, %);

#[test]
fn derive_covers_all_five_operators() {
    let (a, b) = (Derived(6.0), Derived(4.0));
    assert_eq!(add(a, b), Derived(10.0));
    assert_eq!(mul(a, b), Derived(24.0));
    assert_eq!(reassoc::ops::sub(a, b), Derived(2.0));
    assert_eq!(reassoc::ops::div(a, b), Derived(1.5));
    assert_eq!(reassoc::ops::rem(a, b), Derived(2.0));
}

/// A type that implements only three operators must name them; deriving all
/// five would fail to compile, since an unsatisfiable `where` bound on a
/// concrete type is a hard error rather than a lazily-checked one.
#[derive(Debug, Clone, Copy, PartialEq, reassoc::Passthrough)]
#[passthrough(add, sub, mul)]
struct Partial(f32);

impl core::ops::Add for Partial {
    type Output = Partial;
    fn add(self, o: Partial) -> Partial {
        Partial(self.0 + o.0)
    }
}
impl core::ops::Sub for Partial {
    type Output = Partial;
    fn sub(self, o: Partial) -> Partial {
        Partial(self.0 - o.0)
    }
}
impl core::ops::Mul for Partial {
    type Output = Partial;
    fn mul(self, o: Partial) -> Partial {
        Partial(self.0 * o.0)
    }
}

#[test]
fn derive_can_select_a_subset() {
    let (a, b) = (Partial(6.0), Partial(4.0));
    assert_eq!(add(a, b), Partial(10.0));
    assert_eq!(mul(a, b), Partial(24.0));
    // `Partial` implements no Div/Rem, and none were derived.
}

/// Generic types work because the generated `where` bound defers to the
/// type's own `core::ops` impl.
#[derive(Debug, Clone, Copy, PartialEq, reassoc::Passthrough)]
#[passthrough(add)]
struct Pair<T>(T, T);

impl<T: core::ops::Add<Output = T>> core::ops::Add for Pair<T> {
    type Output = Pair<T>;
    fn add(self, o: Pair<T>) -> Pair<T> {
        Pair(self.0 + o.0, self.1 + o.1)
    }
}

#[test]
fn derive_works_on_generic_types() {
    assert_eq!(add(Pair(1.0f32, 2.0), Pair(3.0, 4.0)), Pair(4.0f32, 6.0));
    assert_eq!(add(Pair(1u8, 2u8), Pair(3u8, 4u8)), Pair(4u8, 6u8));
}

// ---- reference operands, the iterator shape ----

// The borrows are the point: these exercise the `&T` impls, so clippy's
// suggestion to drop them would delete what is being tested.
#[allow(clippy::needless_borrows_for_generic_args)]
#[test]
fn opted_in_types_accept_reference_operands() {
    // `passthrough!` type
    let xs = [Vec3(2.0, 2.0, 2.0), Vec3(3.0, 3.0, 3.0)];
    let ys = [Vec3(4.0, 4.0, 4.0), Vec3(5.0, 5.0, 5.0)];
    let got: Vec<Vec3> = xs.iter().zip(&ys).map(|(a, b)| mul(a, b)).collect();
    assert_eq!(got, vec![Vec3(8.0, 8.0, 8.0), Vec3(15.0, 15.0, 15.0)]);
    assert_eq!(mul(&xs[0], ys[0]), Vec3(8.0, 8.0, 8.0));
    assert_eq!(mul(xs[0], &ys[0]), Vec3(8.0, 8.0, 8.0));

    // derived type
    let ds = [Derived(2.0), Derived(3.0)];
    let got: Vec<Derived> = ds.iter().zip(&ds).map(|(a, b)| add(a, b)).collect();
    assert_eq!(got, vec![Derived(4.0), Derived(6.0)]);
}

// The borrows are the point: these exercise the `&T` impls, so clippy's
// suggestion to drop them would delete what is being tested.
#[allow(clippy::needless_borrows_for_generic_args)]
#[test]
fn std_wrappers_accept_reference_operands() {
    use core::num::{Saturating, Wrapping};
    use core::time::Duration;
    let a = Wrapping(3u32);
    let b = Saturating(250u8);
    let d = Duration::from_secs(2);
    assert_eq!(add(&a, &a), Wrapping(6u32));
    assert_eq!(add(&b, Saturating(10u8)), Saturating(255u8));
    assert_eq!(add(&d, &d), Duration::from_secs(4));
}

/// A non-`Copy` type opts out of the reference impls.
#[derive(Debug, Clone, PartialEq, reassoc::Passthrough)]
#[passthrough(add, no_refs)]
struct Owned(String);

impl core::ops::Add for Owned {
    type Output = Owned;
    fn add(self, o: Owned) -> Owned {
        Owned(self.0 + &o.0)
    }
}

#[test]
fn non_copy_types_can_opt_out_of_references() {
    assert_eq!(
        add(Owned("a".into()), Owned("b".into())),
        Owned("ab".into())
    );
}
