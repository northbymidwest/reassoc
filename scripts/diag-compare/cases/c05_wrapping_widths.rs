use std::num::Wrapping;
fn f(a: Wrapping<u8>, b: Wrapping<u32>) -> Wrapping<u32> { reassoc::alg!(a + b) }
fn main() {}
