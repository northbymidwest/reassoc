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

    // The heterogeneous pairs take references on either side too, matching
    // the forward_ref impls `core` provides for them.
    assert_eq!(mul(&d, 3u32), Duration::from_secs(6));
    assert_eq!(mul(d, &3u32), Duration::from_secs(6));
    assert_eq!(mul(&d, &3u32), Duration::from_secs(6));
    assert_eq!(mul(&3u32, d), Duration::from_secs(6));
}

/// An operator whose output is not its left operand.
///
/// `passthrough!` compares the two types as written and declares the output
/// only when they differ — the blanket assumption covers the ordinary case, and
/// declaring it there too would collide with it. Nothing extra is written here,
/// which is the point.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Ray([f64; 2]);

impl core::ops::Mul for Ray {
    type Output = f64;
    fn mul(self, o: Ray) -> f64 {
        self.0[0] * o.0[0] + self.0[1] * o.0[1]
    }
}

passthrough!(mul: Ray, Ray => f64);

#[allow(clippy::needless_borrows_for_generic_args)] // borrows are deliberate
#[test]
fn an_output_that_is_not_the_left_operand() {
    let (u, v) = (Ray([1.0, 2.0]), Ray([3.0, 4.0]));
    assert_eq!(mul(u, v), 11.0);
    assert_eq!(mul(&u, v), 11.0);
    assert_eq!(mul(u, &v), 11.0);
    assert_eq!(mul(&u, &v), 11.0);
}

/// Two heterogeneous opt-ins on one left type, both yielding a type that is
/// not the left operand. The output trait names the right operand, so these
/// are distinct impls; keyed on the left type alone they were the same impl
/// twice, `E0119`.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Q(f64);
#[derive(Debug, Clone, Copy, PartialEq)]
struct R(f64);

impl core::ops::Mul<Q> for Q {
    type Output = f64;
    fn mul(self, o: Q) -> f64 {
        self.0 * o.0
    }
}
impl core::ops::Mul<R> for Q {
    type Output = f64;
    fn mul(self, o: R) -> f64 {
        self.0 * o.0
    }
}

passthrough!(mul: Q, Q => f64);
passthrough!(mul: Q, R => f64);

#[allow(clippy::needless_borrows_for_generic_args)] // borrows are deliberate
#[test]
fn two_opt_ins_with_the_same_foreign_output() {
    let (q, r) = (Q(2.0), R(3.0));
    assert_eq!(mul(q, q), 4.0);
    assert_eq!(mul(q, r), 6.0);
    assert_eq!(mul(&q, &r), 6.0);
    assert_eq!(mul(q, &r), 6.0);
}

/// A non-`Copy` type opts out of the reference impls.
#[derive(Debug, Clone, PartialEq, reassoc::Passthrough)]
#[passthrough(add, add_assign, no_refs)]
struct Owned(String);

impl core::ops::Add for Owned {
    type Output = Owned;
    fn add(self, o: Owned) -> Owned {
        Owned(self.0 + &o.0)
    }
}
impl core::ops::AddAssign for Owned {
    fn add_assign(&mut self, o: Owned) {
        self.0 += &o.0;
    }
}

/// A non-`Copy` type's `+=` through a `&mut` or an index goes through its own
/// `AddAssign`, declared with `add_assign` on the derive or
/// `passthrough!(add_assign: ..)`. A `Copy` type needs neither: its `+=` is
/// formed from `+`.
#[test]
fn non_copy_compound_assignment_in_place() {
    use reassoc::algebraic;
    #[algebraic]
    fn bump(v: &mut [Owned], suffix: Owned) {
        v[0] += suffix;
    }
    let mut v = [Owned("a".into())];
    bump(&mut v, Owned("b".into()));
    assert_eq!(v[0], Owned("ab".into()));
}

/// The macro form, with a right-hand type that is itself a reference: the `&`
/// arm routes to the value form, so no `&&str` impl and no lifetime to name.
struct Label(String);
impl core::ops::Add<&str> for Label {
    type Output = Label;
    fn add(self, o: &str) -> Label {
        Label(self.0 + o)
    }
}
impl core::ops::AddAssign<&str> for Label {
    fn add_assign(&mut self, o: &str) {
        self.0 += o;
    }
}
passthrough!(add: Label, &str => Label);
passthrough!(add_assign: Label, &str);

#[test]
fn reference_right_operand_takes_the_value_form() {
    use reassoc::algebraic;
    #[algebraic]
    fn bump(v: &mut [Label], s: &str) -> Label {
        v[0] += s;
        Label("x".into()) + s
    }
    let mut v = [Label("a".into())];
    let r = bump(&mut v, "!");
    assert_eq!((v[0].0.as_str(), r.0.as_str()), ("a!", "x!"));
}

/// A `Copy` type opted in through the per-operator form alone still gets `*=`
/// through an index, formed from `*`.
#[test]
fn copy_per_operator_opt_in_synthesises_compound_assignment() {
    use reassoc::algebraic;
    #[algebraic]
    fn scale(v: &mut [Scaled], k: u32) {
        v[0] *= k;
    }
    let mut v = [Scaled(3)];
    scale(&mut v, 4);
    assert_eq!(v[0], Scaled(12));
}

#[test]
fn non_copy_types_can_opt_out_of_references() {
    assert_eq!(
        add(Owned("a".into()), Owned("b".into())),
        Owned("ab".into())
    );
}

// ---- derive robustness ----

/// A `where` clause with a trailing comma is what rustfmt writes for any
/// multi-line bound list; the derive must append its own bounds to it without
/// producing `, ,`.
#[derive(Debug, Clone, Copy, PartialEq, reassoc::Passthrough)]
#[passthrough(add)]
struct Bounded<T>
where
    T: Copy,
{
    a: T,
    b: T,
}

impl<T> core::ops::Add for Bounded<T>
where
    T: Copy + core::ops::Add<Output = T>,
{
    type Output = Bounded<T>;
    fn add(self, o: Bounded<T>) -> Bounded<T> {
        Bounded {
            a: self.a + o.a,
            b: self.b + o.b,
        }
    }
}

#[test]
fn derive_accepts_a_where_clause_with_a_trailing_comma() {
    let p = Bounded { a: 1.0f32, b: 2.0 };
    assert_eq!(add(p, p), Bounded { a: 2.0, b: 4.0 });
}

/// Naming only in-place forms must not also opt the type into all five binary
/// operators: this type has `AddAssign` and nothing else, and native `+=` on
/// it is fine.
#[derive(Debug, Clone, PartialEq, reassoc::Passthrough)]
#[passthrough(add_assign, no_refs)]
struct InPlaceOnly(String);

impl core::ops::AddAssign for InPlaceOnly {
    fn add_assign(&mut self, o: InPlaceOnly) {
        self.0 += &o.0;
    }
}

#[test]
fn derive_with_only_in_place_forms_emits_only_those() {
    use reassoc::algebraic;
    #[algebraic]
    fn go(v: &mut [InPlaceOnly], t: InPlaceOnly) {
        v[0] += t;
    }
    let mut v = [InPlaceOnly("a".into())];
    go(&mut v, InPlaceOnly("b".into()));
    assert_eq!(v[0], InPlaceOnly("ab".into()));
}

// ---- scaling a user vector by a float literal, the commonest kernel shape ----

#[derive(Debug, Clone, Copy, PartialEq)]
struct V2(f32, f32);
impl core::ops::Add for V2 {
    type Output = V2;
    fn add(self, o: V2) -> V2 {
        V2(self.0 + o.0, self.1 + o.1)
    }
}
impl core::ops::Mul<f32> for V2 {
    type Output = V2;
    fn mul(self, k: f32) -> V2 {
        V2(self.0 * k, self.1 * k)
    }
}
impl core::ops::Mul<V2> for f32 {
    type Output = V2;
    fn mul(self, v: V2) -> V2 {
        V2(self * v.0, self * v.1)
    }
}
impl core::ops::Div<f32> for V2 {
    type Output = V2;
    fn div(self, k: f32) -> V2 {
        V2(self.0 / k, self.1 / k)
    }
}
passthrough!(add: V2, V2 => V2);
passthrough!(mul: V2, f32 => V2);
passthrough!(mul: f32, V2 => V2);
passthrough!(div: V2, f32 => V2);

/// An unsuffixed float literal on either side of a user vector must infer to
/// the scalar type the opt-in names, through the same-type opt-in sitting
/// beside it, on both binary and compound forms.
#[test]
fn float_literal_scalars_infer_against_a_user_vector() {
    use reassoc::algebraic;
    #[algebraic]
    fn go(v: V2, k: f32) -> (V2, V2, V2, V2, V2) {
        let mut acc = v;
        acc *= 2.0;
        acc /= 4.0;
        (v * 2.0, 2.0 * v, v * k, (v + v) * 0.5, acc)
    }
    let v = V2(1.0, 2.0);
    assert_eq!(
        go(v, 3.0),
        (V2(2.0, 4.0), V2(2.0, 4.0), V2(3.0, 6.0), v, V2(0.5, 1.0))
    );
}
