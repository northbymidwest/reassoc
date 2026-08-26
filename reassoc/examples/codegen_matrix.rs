//! Fixture for the codegen matrix (`tests/codegen_matrix.rs`): every construct
//! the rewriter emits, written twice: `sugar_*` through the macros, `direct_*`
//! by hand with the `algebraic_*` methods (or the native operators, for the
//! types that are dispatched to them), plus `plain_*` IEEE negative controls.
//! The test compiles this at `-O3` to LLVM IR and requires each pair's
//! optimized IR to be identical after alpha-renaming: the dispatch layer,
//! the `match`/`ops::unit` around `+=`, `#[track_caller]`, the marker
//! blanket, the sealed-generic float and int impls, and the foreign tag must
//! all compile to nothing, and a chain of operators must stay one
//! reassociable DAG across every call layer. Hand-written twins use the same
//! parameter names so nothing but the arithmetic differs.
#![allow(
    clippy::many_single_char_names,
    clippy::too_many_arguments,
    clippy::missing_const_for_fn
)]

use core::num::{NonZero, Wrapping};
use core::time::Duration;
use reassoc::{alg, algebraic, passthrough, strict};

// ---- the five binary operators, f32 and f64 ----

macro_rules! binary {
    ($($t:ty: $sa:ident $da:ident + $aa:ident; $ss:ident $ds:ident - $as_:ident;
             $sm:ident $dm:ident * $am:ident; $sd:ident $dd:ident / $ad:ident; $sr:ident $dr:ident % $ar:ident;)*) => {$(
        #[algebraic] #[unsafe(no_mangle)] #[inline(never)] pub fn $sa(a: $t, b: $t) -> $t { a + b }
        #[unsafe(no_mangle)] #[inline(never)] pub fn $da(a: $t, b: $t) -> $t { a.$aa(b) }
        #[algebraic] #[unsafe(no_mangle)] #[inline(never)] pub fn $ss(a: $t, b: $t) -> $t { a - b }
        #[unsafe(no_mangle)] #[inline(never)] pub fn $ds(a: $t, b: $t) -> $t { a.$as_(b) }
        #[algebraic] #[unsafe(no_mangle)] #[inline(never)] pub fn $sm(a: $t, b: $t) -> $t { a * b }
        #[unsafe(no_mangle)] #[inline(never)] pub fn $dm(a: $t, b: $t) -> $t { a.$am(b) }
        #[algebraic] #[unsafe(no_mangle)] #[inline(never)] pub fn $sd(a: $t, b: $t) -> $t { a / b }
        #[unsafe(no_mangle)] #[inline(never)] pub fn $dd(a: $t, b: $t) -> $t { a.$ad(b) }
        #[algebraic] #[unsafe(no_mangle)] #[inline(never)] pub fn $sr(a: $t, b: $t) -> $t { a % b }
        #[unsafe(no_mangle)] #[inline(never)] pub fn $dr(a: $t, b: $t) -> $t { a.$ar(b) }
    )*};
}
binary! {
    f32: sugar_add_f32 direct_add_f32 + algebraic_add; sugar_sub_f32 direct_sub_f32 - algebraic_sub;
         sugar_mul_f32 direct_mul_f32 * algebraic_mul; sugar_div_f32 direct_div_f32 / algebraic_div;
         sugar_rem_f32 direct_rem_f32 % algebraic_rem;
    f64: sugar_add_f64 direct_add_f64 + algebraic_add; sugar_sub_f64 direct_sub_f64 - algebraic_sub;
         sugar_mul_f64 direct_mul_f64 * algebraic_mul; sugar_div_f64 direct_div_f64 / algebraic_div;
         sugar_rem_f64 direct_rem_f64 % algebraic_rem;
}

// ---- reference operands ----

#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_ref_lhs(a: &f32, b: f32) -> f32 {
    a * b
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_ref_lhs(a: &f32, b: f32) -> f32 {
    (*a).algebraic_mul(b)
}
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_ref_rhs(a: f32, b: &f32) -> f32 {
    a * b
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_ref_rhs(a: f32, b: &f32) -> f32 {
    a.algebraic_mul(*b)
}
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_ref_both(a: &f64, b: &f64) -> f64 {
    a + b
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_ref_both(a: &f64, b: &f64) -> f64 {
    (*a).algebraic_add(*b)
}

// ---- compound assignment through every place shape, and in tail position ----

#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_compound_bare(mut x: f32, y: f32) -> f32 {
    x += y;
    x *= y;
    x -= y;
    x /= y;
    x %= y;
    x
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_compound_bare(mut x: f32, y: f32) -> f32 {
    x = x.algebraic_add(y);
    x = x.algebraic_mul(y);
    x = x.algebraic_sub(y);
    x = x.algebraic_div(y);
    x = x.algebraic_rem(y);
    x
}
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_compound_index(v: &mut [f32], i: usize, k: f32) {
    v[i] *= k;
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_compound_index(v: &mut [f32], i: usize, k: f32) {
    v[i] = v[i].algebraic_mul(k);
}
pub struct S {
    pub x: f64,
    pub y: f64,
}
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_compound_field(s: &mut S, k: f64) {
    s.x -= k * s.y;
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_compound_field(s: &mut S, k: f64) {
    let r = k.algebraic_mul(s.y);
    s.x = s.x.algebraic_sub(r);
}
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_compound_deref(p: &mut f32, k: f32) {
    *p /= k;
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_compound_deref(p: &mut f32, k: f32) {
    *p = (*p).algebraic_div(k);
}
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_compound_tail(p: &mut f32, k: f32) {
    *p += k
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_compound_tail(p: &mut f32, k: f32) {
    *p = (*p).algebraic_add(k)
}

// ---- chains: one reassociable DAG across every call layer ----

#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_chain_sum16(a: &[f32; 16]) -> f32 {
    a[0] + a[1]
        + a[2]
        + a[3]
        + a[4]
        + a[5]
        + a[6]
        + a[7]
        + a[8]
        + a[9]
        + a[10]
        + a[11]
        + a[12]
        + a[13]
        + a[14]
        + a[15]
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_chain_sum16(a: &[f32; 16]) -> f32 {
    a[0].algebraic_add(a[1])
        .algebraic_add(a[2])
        .algebraic_add(a[3])
        .algebraic_add(a[4])
        .algebraic_add(a[5])
        .algebraic_add(a[6])
        .algebraic_add(a[7])
        .algebraic_add(a[8])
        .algebraic_add(a[9])
        .algebraic_add(a[10])
        .algebraic_add(a[11])
        .algebraic_add(a[12])
        .algebraic_add(a[13])
        .algebraic_add(a[14])
        .algebraic_add(a[15])
}
/// Negative control: the same chain, strict IEEE. Must NOT be what the
/// optimizer makes of the algebraic one.
#[unsafe(no_mangle)]
#[inline(never)]
pub fn plain_chain_sum16(a: &[f32; 16]) -> f32 {
    a[0] + a[1]
        + a[2]
        + a[3]
        + a[4]
        + a[5]
        + a[6]
        + a[7]
        + a[8]
        + a[9]
        + a[10]
        + a[11]
        + a[12]
        + a[13]
        + a[14]
        + a[15]
}
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_chain_dot8(a: &[f32; 8], b: &[f32; 8]) -> f32 {
    a[0] * b[0]
        + a[1] * b[1]
        + a[2] * b[2]
        + a[3] * b[3]
        + a[4] * b[4]
        + a[5] * b[5]
        + a[6] * b[6]
        + a[7] * b[7]
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_chain_dot8(a: &[f32; 8], b: &[f32; 8]) -> f32 {
    a[0].algebraic_mul(b[0])
        .algebraic_add(a[1].algebraic_mul(b[1]))
        .algebraic_add(a[2].algebraic_mul(b[2]))
        .algebraic_add(a[3].algebraic_mul(b[3]))
        .algebraic_add(a[4].algebraic_mul(b[4]))
        .algebraic_add(a[5].algebraic_mul(b[5]))
        .algebraic_add(a[6].algebraic_mul(b[6]))
        .algebraic_add(a[7].algebraic_mul(b[7]))
}
/// Eight compound steps: each is a `match` in `ops::unit(..)`; the chain
/// across them must still be one DAG.
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_chain_compound8(mut acc: f32, a: &[f32; 8], b: &[f32; 8]) -> f32 {
    acc += a[0] * b[0];
    acc += a[1] * b[1];
    acc += a[2] * b[2];
    acc += a[3] * b[3];
    acc += a[4] * b[4];
    acc += a[5] * b[5];
    acc += a[6] * b[6];
    acc += a[7] * b[7];
    acc
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_chain_compound8(mut acc: f32, a: &[f32; 8], b: &[f32; 8]) -> f32 {
    acc = acc.algebraic_add(a[0].algebraic_mul(b[0]));
    acc = acc.algebraic_add(a[1].algebraic_mul(b[1]));
    acc = acc.algebraic_add(a[2].algebraic_mul(b[2]));
    acc = acc.algebraic_add(a[3].algebraic_mul(b[3]));
    acc = acc.algebraic_add(a[4].algebraic_mul(b[4]));
    acc = acc.algebraic_add(a[5].algebraic_mul(b[5]));
    acc = acc.algebraic_add(a[6].algebraic_mul(b[6]));
    acc = acc.algebraic_add(a[7].algebraic_mul(b[7]));
    acc
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn plain_chain_compound8(mut acc: f32, a: &[f32; 8], b: &[f32; 8]) -> f32 {
    acc += a[0] * b[0];
    acc += a[1] * b[1];
    acc += a[2] * b[2];
    acc += a[3] * b[3];
    acc += a[4] * b[4];
    acc += a[5] * b[5];
    acc += a[6] * b[6];
    acc += a[7] * b[7];
    acc
}
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_horner8(x: f64, c: &[f64; 9]) -> f64 {
    (((((((c[8] * x + c[7]) * x + c[6]) * x + c[5]) * x + c[4]) * x + c[3]) * x + c[2]) * x + c[1])
        * x
        + c[0]
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_horner8(x: f64, c: &[f64; 9]) -> f64 {
    c[8].algebraic_mul(x)
        .algebraic_add(c[7])
        .algebraic_mul(x)
        .algebraic_add(c[6])
        .algebraic_mul(x)
        .algebraic_add(c[5])
        .algebraic_mul(x)
        .algebraic_add(c[4])
        .algebraic_mul(x)
        .algebraic_add(c[3])
        .algebraic_mul(x)
        .algebraic_add(c[2])
        .algebraic_mul(x)
        .algebraic_add(c[1])
        .algebraic_mul(x)
        .algebraic_add(c[0])
}
/// The loop forms: the dot product that is the crate's headline, `f32` and
/// `f64`, and `axpy` (an index place in a loop). The `plain_` twin of the
/// `f32` dot is the negative control for vectorization: strict IEEE addition
/// may not be reassociated, so its reduction must stay a serial chain while
/// the algebraic one is free to become a vector reduction.
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_dot_loop_f32(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0;
    for i in 0..a.len().min(b.len()) {
        sum += a[i] * b[i];
    }
    sum
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_dot_loop_f32(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    for i in 0..a.len().min(b.len()) {
        sum = sum.algebraic_add(a[i].algebraic_mul(b[i]));
    }
    sum
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn plain_dot_loop_f32(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    for i in 0..a.len().min(b.len()) {
        sum += a[i] * b[i];
    }
    sum
}
// ---- generic code over a user float trait (`#[algebraic_float]`) ----
//
// The dispatch path is the same sealed-generic float impl the concrete
// `f32` pairs above go through; what differs is that the body is written
// against a type parameter and monomorphised. The twin is the hand-written
// `f32` dot loop, so this pins that the generic wrapper adds nothing.

#[reassoc::algebraic_float]
pub trait UserFloat: Copy {
    fn zero() -> Self;
}
impl UserFloat for f32 {
    #[inline(always)]
    fn zero() -> f32 {
        0.0
    }
}
#[algebraic]
#[inline(always)]
fn generic_dot<T: UserFloat>(a: &[T], b: &[T]) -> T {
    let mut sum = T::zero();
    for i in 0..a.len().min(b.len()) {
        sum += a[i] * b[i];
    }
    sum
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_generic_dot_f32(a: &[f32], b: &[f32]) -> f32 {
    generic_dot(a, b)
}
// The twin goes through the same always-inlined helper, so nothing but the
// arithmetic differs.
#[inline(always)]
fn direct_dot_inner(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    for i in 0..a.len().min(b.len()) {
        sum = sum.algebraic_add(a[i].algebraic_mul(b[i]));
    }
    sum
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_generic_dot_f32(a: &[f32], b: &[f32]) -> f32 {
    direct_dot_inner(a, b)
}
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_axpy_loop(a: f32, x: &[f32], y: &mut [f32]) {
    for i in 0..x.len().min(y.len()) {
        y[i] += a * x[i];
    }
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_axpy_loop(a: f32, x: &[f32], y: &mut [f32]) {
    for i in 0..x.len().min(y.len()) {
        let r = a.algebraic_mul(x[i]);
        y[i] = y[i].algebraic_add(r);
    }
}
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_dot_loop_f64(a: &[f64], b: &[f64]) -> f64 {
    let mut sum = 0.0;
    for i in 0..a.len().min(b.len()) {
        sum += a[i] * b[i];
    }
    sum
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_dot_loop_f64(a: &[f64], b: &[f64]) -> f64 {
    let mut sum = 0.0f64;
    for i in 0..a.len().min(b.len()) {
        sum = sum.algebraic_add(a[i].algebraic_mul(b[i]));
    }
    sum
}

// ---- strict!, unary minus, literals, closures, alg! ----

/// `strict!` in the middle of an algebraic expression is a barrier exactly
/// there: the inner `b + c` is IEEE, the outer `+` algebraic.
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_strict_mid(a: f32, b: f32, c: f32) -> f32 {
    a + strict!(b + c)
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_strict_mid(a: f32, b: f32, c: f32) -> f32 {
    a.algebraic_add(b + c)
}
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_neg(a: f32, b: f32) -> f32 {
    -(a * b) + 1.5
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_neg(a: f32, b: f32) -> f32 {
    (-a.algebraic_mul(b)).algebraic_add(1.5)
}
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_literals(x: f32) -> f32 {
    let k = 2.0;
    -(3.0 * 0.5) * x + k * x
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_literals(x: f32) -> f32 {
    let k = 2.0f32;
    (-(3.0f32.algebraic_mul(0.5)))
        .algebraic_mul(x)
        .algebraic_add(k.algebraic_mul(x))
}
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_closure(xs: &[f32], k: f32) -> f32 {
    xs.iter().map(|x| x * k + 1.0).fold(0.0, |a, b| a + b)
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_closure(xs: &[f32], k: f32) -> f32 {
    xs.iter()
        .map(|x| x.algebraic_mul(k).algebraic_add(1.0))
        .fold(0.0, |a, b| a.algebraic_add(b))
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_alg_forms(a: f32, b: f32, c: f32) -> f32 {
    let mut s = alg!(a * b + c);
    alg! { s += a; s *= c; }
    alg!(s += b);
    s
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_alg_forms(a: f32, b: f32, c: f32) -> f32 {
    let mut s = a.algebraic_mul(b).algebraic_add(c);
    s = s.algebraic_add(a);
    s = s.algebraic_mul(c);
    s = s.algebraic_add(b);
    s
}

// ---- integers: the sealed generic impls must be the plain operators ----

#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_ints(a: u32, b: u32, c: i64, d: i64, mut e: usize, f: usize) -> (u32, i64, usize, u8) {
    e += f;
    e *= f;
    (a + b * a, c % d - c / d, e, (a as u8) - (b as u8))
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_ints(
    a: u32,
    b: u32,
    c: i64,
    d: i64,
    mut e: usize,
    f: usize,
) -> (u32, i64, usize, u8) {
    e += f;
    e *= f;
    (a + b * a, c % d - c / d, e, (a as u8) - (b as u8))
}

// ---- user types through the marker blanket (native operators) ----

// An integer on the left of an opted-in type: the per-integer blanket.
impl core::ops::Mul<V> for u32 {
    type Output = V;
    #[inline]
    fn mul(self, v: V) -> V {
        V(v.0 * self as f32, v.1 * self as f32)
    }
}
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_int_left(n: u32, v: V, w: V) -> V {
    n * v + n * w
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_int_left(n: u32, v: V, w: V) -> V {
    n * v + n * w
}

#[derive(Clone, Copy, Debug, PartialEq, reassoc::Passthrough)]
pub struct V(pub f32, pub f32);
impl core::ops::Add for V {
    type Output = V;
    #[inline]
    fn add(self, o: V) -> V {
        V(self.0 + o.0, self.1 + o.1)
    }
}
impl core::ops::Mul<f32> for V {
    type Output = V;
    #[inline]
    fn mul(self, k: f32) -> V {
        V(self.0 * k, self.1 * k)
    }
}
impl core::ops::Mul<V> for f32 {
    type Output = V;
    #[inline]
    fn mul(self, v: V) -> V {
        v * self
    }
}
impl core::ops::AddAssign for V {
    #[inline]
    fn add_assign(&mut self, o: V) {
        self.0 += o.0;
        self.1 += o.1;
    }
}
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_user_type(v: V, w: V, k: f32) -> V {
    let mut a = v + w * k + 2.0 * v;
    a += w;
    a
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_user_type(v: V, w: V, k: f32) -> V {
    let mut a = v + w * k + 2.0 * v;
    a += w;
    a
}

#[derive(Clone, Copy, Debug, PartialEq, reassoc::Passthrough)]
pub struct Pair<T>(pub T, pub T);
impl<T: core::ops::Mul<Output = T>> core::ops::Mul for Pair<T> {
    type Output = Pair<T>;
    #[inline]
    fn mul(self, o: Pair<T>) -> Pair<T> {
        Pair(self.0 * o.0, self.1 * o.1)
    }
}
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_derive_generic(p: Pair<f64>, q: Pair<f64>) -> Pair<f64> {
    p * q
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_derive_generic(p: Pair<f64>, q: Pair<f64>) -> Pair<f64> {
    p * q
}

/// A non-`Copy` type with its operators on references and a heterogeneous
/// output: the heavy-numeric shape. `&Heavy + &Heavy`, `&Heavy * f64`,
/// `&Heavy * &Heavy => f64`, `Heavy += Heavy` (a move in, as natively).
#[derive(Clone, Debug, PartialEq, reassoc::Passthrough)]
pub struct Heavy(pub Vec<f64>);
impl core::ops::Add<&Heavy> for &Heavy {
    type Output = Heavy;
    #[inline]
    fn add(self, o: &Heavy) -> Heavy {
        Heavy(self.0.iter().zip(&o.0).map(|(a, b)| a + b).collect())
    }
}
impl core::ops::Mul<f64> for &Heavy {
    type Output = Heavy;
    #[inline]
    fn mul(self, k: f64) -> Heavy {
        Heavy(self.0.iter().map(|a| a * k).collect())
    }
}
impl core::ops::Mul<&Heavy> for &Heavy {
    type Output = f64;
    #[inline]
    fn mul(self, o: &Heavy) -> f64 {
        self.0.iter().zip(&o.0).map(|(a, b)| a * b).sum()
    }
}
impl core::ops::AddAssign for Heavy {
    #[inline]
    fn add_assign(&mut self, o: Heavy) {
        for (a, b) in self.0.iter_mut().zip(o.0) {
            *a += b;
        }
    }
}
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_non_copy(a: &Heavy, b: &Heavy, k: f64) -> (Heavy, f64) {
    let mut s = a + b;
    s += &s * k;
    let dot = a * b;
    (s, dot)
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_non_copy(a: &Heavy, b: &Heavy, k: f64) -> (Heavy, f64) {
    let mut s = a + b;
    let r = &s * k;
    s += r;
    let dot = a * b;
    (s, dot)
}

// ---- a type from another crate, through the foreign tag ----

use foreign_types::Vec3;
passthrough!(foreign Vec3);
passthrough!(foreign mul: f32, Vec3 => Vec3);
#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_foreign(p: Vec3, v: Vec3, dt: f32) -> Vec3 {
    let mut q = p + v * dt + 0.5 * v;
    q += v;
    q
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_foreign(p: Vec3, v: Vec3, dt: f32) -> Vec3 {
    let mut q = p + v * dt + 0.5 * v;
    q += v;
    q
}

// ---- std types ----

#[algebraic]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn sugar_std(
    w: Wrapping<u32>,
    d: Duration,
    n: u32,
    z: NonZero<u32>,
) -> (Wrapping<u32>, Duration, u32) {
    let mut x = w + w;
    x *= w;
    (x, d * n + d, n / z + n % z)
}
#[unsafe(no_mangle)]
#[inline(never)]
pub fn direct_std(
    w: Wrapping<u32>,
    d: Duration,
    n: u32,
    z: NonZero<u32>,
) -> (Wrapping<u32>, Duration, u32) {
    let mut x = w + w;
    x *= w;
    (x, d * n + d, n / z + n % z)
}

fn main() {
    // Keep the example runnable; the test reads the IR, not the output.
    let a = [1.0f32; 16];
    println!(
        "{} {} {}",
        sugar_chain_sum16(&a),
        direct_chain_sum16(&a),
        plain_chain_sum16(&a)
    );
}
