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
