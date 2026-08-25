//! A mismatched operand must be blamed on the mismatch, not on the left type.
//!
//! Native Rust rejects each of these with "expected `X`, found `Y`". The
//! dispatch layer used to turn every one of them into "`X` can't be used with
//! `+`", and advise `passthrough!(X)`; for types that are already opted in,
//! where the advice could not have helped. This pins the wording that
//! replaced it, across every family of operand type the crate covers.
use core::num::Wrapping;
use core::time::Duration;
use reassoc::alg;

#[derive(Clone, Copy, reassoc::Passthrough)]
struct Metres(f64);
impl core::ops::Add for Metres {
    type Output = Metres;
    fn add(self, o: Metres) -> Metres {
        Metres(self.0 + o.0)
    }
}

fn float_widths(a: f32, b: f64) -> f64 {
    alg!(a + b)
}

fn float_widths_by_reference(a: &f64, b: &f32) -> f64 {
    alg!(a * b)
}

fn integer_widths(a: u8, b: u32) -> u32 {
    alg!(a + b)
}

fn signedness(a: i32, b: u32) -> i32 {
    alg!(a + b)
}

fn integer_against_float(a: u32, b: f64) -> f64 {
    alg!(a + b)
}

fn wrapped_integers(a: Wrapping<u8>, b: Wrapping<u32>) -> Wrapping<u32> {
    alg!(a + b)
}

// `Duration * u32` is deliberately heterogeneous: the message must name the
// type the operator actually takes, not the type on the left.
fn heterogeneous_operator(a: Duration, b: u64) -> Duration {
    alg!(a * b)
}

fn opted_in_type(a: Metres, b: f64) -> Metres {
    alg!(a + b)
}

fn main() {
    let _ = float_widths(1.0, 2.0);
    let _ = float_widths_by_reference(&1.0, &2.0);
    let _ = integer_widths(1, 2);
    let _ = signedness(1, 2);
    let _ = integer_against_float(1, 2.0);
    let _ = wrapped_integers(Wrapping(1), Wrapping(2));
    let _ = heterogeneous_operator(Duration::from_secs(1), 2);
    let _ = opted_in_type(Metres(1.0), 2.0);
}
