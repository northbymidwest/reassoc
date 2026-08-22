fn f<T: core::ops::Mul<Output = T> + Copy>(a: T, b: T) -> T { reassoc::alg!(a * b) }
fn main() {}
