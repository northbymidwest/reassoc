use reassoc::alg;

#[test]
fn rewrites_binary_operators() {
    let (a, b, c) = (2.0f32, 3.0f32, 4.0f32);
    assert_eq!(alg!(a * b), 6.0);
    assert_eq!(alg!(a + b), 5.0);
    assert_eq!(alg!(b - a), 1.0);
    assert_eq!(alg!(c / a), 2.0);
    assert_eq!(alg!(c % b), 1.0);
}

#[test]
fn respects_precedence_and_nesting() {
    let (a, b, c, d) = (2.0f32, 3.0f32, 4.0f32, 8.0f32);
    assert_eq!(alg!(a * b + c / a), 8.0);
    assert_eq!(alg!((a + b) * c), 20.0);
    assert_eq!(alg!(d / (a * a)), 2.0);
}

#[test]
fn leaves_integer_arithmetic_working() {
    let v = [1.0f32, 2.0, 3.0];
    let n = 2usize;
    assert_eq!(alg!(v[n - 1] * 2.0), 4.0);
}

#[test]
fn rewrites_inside_calls_and_indices() {
    fn twice(x: f32) -> f32 { x * 2.0 }
    let (a, b) = (2.0f32, 3.0f32);
    assert_eq!(alg!(twice(a * b)), 12.0);
}

#[test]
fn does_not_rewrite_unary_negation() {
    // There is no algebraic_neg; this must still compile and behave normally.
    let a = 2.0f32;
    assert_eq!(alg!(-a * a), -4.0);
}
