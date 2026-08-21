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

#[test]
fn reference_operands_dispatch() {
    let (a, b) = (3.0f32, 4.0f32);
    assert_eq!(mul(&a, &b), 12.0);
    assert_eq!(mul(a, &b), 12.0);
    assert_eq!(mul(&a, b), 12.0);
}
