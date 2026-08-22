struct Odd;
fn f(a: Odd, b: Odd) -> Odd { reassoc::alg!(a * b) }
fn main() {}
