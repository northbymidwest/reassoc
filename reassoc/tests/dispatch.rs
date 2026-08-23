use reassoc::ops::{add, div, mul, rem, sub};

#[test]
fn float_ops_produce_correct_values() {
    assert_eq!(add(1.0f32, 2.0f32), 3.0);
    assert_eq!(sub(1.0f32, 2.0f32), -1.0);
    assert_eq!(mul(3.0f32, 4.0f32), 12.0);
    assert_eq!(div(1.0f32, 2.0f32), 0.5);
    assert_eq!(rem(3.0f32, 2.0f32), 1.0);
    assert_eq!(mul(3.0f64, 4.0f64), 12.0);
}

/// Regression test for the inference property the whole design rests on.
/// If `O` is ever changed to an associated type, this stops compiling.
#[test]
fn unannotated_float_literals_infer_from_return_type() {
    fn accumulate() -> f32 {
        let s = 0.0; // no suffix, no annotation
        add(s, 1.0)
    }
    assert_eq!(accumulate(), 1.0);
}

// The borrows are the point of this test: it exercises the `&T` impls.
// Clippy's suggestion to remove them would delete what is being tested.
#[allow(clippy::needless_borrows_for_generic_args)]
#[test]
fn reference_operands_dispatch() {
    let (a, b) = (3.0f32, 4.0f32);
    assert_eq!(mul(&a, &b), 12.0);
    assert_eq!(mul(a, &b), 12.0);
    assert_eq!(mul(&a, b), 12.0);

    // Both widths: the reference variants live inside one `Alg*` impl per
    // operand type, so each width's four combinations are generated together.
    let (c, d) = (3.0f64, 4.0f64);
    assert_eq!(mul(&c, &d), 12.0);
    assert_eq!(mul(c, &d), 12.0);
    assert_eq!(mul(&c, d), 12.0);
}

#[allow(clippy::needless_borrows_for_generic_args)] // borrows are deliberate
#[test]
fn integer_ops_fall_back_to_plain_operators() {
    assert_eq!(add(2usize, 3usize), 5);
    assert_eq!(sub(10i32, 3i32), 7);
    assert_eq!(mul(6u8, 7u8), 42);
    assert_eq!(div(9i64, 2i64), 4);
    assert_eq!(rem(9u32, 4u32), 1);
    let (a, b) = (6i32, 7i32);
    assert_eq!(mul(&a, &b), 42);
}

#[test]
fn unannotated_integer_literals_infer_from_return_type() {
    fn count() -> usize {
        let n = 3;
        add(n, 4)
    }
    assert_eq!(count(), 7);
}

/// `#[track_caller]` on the dispatch functions and the passthrough impls: a
/// debug-build integer overflow must be reported at the user's operator, not
/// inside this crate. Debug only — release builds do not check overflow.
#[cfg(debug_assertions)]
#[test]
fn integer_overflow_panics_at_the_users_operator() {
    use std::panic;
    use std::sync::{Arc, Mutex};
    let seen: Arc<Mutex<Option<(String, u32)>>> = Arc::new(Mutex::new(None));
    let hook_seen = Arc::clone(&seen);
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        if let Some(loc) = info.location() {
            *hook_seen.lock().unwrap() = Some((loc.file().to_string(), loc.line()));
        }
    }));
    let expected_line = std::sync::atomic::AtomicU32::new(0);
    let result = panic::catch_unwind(|| {
        let (a, b) = (u8::MAX, 1u8);
        expected_line.store(line!() + 1, std::sync::atomic::Ordering::Relaxed);
        add(a, b) // <- the panic must point here
    });
    panic::set_hook(previous);
    assert!(
        result.is_err(),
        "u8::MAX + 1 must overflow in a debug build"
    );
    let (file, line) = seen
        .lock()
        .unwrap()
        .clone()
        .expect("panic hook saw no location");
    assert!(
        file.ends_with("dispatch.rs"),
        "panicked in {file}, not at the caller"
    );
    assert_eq!(
        line,
        expected_line.load(std::sync::atomic::Ordering::Relaxed)
    );
}

// ---- completeness of the macro-generated impl lists ----
//
// The float, integer and `NonZero` impls are stamped out by macros over a
// list of types; a type dropped from a list fails here at compile time.
// Mutation testing cannot see inside those macros, so these are the guard.

macro_rules! every_int_shape {
    ($($t:ty),* $(,)?) => {{
        use reassoc::algebraic;
        #[algebraic]
        fn go<T: Copy + PartialEq + core::fmt::Debug>(_: T) {}
        $({
            #[algebraic]
            #[allow(clippy::op_ref)]
            fn shapes(a: $t, b: $t) -> [$t; 20] {
                let (ra, rb) = (&a, &b);
                let mut c = [a; 5];
                c[0] += b;
                c[1] -= b;
                c[2] *= b;
                c[3] /= b;
                c[4] %= b;
                let mut d = [a; 5];
                d[0] += rb;
                d[1] -= rb;
                d[2] *= rb;
                d[3] /= rb;
                d[4] %= rb;
                [
                    a + b, a - b, a * b, a / b, a % b,
                    ra + rb, ra - rb, ra * rb, ra / rb, ra % rb,
                    c[0], c[1], c[2], c[3], c[4],
                    d[0], d[1], d[2], d[3], d[4],
                ]
            }
            let (a, b): ($t, $t) = (13, 4);
            let native = [a + b, a - b, a * b, a / b, a % b];
            let got = shapes(a, b);
            assert_eq!(&got[0..5], &native, stringify!($t));
            assert_eq!(&got[5..10], &native, stringify!($t));
            assert_eq!(&got[10..15], &native, stringify!($t));
            assert_eq!(&got[15..20], &native, stringify!($t));
            go(a);
        })*
    }};
}

#[test]
fn every_integer_type_dispatches_every_operator_in_every_shape() {
    every_int_shape!(
        i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
    );
}

macro_rules! every_float_shape {
    ($($t:ty),* $(,)?) => {{
        $({
            use reassoc::algebraic;
            #[algebraic]
            #[allow(clippy::op_ref)]
            fn shapes(a: $t, b: $t) -> [$t; 30] {
                let (ra, rb) = (&a, &b);
                let mut c = [a; 5];
                c[0] += b;
                c[1] -= b;
                c[2] *= b;
                c[3] /= b;
                c[4] %= b;
                let mut d = [a; 5];
                d[0] += rb;
                d[1] -= rb;
                d[2] *= rb;
                d[3] /= rb;
                d[4] %= rb;
                [
                    a + b, a - b, a * b, a / b, a % b,
                    ra + b, ra - b, ra * b, ra / b, ra % b,
                    a + rb, a - rb, a * rb, a / rb, a % rb,
                    ra + rb, ra - rb, ra * rb, ra / rb, ra % rb,
                    c[0], c[1], c[2], c[3], c[4],
                    d[0], d[1], d[2], d[3], d[4],
                ]
            }
            let (a, b): ($t, $t) = (13.0, 4.0);
            let native = [a + b, a - b, a * b, a / b, a % b];
            let got = shapes(a, b);
            for k in 0..6 {
                assert_eq!(&got[k * 5..k * 5 + 5], &native, "{} shape {k}", stringify!($t));
            }
        })*
    }};
}

#[test]
fn every_float_type_dispatches_every_operator_in_every_reference_shape() {
    every_float_shape!(f32, f64);
}

#[test]
fn every_unsigned_width_divides_by_its_nonzero() {
    use core::num::NonZero;
    use reassoc::algebraic;
    macro_rules! widths {
        ($($t:ty),* $(,)?) => {$({
            #[algebraic]
            fn go(mut x: $t, n: NonZero<$t>) -> [$t; 4] {
                let q = x / n;
                let r = x % n;
                x /= n;
                x %= n;
                [q, r, x, 0]
            }
            assert_eq!(go(13, NonZero::new(4).unwrap()), [3, 1, 3, 0], stringify!($t));
        })*};
    }
    widths!(u8, u16, u32, u64, u128, usize);
}
