//! `passthrough!` and `#[derive(Passthrough)]`: one line opts a type in, and
//! every operator it implements (any right-hand type, any output, the
//! in-place forms, references wherever the type implements them) is
//! dispatched from then on, exactly as `std::ops` defines it.
use reassoc::{alg, passthrough, strict};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec3(f32, f32, f32);

macro_rules! vec3_ops {
    ($($t:ident, $m:ident, $op:tt);* $(;)?) => {$(
        impl core::ops::$t for Vec3 {
            type Output = Vec3;
            fn $m(self, o: Vec3) -> Vec3 { Vec3(self.0 $op o.0, self.1 $op o.1, self.2 $op o.2) }
        }
    )*};
}
vec3_ops!(Add, add, +; Sub, sub, -; Mul, mul, *; Div, div, /; Rem, rem, %);

/// `Scaled * u32`: one operator, a foreign right-hand type. Nothing names it:
/// the opt-in covers whatever the type implements.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Scaled(u32);
impl core::ops::Mul<u32> for Scaled {
    type Output = Scaled;
    fn mul(self, n: u32) -> Scaled {
        Scaled(self.0 * n)
    }
}

passthrough!(Vec3);
passthrough!(Scaled);

#[test]
fn one_line_covers_every_operator_the_type_has() {
    let (a, b) = (Vec3(1.0, 2.0, 3.0), Vec3(1.0, 2.0, 3.0));
    assert_eq!(alg!(a + b), Vec3(2.0, 4.0, 6.0));
    assert_eq!(alg!(a - b), Vec3(0.0, 0.0, 0.0));
    assert_eq!(alg!(a * b), Vec3(1.0, 4.0, 9.0));
    assert_eq!(alg!(a / b), Vec3(1.0, 1.0, 1.0));
    assert_eq!(alg!(a % b), Vec3(0.0, 0.0, 0.0));
    let k = 4u32;
    assert_eq!(alg!(Scaled(3) * k), Scaled(12));
}

#[test]
fn strict_is_an_identity_macro() {
    assert_eq!(strict!(1.0f32 + 2.0), 3.0);
    let x: f64 = strict!(0.1 + 0.2);
    assert_eq!(x, 0.1 + 0.2);
}

#[test]
fn wrapping_and_saturating_need_no_opt_in() {
    use core::num::{Saturating, Wrapping};
    let (a, b) = (Wrapping(250u8), Wrapping(10u8));
    assert_eq!(alg!(a + b), Wrapping(4u8));
    assert_eq!(alg!(a * b), Wrapping(196u8));
    let (c, d) = (Saturating(250u8), Saturating(10u8));
    assert_eq!(alg!(c + d), Saturating(255u8));
    assert_eq!(alg!(c - d), Saturating(240u8));
    // And the compound forms, through the wrappers' own `AddAssign`.
    let mut w = a;
    alg!(w += b);
    assert_eq!(w, Wrapping(4u8));
    let mut s = c;
    alg!(s += d);
    assert_eq!(s, Saturating(255u8));
}

// ---- the derive ----

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
    let (a, b) = (Derived(6.0), Derived(3.0));
    assert_eq!(alg!(a + b), Derived(9.0));
    assert_eq!(alg!(a - b), Derived(3.0));
    assert_eq!(alg!(a * b), Derived(18.0));
    assert_eq!(alg!(a / b), Derived(2.0));
    assert_eq!(alg!(a % b), Derived(0.0));
}

/// A type that implements three operators gets three. Nothing is named, and
/// nothing is emitted for the two it lacks, so `Partial / Partial` is rejected
/// exactly as native Rust rejects it (`tests/ui/unsupported_type.rs`).
#[derive(Debug, Clone, Copy, PartialEq, reassoc::Passthrough)]
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
fn derive_covers_exactly_the_operators_the_type_has() {
    let (a, b) = (Partial(6.0), Partial(3.0));
    assert_eq!(alg!(a + b - a * b), Partial(-9.0));
}

/// A generic type is opted in for every instantiation, with the operators
/// each instantiation has.
#[derive(Debug, Clone, Copy, PartialEq, reassoc::Passthrough)]
struct Pair<T>(T, T);
impl<T: core::ops::Add<Output = T>> core::ops::Add for Pair<T> {
    type Output = Pair<T>;
    fn add(self, o: Pair<T>) -> Pair<T> {
        Pair(self.0 + o.0, self.1 + o.1)
    }
}

#[test]
fn derive_works_on_generic_types() {
    assert_eq!(alg!(Pair(1, 2) + Pair(3, 4)), Pair(4, 6));
    assert_eq!(alg!(Pair(1.5f32, 2.0) + Pair(3.0, 4.0)), Pair(4.5, 6.0));
}

/// A `where` clause with a trailing comma is what rustfmt writes for any
/// multi-line bound list; the derive must reproduce it without `, ,`.
#[derive(Debug, Clone, Copy, PartialEq, reassoc::Passthrough)]
struct Bounded<T>
where
    T: Copy,
    T: core::ops::Add<Output = T>,
{
    v: T,
}
impl<T> core::ops::Add for Bounded<T>
where
    T: Copy,
    T: core::ops::Add<Output = T>,
{
    type Output = Bounded<T>;
    fn add(self, o: Bounded<T>) -> Bounded<T> {
        Bounded { v: self.v + o.v }
    }
}

#[test]
fn derive_accepts_a_where_clause_with_a_trailing_comma() {
    assert_eq!(alg!(Bounded { v: 1 } + Bounded { v: 2 }), Bounded { v: 3 });
}

/// Derive on an enum and on a type with a lifetime parameter.
#[derive(Debug, Clone, Copy, PartialEq, reassoc::Passthrough)]
enum Sign {
    Pos,
    Neg,
}
impl core::ops::Mul for Sign {
    type Output = Sign;
    fn mul(self, o: Sign) -> Sign {
        if self == o { Sign::Pos } else { Sign::Neg }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, reassoc::Passthrough)]
struct Tagged<'a>(f32, &'a str);
impl core::ops::Add for Tagged<'_> {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Tagged(self.0 + o.0, self.1)
    }
}
impl core::ops::Mul for Tagged<'_> {
    type Output = Self;
    fn mul(self, o: Self) -> Self {
        Tagged(self.0 * o.0, self.1)
    }
}

#[test]
fn derive_on_an_enum_and_a_type_with_a_lifetime() {
    assert_eq!(alg!(Sign::Neg * Sign::Neg), Sign::Pos);
    let t = Tagged(2.0, "m");
    assert_eq!(alg!(t + t * t), Tagged(6.0, "m"));
}

// ---- references follow the type ----

/// Reference operands work exactly when the type implements them, as native
/// does: `Vec3` above has no `Add<&Vec3>`, so `&v + v` is rejected there
/// (`tests/ui/reference_operand_needs_impl.rs`); this type has the impls
/// and gets every combination.
#[derive(Debug, Clone, Copy, PartialEq, reassoc::Passthrough)]
struct RefOps(f64);
impl core::ops::Add for RefOps {
    type Output = RefOps;
    fn add(self, o: RefOps) -> RefOps {
        RefOps(self.0 + o.0)
    }
}
impl core::ops::Add<&RefOps> for RefOps {
    type Output = RefOps;
    fn add(self, o: &RefOps) -> RefOps {
        RefOps(self.0 + o.0)
    }
}
impl core::ops::Add<RefOps> for &RefOps {
    type Output = RefOps;
    fn add(self, o: RefOps) -> RefOps {
        RefOps(self.0 + o.0)
    }
}
impl core::ops::Add<&RefOps> for &RefOps {
    type Output = RefOps;
    fn add(self, o: &RefOps) -> RefOps {
        RefOps(self.0 + o.0)
    }
}
impl core::ops::AddAssign<&RefOps> for RefOps {
    fn add_assign(&mut self, o: &RefOps) {
        self.0 += o.0;
    }
}

#[test]
fn reference_operands_follow_the_types_own_impls() {
    let (a, b) = (RefOps(1.0), RefOps(2.0));
    assert_eq!(alg!(a + b), RefOps(3.0));
    assert_eq!(alg!(&a + b), RefOps(3.0));
    assert_eq!(alg!(a + &b), RefOps(3.0));
    assert_eq!(alg!(&a + &b), RefOps(3.0));
    let mut c = a;
    alg!(c += &b);
    assert_eq!(c, RefOps(3.0));
    // Iterator code, where the operands are references.
    let xs = [a, b];
    assert_eq!(
        alg!(xs.iter().fold(RefOps(0.0), |acc, x| acc + x)),
        RefOps(3.0)
    );
}

#[test]
fn std_wrappers_accept_reference_operands() {
    use core::num::{Saturating, Wrapping};
    let (a, b) = (Wrapping(250u8), Wrapping(10u8));
    assert_eq!(alg!(&a + b), Wrapping(4u8));
    assert_eq!(alg!(a + &b), Wrapping(4u8));
    assert_eq!(alg!(&a * &b), Wrapping(196u8));
    let (c, d) = (Saturating(250u8), Saturating(10u8));
    assert_eq!(alg!(&c + d), Saturating(255u8));
    assert_eq!(alg!(c - &d), Saturating(240u8));
    assert_eq!(alg!(&c - &d), Saturating(240u8));
}

// ---- outputs that are not the left type ----

/// A dot product: the output is not the left operand. Nothing is declared;
/// the type's own `Output` is used.
#[derive(Debug, Clone, Copy, PartialEq, reassoc::Passthrough)]
struct Ray([f64; 2]);
impl core::ops::Mul for Ray {
    type Output = f64;
    fn mul(self, o: Ray) -> f64 {
        self.0[0] * o.0[0] + self.0[1] * o.0[1]
    }
}

#[test]
fn an_output_that_is_not_the_left_operand() {
    let (u, v) = (Ray([1.0, 2.0]), Ray([3.0, 4.0]));
    assert_eq!(alg!(u * v), 11.0);
    let s: f64 = alg!(u * v + 1.0);
    assert_eq!(s, 12.0);
}

/// Two operators on one left type, two right types, one foreign output.
#[derive(Debug, Clone, Copy, PartialEq, reassoc::Passthrough)]
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

#[test]
fn two_right_hand_types_with_the_same_foreign_output() {
    let (q, r) = (Q(2.0), R(3.0));
    assert_eq!(alg!(q * q), 4.0);
    assert_eq!(alg!(q * r), 6.0);
}

// ---- non-Copy types and the in-place forms ----

/// A non-`Copy` type: `+` through its `Add`, `+=` through its `AddAssign`,
/// through a `&mut`, an index, or a bare local alike.
#[derive(Debug, Clone, PartialEq, reassoc::Passthrough)]
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

#[test]
fn non_copy_compound_assignment_in_place() {
    use reassoc::algebraic;
    #[algebraic]
    fn bump(v: &mut [Owned], mut local: Owned, suffix: Owned) -> Owned {
        v[0] += suffix.clone();
        local += suffix;
        local
    }
    let mut v = [Owned("a".into())];
    let l = bump(&mut v, Owned("x".into()), Owned("b".into()));
    assert_eq!((v[0].clone(), l), (Owned("ab".into()), Owned("xb".into())));
}

/// A right-hand type that is itself a reference: `Label + &str`, `Label +=
/// &str`, through the type's own impls.
#[derive(reassoc::Passthrough)]
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

#[test]
fn reference_right_operand() {
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

/// A type with `AddAssign` and nothing else: native `+=` on it is fine, and
/// so is the dispatched one; `+` is rejected, as natively.
#[derive(Debug, Clone, PartialEq, reassoc::Passthrough)]
struct InPlaceOnly(String);
impl core::ops::AddAssign for InPlaceOnly {
    fn add_assign(&mut self, o: InPlaceOnly) {
        self.0 += &o.0;
    }
}

#[test]
fn in_place_only_type() {
    use reassoc::algebraic;
    #[algebraic]
    fn go(v: &mut [InPlaceOnly], t: InPlaceOnly) {
        v[0] += t;
    }
    let mut v = [InPlaceOnly("a".into())];
    go(&mut v, InPlaceOnly("b".into()));
    assert_eq!(v[0], InPlaceOnly("ab".into()));
}

/// Five operators by value on a non-`Copy` type.
#[derive(Debug, Clone, PartialEq, reassoc::Passthrough)]
struct Big(i64);
macro_rules! big_ops {
    ($($t:ident, $m:ident, $op:tt);* $(;)?) => {$(
        impl core::ops::$t for Big {
            type Output = Big;
            fn $m(self, o: Big) -> Big { Big(self.0 $op o.0) }
        }
    )*};
}
big_ops!(Add, add, +; Sub, sub, -; Mul, mul, *; Div, div, /; Rem, rem, %);

#[test]
fn non_copy_type_by_value() {
    let (a, b) = (Big(7), Big(2));
    assert_eq!(alg!(a.clone() + b.clone()), Big(9));
    assert_eq!(alg!(a.clone() - b.clone()), Big(5));
    assert_eq!(alg!(a.clone() * b.clone()), Big(14));
    assert_eq!(alg!(a.clone() / b.clone()), Big(3));
    assert_eq!(alg!(a % b), Big(1));
}

// ---- scalars ----

#[derive(Debug, Clone, Copy, PartialEq, reassoc::Passthrough)]
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
        v * self
    }
}
impl core::ops::Div<f32> for V2 {
    type Output = V2;
    fn div(self, k: f32) -> V2 {
        V2(self.0 / k, self.1 / k)
    }
}
impl core::ops::MulAssign<f32> for V2 {
    fn mul_assign(&mut self, k: f32) {
        self.0 *= k;
        self.1 *= k;
    }
}
impl core::ops::DivAssign<f32> for V2 {
    fn div_assign(&mut self, k: f32) {
        self.0 /= k;
        self.1 /= k;
    }
}

/// An unsuffixed float literal on either side of a user vector (`v * 2.0`,
/// `2.0 * v`) infers to the scalar type the type's own impl names, on both
/// binary and compound forms; a float on the left goes through the type's
/// `Mul<V2> for f32` without being named.
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

// ---- operators on references only: the non-Copy numeric shape ----

/// `&Heavy + &Heavy`, `&Heavy * f64`, `Heavy * &Heavy`, and a dot product
/// `&Heavy * &Heavy => f64`: the standard shape for a non-`Copy` numeric
/// type, one line.
#[derive(Debug, Clone, PartialEq, reassoc::Passthrough)]
struct Heavy(Vec<f64>);
impl core::ops::Add<&Heavy> for &Heavy {
    type Output = Heavy;
    fn add(self, o: &Heavy) -> Heavy {
        Heavy(self.0.iter().zip(&o.0).map(|(a, b)| a + b).collect())
    }
}
impl core::ops::Sub<&Heavy> for &Heavy {
    type Output = Heavy;
    fn sub(self, o: &Heavy) -> Heavy {
        Heavy(self.0.iter().zip(&o.0).map(|(a, b)| a - b).collect())
    }
}
impl core::ops::Mul<f64> for &Heavy {
    type Output = Heavy;
    fn mul(self, k: f64) -> Heavy {
        Heavy(self.0.iter().map(|a| a * k).collect())
    }
}
impl core::ops::Mul<&Heavy> for Heavy {
    type Output = Heavy;
    fn mul(self, o: &Heavy) -> Heavy {
        Heavy(self.0.iter().zip(&o.0).map(|(a, b)| a * b).collect())
    }
}
impl core::ops::Mul<&Heavy> for &Heavy {
    type Output = f64; // a dot product: the output is not the left type
    fn mul(self, o: &Heavy) -> f64 {
        self.0.iter().zip(&o.0).map(|(a, b)| a * b).sum()
    }
}

#[test]
fn operators_on_references_only() {
    use reassoc::algebraic;
    #[algebraic]
    fn go(a: &Heavy, b: &Heavy, k: f64) -> (Heavy, Heavy, f64) {
        let s = &(a + b) * k * b;
        let d = a - b;
        let dot = a * b;
        (s, d, dot)
    }
    let (s, d, dot) = go(&Heavy(vec![1.0, 2.0]), &Heavy(vec![3.0, 4.0]), 2.0);
    assert_eq!(s, Heavy(vec![24.0, 48.0]));
    assert_eq!(d, Heavy(vec![-2.0, -2.0]));
    assert_eq!(dot, 11.0);
}

// An integer on the left of an opted-in type, the way a float already was:
// `k * v` with `impl Mul<IVec> for i32`. Found adopting glam, whose integer
// vectors have exactly this (`i8 / I8Vec2`): with only the float-left
// blanket the operator had no impl. A literal on the left (`2 * v`) was
// always fine: the literal rule leaves it native.
#[derive(Clone, Copy, Debug, PartialEq, reassoc::Passthrough)]
struct IVec(i32, i32);
impl core::ops::Mul<IVec> for i32 {
    type Output = IVec;
    fn mul(self, v: IVec) -> IVec {
        IVec(self * v.0, self * v.1)
    }
}
impl core::ops::Div<IVec> for i32 {
    type Output = IVec;
    fn div(self, v: IVec) -> IVec {
        IVec(self / v.0, self / v.1)
    }
}
impl core::ops::Add<IVec> for u8 {
    type Output = IVec;
    fn add(self, v: IVec) -> IVec {
        IVec(self as i32 + v.0, self as i32 + v.1)
    }
}
impl core::ops::Add for IVec {
    type Output = IVec;
    fn add(self, o: IVec) -> IVec {
        IVec(self.0 + o.0, self.1 + o.1)
    }
}

/// A primitive on the left of an opted-in type **in place**: `x *= v` with
/// `impl MulAssign<V> for f32`, which is native Rust and was not dispatched
/// (found adopting micromath, whose `f32 *= F32` is exactly this). The
/// binary form was already covered; this is its compound twin.
impl core::ops::MulAssign<IVec> for i32 {
    fn mul_assign(&mut self, v: IVec) {
        *self *= v.0;
    }
}
impl core::ops::AddAssign<IVec> for u8 {
    fn add_assign(&mut self, v: IVec) {
        *self += v.0 as u8;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, reassoc::Passthrough)]
struct Scale(f64);
impl core::ops::MulAssign<Scale> for f64 {
    fn mul_assign(&mut self, s: Scale) {
        *self *= s.0;
    }
}
impl core::ops::SubAssign<Scale> for f64 {
    fn sub_assign(&mut self, s: Scale) {
        *self -= s.0;
    }
}

#[test]
fn primitive_scalars_on_the_left_of_a_user_type_in_place() {
    use reassoc::algebraic;
    #[algebraic]
    fn go(mut x: f64, mut k: i32, mut b: u8, s: Scale, v: IVec) -> (f64, f64, i32, u8) {
        let mut y = x;
        x *= s;
        y -= s;
        k *= v;
        b += v;
        (x, y, k, b)
    }
    assert_eq!(go(3.0, 4, 1, Scale(2.0), IVec(3, 0)), (6.0, 1.0, 12, 4));
}

#[test]
fn integer_scalars_on_the_left_of_a_user_type() {
    use reassoc::algebraic;
    #[algebraic]
    fn go(k: i32, b: u8, v: IVec) -> (IVec, IVec, IVec, IVec) {
        (k * v, k / v + v, b + v, 2 * v)
    }
    assert_eq!(
        go(12, 1, IVec(3, 4)),
        (IVec(36, 48), IVec(7, 7), IVec(4, 5), IVec(6, 8))
    );
}
