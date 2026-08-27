//! Compound assignment, exhaustively. Two expansions exist (a bare path is
//! assigned through by name, anything else goes through
//! `ops::add_assign(&mut place, rhs)`), and both must agree with native `+=`
//! on evaluation order, evaluation count, drop timing, aliasing, and value.
//!
//! Every case that compiles natively is written twice, native and rewritten,
//! and the two are compared on a log or a value. Cases that exist only to
//! prove the rewrite *happened* use `Dispatched`, which has no `std::ops`.
#![allow(clippy::all)]

use core::cell::{Cell, RefCell};
use reassoc::{alg, algebraic};

// ---------------------------------------------------------------------------
// Evaluation order and count, on the `&mut place` path
// ---------------------------------------------------------------------------

/// Records every observable step of `v[idx()] += guard().value()`, where the
/// guard is a temporary of the RHS *expression*, not a local inside a
/// closure, so its drop marks the end of the statement's temporaries. A
/// `let`-based expansion would drop it before the place is evaluated.
fn trace(rewritten: bool) -> (Vec<&'static str>, f64) {
    let log: RefCell<Vec<&'static str>> = RefCell::new(Vec::new());
    struct Guard<'a>(&'a RefCell<Vec<&'static str>>);
    impl Guard<'_> {
        fn value(&self) -> f64 {
            10.0
        }
    }
    impl Drop for Guard<'_> {
        fn drop(&mut self) {
            self.0.borrow_mut().push("rhs temporary dropped");
        }
    }
    let mut v = [1.0f64, 2.0];
    let idx = || {
        log.borrow_mut().push("place evaluated");
        1usize
    };
    let guard = || {
        log.borrow_mut().push("rhs evaluated");
        Guard(&log)
    };
    if rewritten {
        alg!(v[idx()] += guard().value());
    } else {
        v[idx()] += guard().value();
    }
    log.borrow_mut().push("statement ended");
    (log.into_inner(), v[1])
}

#[test]
fn index_place_matches_native_order_count_and_drop_timing() {
    assert_eq!(trace(true), trace(false));
    // RHS first, then the place, and the RHS's temporary lives until the
    // statement ends, after the place is evaluated and written.
    assert_eq!(
        trace(false).0,
        [
            "rhs evaluated",
            "place evaluated",
            "rhs temporary dropped",
            "statement ended"
        ]
    );
}

#[test]
fn place_and_rhs_are_each_evaluated_exactly_once() {
    let (places, rhss) = (Cell::new(0), Cell::new(0));
    let mut v = [0.0f64; 4];
    let idx = || {
        places.set(places.get() + 1);
        2
    };
    let rhs = || {
        rhss.set(rhss.get() + 1);
        1.5
    };
    #[algebraic]
    fn go(v: &mut [f64], idx: impl Fn() -> usize, rhs: impl Fn() -> f64) {
        v[idx()] += rhs();
        v[idx()] *= rhs();
        v[idx()] -= rhs();
    }
    go(&mut v, idx, rhs);
    assert_eq!((places.get(), rhss.get()), (3, 3));
    assert_eq!(v[2], (0.0 + 1.5) * 1.5 - 1.5);
}

// ---------------------------------------------------------------------------
// The RHS reads or writes the place
// ---------------------------------------------------------------------------

#[algebraic]
fn rhs_reads_place(v: &mut [f64], s: &mut f64, k: f64) {
    v[0] += v[1]; // another element
    v[1] += v[1]; // itself
    v[0] *= v[0] + v[1]; // itself, inside an expression
    *s += *s * k; // through a &mut param
    let i = 1;
    v[i] -= v[i - 1];
}
fn rhs_reads_place_native(v: &mut [f64], s: &mut f64, k: f64) {
    v[0] += v[1];
    v[1] += v[1];
    v[0] *= v[0] + v[1];
    *s += *s * k;
    let i = 1;
    v[i] -= v[i - 1];
}

#[test]
fn rhs_that_reads_the_place_borrow_checks_and_agrees_with_native() {
    let (mut a, mut b) = ([1.0, 2.0], [1.0, 2.0]);
    let (mut sa, mut sb) = (3.0, 3.0);
    rhs_reads_place(&mut a, &mut sa, 0.5);
    rhs_reads_place_native(&mut b, &mut sb, 0.5);
    assert_eq!((a, sa), (b, sb));
}

#[test]
fn rhs_that_writes_the_place_runs_first_like_native() {
    // Native primitive `+=` evaluates the RHS before reading the place, so a
    // RHS that overwrites the place is seen by the addition. Both expansions
    // must agree: the bare-path one (`a`) and the index one (`v[0]`).
    #[allow(unused_assignments)]
    #[algebraic]
    fn go() -> (f64, f64) {
        let mut a = 1.0;
        a += {
            a = 5.0;
            1.0
        };
        let mut v = [1.0];
        v[0] += {
            v[0] = 5.0;
            1.0
        };
        (a, v[0])
    }
    #[allow(unused_assignments)]
    fn native() -> (f64, f64) {
        let mut a = 1.0;
        a += {
            a = 5.0;
            1.0
        };
        let mut v = [1.0];
        v[0] += {
            v[0] = 5.0;
            1.0
        };
        (a, v[0])
    }
    assert_eq!(go(), native());
    assert_eq!(go(), (6.0, 6.0));
}

// ---------------------------------------------------------------------------
// Place shapes: every way to reach a place that is not a bare path
// ---------------------------------------------------------------------------

struct Inner {
    x: f64,
}
struct Outer {
    inner: Inner,
    boxed: Box<f64>,
    cell: RefCell<f64>,
    v: Vec<f64>,
}

impl Outer {
    #[algebraic]
    fn bump(&mut self, d: f64) {
        self.inner.x += d; // field of field, behind &mut self
        *self.boxed *= d; // deref of a Box field
        *self.cell.borrow_mut() -= d; // deref of a temporary RefMut
        self.v[1] /= d; // index into a Vec field
        let i = self.v.len() - 1;
        self.v[i] += self.v[0]; // index computed from the same struct
    }
    fn bump_native(&mut self, d: f64) {
        self.inner.x += d;
        *self.boxed *= d;
        *self.cell.borrow_mut() -= d;
        self.v[1] /= d;
        let i = self.v.len() - 1;
        self.v[i] += self.v[0];
    }
    fn snapshot(&self) -> (f64, f64, f64, Vec<f64>) {
        (
            self.inner.x,
            *self.boxed,
            *self.cell.borrow(),
            self.v.clone(),
        )
    }
}

fn outer() -> Outer {
    Outer {
        inner: Inner { x: 1.0 },
        boxed: Box::new(2.0),
        cell: RefCell::new(3.0),
        v: vec![4.0, 8.0, 16.0],
    }
}

#[test]
fn every_place_shape_agrees_with_native() {
    let (mut a, mut b) = (outer(), outer());
    a.bump(2.0);
    b.bump_native(2.0);
    assert_eq!(a.snapshot(), b.snapshot());
}

#[test]
fn place_through_a_closure_capture_and_a_nested_index() {
    #[algebraic]
    fn go() -> ([f64; 3], [usize; 2]) {
        let mut v = [1.0, 2.0, 3.0];
        let idx = [2usize, 0];
        let mut add_to = |i: usize, d: f64| v[i] += d; // captures v by &mut
        add_to(0, 1.0);
        add_to(2, 0.5);
        v[idx[1]] += v[idx[0]]; // index through another index
        (v, idx)
    }
    assert_eq!(go(), ([5.5, 2.0, 3.5], [2, 0]));
}

#[cfg(feature = "std")]
#[test]
fn place_behind_a_mutex_guard() {
    use std::sync::Mutex;
    let m = Mutex::new(1.0f64);
    #[algebraic]
    fn go(m: &Mutex<f64>, d: f64) {
        *m.lock().unwrap() += d * 2.0;
    }
    go(&m, 1.0);
    assert_eq!(*m.lock().unwrap(), 3.0);
}

// ---------------------------------------------------------------------------
// Every built-in place type through an index, with a non-literal RHS
// ---------------------------------------------------------------------------

#[algebraic]
fn builtins(f: &mut [f32], d: &mut [f64], i: &mut [i32], u: &mut [u64], k: f32, n: i32, m: u64) {
    f[0] += k;
    f[1] -= k;
    f[0] *= f[1];
    f[1] /= k;
    f[0] %= k;
    d[0] += d[1];
    d[1] *= d[0];
    i[0] += n;
    i[1] -= n;
    i[0] *= i[1];
    i[1] /= n;
    i[0] %= n;
    u[0] += m;
    u[1] *= m;
}
fn builtins_native(
    f: &mut [f32],
    d: &mut [f64],
    i: &mut [i32],
    u: &mut [u64],
    k: f32,
    n: i32,
    m: u64,
) {
    f[0] += k;
    f[1] -= k;
    f[0] *= f[1];
    f[1] /= k;
    f[0] %= k;
    d[0] += d[1];
    d[1] *= d[0];
    i[0] += n;
    i[1] -= n;
    i[0] *= i[1];
    i[1] /= n;
    i[0] %= n;
    u[0] += m;
    u[1] *= m;
}

#[test]
fn builtin_types_through_an_index_agree_with_native() {
    // Exactly representable values, so float results are bit-identical.
    let (mut f1, mut f2) = ([8.0f32, 4.0], [8.0f32, 4.0]);
    let (mut d1, mut d2) = ([8.0, 4.0], [8.0, 4.0]);
    let (mut i1, mut i2) = ([8, 4], [8, 4]);
    let (mut u1, mut u2) = ([8u64, 4], [8u64, 4]);
    builtins(&mut f1, &mut d1, &mut i1, &mut u1, 2.0, 3, 5);
    builtins_native(&mut f2, &mut d2, &mut i2, &mut u2, 2.0, 3, 5);
    assert_eq!((f1, d1, i1, u1), (f2, d2, i2, u2));
}

#[test]
fn wrapping_saturating_and_duration_through_an_index() {
    use core::num::{Saturating, Wrapping};
    use core::time::Duration;
    #[algebraic]
    fn go(w: &mut [Wrapping<u8>], s: &mut [Saturating<u8>], d: &mut [Duration], n: u32) {
        w[0] += w[1];
        let one = w[1];
        w[0] += &one; // a reference RHS; `&w[1]` itself would be E0502 natively too
        s[0] += s[1];
        d[0] += d[1];
        d[1] *= n;
        let second = d[1];
        d[0] -= second; // `Duration: SubAssign<Duration>` only, no `&` form natively either
    }
    let mut w = [Wrapping(250u8), Wrapping(5)];
    let mut s = [Saturating(250u8), Saturating(10)];
    let mut d = [Duration::from_secs(10), Duration::from_secs(1)];
    go(&mut w, &mut s, &mut d, 3);
    assert_eq!(w[0], Wrapping(4));
    assert_eq!(s[0], Saturating(255));
    assert_eq!(d, [Duration::from_secs(8), Duration::from_secs(3)]);
}

#[cfg(feature = "std")]
#[test]
fn instant_through_an_index() {
    use core::time::Duration;
    use std::time::Instant;
    #[algebraic]
    fn go(t: &mut [Instant], d: Duration) {
        t[0] += d;
        t[0] -= d;
    }
    let now = Instant::now();
    let mut t = [now];
    go(&mut t, Duration::from_millis(5));
    assert_eq!(t[0], now);
}

#[test]
fn reference_rhs_on_the_index_path() {
    #[algebraic]
    fn go(v: &mut [f64], r: &f64) {
        v[0] += r;
        let first = v[0];
        v[1] *= &first;
    }
    let mut v = [1.0, 2.0];
    go(&mut v, &3.0);
    assert_eq!(v, [4.0, 8.0]);
}

#[test]
fn integer_literal_rhs_through_an_index_stays_native() {
    // A literal on the right leaves the operation native (it cannot be float
    // arithmetic), so this is plain `AddAssign` on usize, including through
    // an index, with no dispatch involved.
    #[algebraic]
    fn count(hist: &mut [usize], keys: &[usize]) {
        for &k in keys {
            hist[k] += 1;
        }
    }
    let mut h = [0usize; 3];
    count(&mut h, &[0, 2, 2]);
    assert_eq!(h, [1, 0, 2]);
}

// ---------------------------------------------------------------------------
// Strings: in place, through every shape, value and reference RHS
// ---------------------------------------------------------------------------

#[cfg(feature = "alloc")]
#[test]
fn string_in_place_through_every_shape() {
    struct Named {
        name: String,
        tags: Vec<String>,
    }
    impl Named {
        #[algebraic]
        fn go(&mut self, s: &str, t: &String, u: String) {
            self.name += s;
            self.name += t;
            self.tags[0] += s;
            self.tags[1] += t;
            self.tags[0] += u.as_str();
        }
        fn go_native(&mut self, s: &str, t: &String, u: String) {
            self.name += s;
            self.name += t;
            self.tags[0] += s;
            self.tags[1] += t;
            self.tags[0] += u.as_str();
        }
    }
    let fresh = || Named {
        name: "n".into(),
        tags: vec!["a".into(), "b".into()],
    };
    let (mut x, mut y) = (fresh(), fresh());
    x.go("!", &"?".to_string(), "#".into());
    y.go_native("!", &"?".to_string(), "#".into());
    assert_eq!((x.name, x.tags), (y.name, y.tags));
}

/// Documented divergence, pinned so it cannot change silently: native
/// overloaded `+=` evaluates the place before the RHS (a two-phase borrow), so
/// a RHS that mutates the place is rejected. The rewrite evaluates the RHS
/// first for every type, so the same code compiles and sees the mutation.
#[cfg(feature = "alloc")]
#[test]
fn in_place_rhs_that_mutates_the_place_is_accepted_rhs_first() {
    #[algebraic]
    fn go(v: &mut Vec<String>) {
        v[0] += &{
            v[0].push('x');
            String::from("y")
        };
    }
    let mut v = vec![String::from("a")];
    go(&mut v);
    assert_eq!(v[0], "axy");
}

// ---------------------------------------------------------------------------
// User types: the rewrite is observable, generics, derive
// ---------------------------------------------------------------------------

/// Implements the dispatch traits and no `std::ops`, so `v[0] += d` can only
/// compile by going through `ops::add_assign` and the synthesised impl.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Dispatched(f32);
impl reassoc::__private::traits::AddRhs<Dispatched, Dispatched> for Dispatched {
    fn add_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 + self.0)
    }
}
impl reassoc::__private::traits::AddAssignRhs<Dispatched> for Dispatched {
    fn add_assign_rhs(self, lhs: &mut Dispatched) {
        lhs.0 += self.0
    }
}

#[test]
fn index_place_compound_assignment_is_dispatched_not_native() {
    #[algebraic]
    fn go(v: &mut [Dispatched], d: Dispatched) {
        v[0] += d;
    }
    let mut v = [Dispatched(1.0)];
    go(&mut v, Dispatched(2.0));
    assert_eq!(v[0], Dispatched(3.0));
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[reassoc::passthrough]
struct Pair<T>(T, T);
impl<T: core::ops::Add<Output = T>> core::ops::Add for Pair<T> {
    type Output = Pair<T>;
    fn add(self, o: Pair<T>) -> Pair<T> {
        Pair(self.0 + o.0, self.1 + o.1)
    }
}
impl<T: core::ops::Mul<Output = T>> core::ops::Mul for Pair<T> {
    type Output = Pair<T>;
    fn mul(self, o: Pair<T>) -> Pair<T> {
        Pair(self.0 * o.0, self.1 * o.1)
    }
}
impl<T: core::ops::AddAssign> core::ops::AddAssign for Pair<T> {
    fn add_assign(&mut self, o: Pair<T>) {
        self.0 += o.0;
        self.1 += o.1;
    }
}
impl<T: core::ops::AddAssign + Copy> core::ops::AddAssign<&Pair<T>> for Pair<T> {
    fn add_assign(&mut self, o: &Pair<T>) {
        self.0 += o.0;
        self.1 += o.1;
    }
}
impl<T: core::ops::MulAssign> core::ops::MulAssign for Pair<T> {
    fn mul_assign(&mut self, o: Pair<T>) {
        self.0 *= o.0;
        self.1 *= o.1;
    }
}

#[test]
fn generic_copy_derive_through_an_index() {
    #[algebraic]
    fn go(v: &mut [Pair<f64>], p: Pair<f64>) {
        v[0] += p;
        v[1] *= v[0];
        let first = v[0];
        v[1] += &first;
    }
    let mut v = [Pair(1.0, 2.0), Pair(2.0, 2.0)];
    go(&mut v, Pair(1.0, 1.0));
    assert_eq!(v, [Pair(2.0, 3.0), Pair(6.0, 9.0)]);
}

#[derive(Debug, Clone, PartialEq)]
#[reassoc::passthrough]
struct Tag(String);
impl core::ops::Add for Tag {
    type Output = Tag;
    fn add(self, o: Tag) -> Tag {
        Tag(self.0 + &o.0)
    }
}
impl core::ops::AddAssign for Tag {
    fn add_assign(&mut self, o: Tag) {
        self.0 += &o.0;
    }
}

#[test]
fn non_copy_derive_in_place_through_field_and_index() {
    struct Holder {
        tag: Tag,
        tags: Vec<Tag>,
    }
    impl Holder {
        #[algebraic]
        fn go(&mut self, t: Tag) {
            self.tag += t.clone();
            self.tags[0] += t;
        }
    }
    let mut h = Holder {
        tag: Tag("a".into()),
        tags: vec![Tag("b".into())],
    };
    h.go(Tag("!".into()));
    assert_eq!((h.tag, h.tags), (Tag("a!".into()), vec![Tag("b!".into())]));
}

// ---------------------------------------------------------------------------
// Bare paths: locals, statics, non-Copy locals
// ---------------------------------------------------------------------------

static mut COUNTER: u64 = 0;

#[test]
fn bare_paths_including_a_static_mut() {
    #[algebraic]
    fn go(k: u64, t: Tag) -> (u64, Tag, f64) {
        let ticks = unsafe {
            COUNTER += k; // static mut: `&mut` on it is allowed by the expansion
            COUNTER
        };
        let mut s = t; // non-Copy local: in place through its `AddAssign`
        s += Tag("?".into());
        let (mut a, b) = (1.0, 2.0); // pattern-bound local
        a += b;
        (ticks, s, a)
    }
    let (ticks, s, a) = go(2, Tag("x".into()));
    assert_eq!((ticks, s, a), (2, Tag("x?".into()), 3.0));
}

// ---------------------------------------------------------------------------
// Every place goes through `&mut`, bare paths included. A bare path used to
// be assigned through by name (`s = add(s, rhs)`), which moved a non-`Copy`
// local out of a closure or `async` block and needed `+` where native needs
// `+=`. These pin the `&mut` route.
// ---------------------------------------------------------------------------

#[cfg(feature = "alloc")]
#[test]
fn non_copy_local_captured_by_a_closure_stays_fn_mut() {
    #[algebraic]
    fn go(parts: &[&str]) -> String {
        let mut s = String::new();
        let mut push = |p: &str| s += p; // FnMut natively; must not become FnOnce
        for p in parts {
            push(p);
        }
        s
    }
    assert_eq!(go(&["a", "b", "c"]), "abc");
}

#[cfg(feature = "alloc")]
#[test]
fn non_copy_local_captured_by_an_async_block_is_borrowed_not_moved() {
    use core::future::Future;
    use core::pin::pin;
    use core::task::{Context, Poll, Waker};
    #[algebraic]
    async fn go(t: &str) -> String {
        let mut s = String::new();
        let fut = async {
            s += t;
        };
        fut.await;
        s // still usable: the block borrowed `s`, it did not move it
    }
    let mut fut = pin!(go("x"));
    let Poll::Ready(got) = fut.as_mut().poll(&mut Context::from_waker(Waker::noop())) else {
        panic!("the future has no await points that pend");
    };
    assert_eq!(got, "x");
}

#[test]
fn in_place_only_type_on_a_bare_path() {
    // `AddAssign` without `Add`: native `a += b` works, and it must here too.
    #[derive(Debug, PartialEq)]
    #[reassoc::passthrough]
    struct Acc(f64);
    impl core::ops::AddAssign<&Acc> for Acc {
        fn add_assign(&mut self, o: &Acc) {
            self.0 += o.0;
        }
    }
    #[algebraic]
    fn go(mut a: Acc, b: &Acc) -> Acc {
        a += b;
        a
    }
    assert_eq!(go(Acc(1.0), &Acc(2.0)), Acc(3.0));
}

#[test]
fn struct_literal_rhs_on_both_paths() {
    // The RHS is bound through a `match`; a bare struct literal is not allowed
    // as a scrutinee, so the expansion must not put it there unwrapped.
    #[derive(Clone, Copy, Debug, PartialEq)]
    #[reassoc::passthrough]
    struct P {
        x: f64,
    }
    impl core::ops::Add for P {
        type Output = P;
        fn add(self, o: P) -> P {
            P { x: self.x + o.x }
        }
    }
    impl core::ops::AddAssign for P {
        fn add_assign(&mut self, o: P) {
            self.x += o.x;
        }
    }
    impl core::ops::Neg for P {
        type Output = P;
        fn neg(self) -> P {
            P { x: -self.x }
        }
    }
    #[allow(unused_parens)]
    #[algebraic]
    fn go(mut acc: P, v: &mut [P]) -> P {
        acc += P { x: 1.0 };
        acc += (P { x: 2.0 });
        acc += -P { x: 4.0 };
        acc += P { x: 8.0 }.x.into_p();
        v[0] += P { x: 16.0 };
        acc + v[0]
    }
    trait IntoP {
        fn into_p(self) -> P;
    }
    impl IntoP for f64 {
        fn into_p(self) -> P {
            P { x: self }
        }
    }
    let mut v = [P { x: 0.0 }];
    assert_eq!(go(P { x: 0.0 }, &mut v), P { x: 23.0 });
}

#[test]
fn a_user_variable_named_like_the_generated_binding_on_the_right_still_resolves() {
    // The binding resolves at the call site (see the emitter for why not
    // mixed-site hygiene), so a same-named user variable on the right is
    // shadowed by its own value, harmlessly. The same name as the *place* is a
    // compile error rather than a misresolve; `tests/ui/binding_collision.rs`.
    let __reassoc_rhs_9f2c1a = 2.0f64;
    let mut x = 10.0f64;
    alg!(x += __reassoc_rhs_9f2c1a);
    assert_eq!(x, 12.0);
    let mut v = [1.0f64];
    alg!(v[0] += __reassoc_rhs_9f2c1a * x);
    assert_eq!(v[0], 25.0);
}

// ---------------------------------------------------------------------------
// More place shapes: raw pointer deref, nested index, tuple fields
// ---------------------------------------------------------------------------

/// `*p` for a raw pointer is a place; `&mut *p` reborrows it. The one `unsafe`
/// shape not covered above.
#[test]
fn raw_pointer_deref_place() {
    #[algebraic]
    unsafe fn bump(p: *mut f64, x: f64) {
        unsafe { *p += x * 2.0 }
    }
    let mut v = 1.0f64;
    unsafe { bump(&mut v, 3.0) };
    assert_eq!(v, 7.0);
}

/// A place that is an index of an index, and a tuple field: nested, so the
/// re-emitted `&mut t.0.1` is three tokens and not the float literal `0.1`.
#[test]
fn nested_index_and_tuple_field_places() {
    #[algebraic]
    fn go(m: &mut [[f64; 2]; 2], t: &mut (f64, (f64, f64)), k: f64) {
        m[1][0] += k;
        m[0][1] *= m[1][0];
        t.0 += k;
        t.1.1 -= t.0;
        (*t).1.0 /= k;
    }
    let mut m = [[1.0, 2.0], [3.0, 4.0]];
    let mut t = (1.0, (8.0, 2.0));
    go(&mut m, &mut t, 2.0);
    assert_eq!(m, [[1.0, 10.0], [5.0, 4.0]]);
    assert_eq!(t, (3.0, (4.0, -1.0)));
}

/// A rewritten compound statement is a call (`ops::unit(match ..)`), not a
/// bare `match`, so the user's `;` stays exactly where it was. Every
/// following-token class a statement can start with, tail position, and every
/// enclosing shape, with the values checked against plain `f64`, so neither
/// order nor what executes can drift.
#[test]
fn compound_statements_keep_their_meaning_in_every_position() {
    use reassoc::{alg, algebraic};
    #[allow(unused_must_use, clippy::no_effect, clippy::unnecessary_operation)]
    #[algebraic]
    fn go(
        mut x: f64,
        p: &mut f64,
        v: &mut [f64; 2],
        mut d: Dispatched,
    ) -> (f64, f64, [f64; 2], Dispatched, f64) {
        let k = 2.0;
        x += k; // followed by a deref-assignment statement
        *p = x;
        x += k; // a unary-minus expression statement
        -x;
        x += k; // destructuring assignment, tuple
        (x, *p) = (*p, x);
        x += k; // destructuring assignment, slice
        [v[0], v[1]] = [x, *p];
        x += k; // a reference expression statement
        &x;
        x += k; // a `!` expression statement
        !true;
        x += k; // a plain assignment
        x = x * 1.0;
        x += k; // a `let`
        let y = x;
        x += k; // a block
        {
            let _ = y;
        }
        x += k; // a literal
        1.0;
        x += k; // a macro statement (entered)
        assert!(x > 0.0);
        x += k; // an `if`
        if x > 0.0 {
            x -= 1.0
        } else {
            x += 1.0
        }
        x += k; // a `match` with arm-body compounds (not statements)
        match y as i64 % 2 {
            0 => x += 0.5,
            _ => x -= 0.5,
        }
        x += k; // a loop whose last statement is a compound
        for _ in 0..2 {
            x *= 1.5;
        }
        x += k; // a block ending in a compound
        {
            x /= 2.0;
        }
        x += k; // a closure whose body is a compound statement
        let mut bump = |z: f64| {
            x += z;
        };
        bump(k);
        x += k; // another compound, consecutive
        x *= 1.0;
        x -= k; // `alg!` block form inside an algebraic scope
        alg! { x += k; x += k; }
        d += Dispatched(1.0); // the dispatch-only type, through an index too
        let mut arr = [d];
        arr[0] += Dispatched(1.0);
        let tail = {
            let mut t = x;
            t += k; // last statement of a block expression
            t
        };
        x += k; // tail position of the function body after this: fine as `()`
        (x, *p, *v, arr[0], tail)
    }
    let mut p = 0.0;
    let mut v = [0.0; 2];
    let (x, pp, vv, d, tail) = go(1.0, &mut p, &mut v, Dispatched(1.0));
    // Recomputed by hand with plain `f64`: the values are exact, so any change
    // in order or in what executes shows up.
    let mut ex = 1.0f64;
    let mut ep: f64;
    let k = 2.0;
    ex += k;
    ep = ex;
    ex += k;
    ex += k;
    (ex, ep) = (ep, ex);
    ex += k;
    let ev = [ex, ep];
    ex += k;
    ex += k;
    ex += k;
    ex += k;
    let ey = ex;
    ex += k;
    ex += k;
    ex += k;
    ex += k;
    if ex > 0.0 {
        ex -= 1.0
    } else {
        ex += 1.0
    }
    ex += k;
    match ey as i64 % 2 {
        0 => ex += 0.5,
        _ => ex -= 0.5,
    }
    ex += k;
    for _ in 0..2 {
        ex *= 1.5;
    }
    ex += k;
    ex /= 2.0;
    ex += k;
    ex += k;
    ex += k;
    ex *= 1.0;
    ex -= k;
    ex += k;
    ex += k;
    let et = ex + k;
    ex += k;
    assert_eq!((x, pp, vv, d, tail), (ex, ep, ev, Dispatched(3.0), et));
}

/// A compound left native by the literal rule is untouched, tokens and all,
/// beside a rewritten one; a user's `;;` after a rewritten compound is still
/// a redundant semicolon for rustc (`tests/ui/redundant_semicolon_after_compound.rs`).
#[test]
fn native_and_rewritten_compounds_side_by_side() {
    use reassoc::algebraic;
    #[algebraic]
    fn go(mut i: usize, mut x: f64) -> (usize, f64) {
        i += 1; // native: `{integer}` literal on the right
        i += 2 * 3;
        x += 1.0; // rewritten: no `;` emitted, same value
        (i, x)
    }
    assert_eq!(go(0, 0.5), (7, 1.5));
}

/// A place wrapped in more than one paren layer: the emitter strips exactly
/// one, so the place check must see through the rest rather than report an
/// invalid left-hand side (`tests/ui/invalid_place.rs` is the real E0067).
#[test]
#[allow(unused_parens)]
#[rustfmt::skip] // rustfmt would fold the very parens under test
fn a_doubly_parenthesised_place_is_still_a_place() {
    use reassoc::alg;
    let mut x = 1.0f64;
    let mut v = [1.0f64; 2];
    alg!(((x)) += 2.0);
    alg!(((v[1])) *= 3.0);
    alg! { (((x))) -= 0.5; }
    assert_eq!((x, v), (2.5, [1.0, 3.0]));
}

/// The RHS is bound before the place, so an overloaded `+=` through a
/// trait-indexed container compiles here where plain Rust is `E0502`: native
/// runs `index_mut` before `index`, and this reads the right-hand side first,
/// lets that borrow end, and borrows the place after. The program is correct
/// either way, there being no aliasing at any point.
///
/// Pinned because it is a documented divergence (`docs/limitations.md`), and
/// because the alternative was measured: emitting the place first reproduces
/// native's `E0502` here and introduces one for `Vec<f32>`, which native
/// accepts. `slice_of_an_opted_in_type` is the control, native accepting that
/// one too, since built-in indexing takes no such borrow.
#[test]
fn overloaded_compound_assign_through_a_vec_index() {
    use core::ops::AddAssign;

    #[derive(Clone, Copy, Debug, PartialEq)]
    #[reassoc::passthrough]
    struct V(f64);
    impl AddAssign for V {
        fn add_assign(&mut self, o: V) {
            self.0 += o.0;
        }
    }

    #[algebraic]
    fn vec_index(v: &mut Vec<V>) {
        v[0] += v[1]; // `E0502` without the attribute
    }

    #[algebraic]
    fn slice_of_an_opted_in_type(v: &mut [V]) {
        v[0] += v[1]; // native accepts this one as well
    }

    #[algebraic]
    fn vec_of_a_primitive(v: &mut Vec<f64>) {
        v[0] += v[1]; // and this one: the case the RHS-first order protects
    }

    let mut owned = vec![V(1.0), V(2.0)];
    vec_index(&mut owned);
    assert_eq!(owned, [V(3.0), V(2.0)]);

    let mut slice = [V(1.0), V(2.0)];
    slice_of_an_opted_in_type(&mut slice);
    assert_eq!(slice, [V(3.0), V(2.0)]);

    let mut floats = vec![1.0f64, 2.0];
    vec_of_a_primitive(&mut floats);
    assert_eq!(floats, [3.0, 2.0]);
}
