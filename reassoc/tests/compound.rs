//! Compound assignment, exhaustively. Two expansions exist — a bare path is
//! assigned through by name, anything else goes through
//! `ops::add_assign(&mut place, rhs)` — and both must agree with native `+=`
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
/// guard is a temporary of the RHS *expression* — not a local inside a
/// closure — so its drop marks the end of the statement's temporaries. A
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
    // statement ends — after the place is evaluated and written.
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
        d[0] -= &second;
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
    // arithmetic), so this is plain `AddAssign` on usize — including through
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
impl reassoc::traits::AddRhs<Dispatched, Dispatched> for Dispatched {
    fn add_rhs(self, lhs: Dispatched) -> Dispatched {
        Dispatched(lhs.0 + self.0)
    }
}
impl reassoc::traits::SynthAddAssign<Dispatched> for Dispatched {}

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

#[derive(Debug, Clone, Copy, PartialEq, reassoc::Passthrough)]
#[passthrough(add, mul)]
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

#[derive(Debug, Clone, PartialEq, reassoc::Passthrough)]
#[passthrough(add, add_assign, no_refs)]
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
    struct Acc(f64);
    impl core::ops::AddAssign<&Acc> for Acc {
        fn add_assign(&mut self, o: &Acc) {
            self.0 += o.0;
        }
    }
    reassoc::passthrough!(add_assign: Acc, &Acc);
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
    #[derive(Clone, Copy, Debug, PartialEq, reassoc::Passthrough)]
    #[passthrough(add)]
    struct P {
        x: f64,
    }
    impl core::ops::Add for P {
        type Output = P;
        fn add(self, o: P) -> P {
            P { x: self.x + o.x }
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
