use std::time::Duration;
fn f(d: Duration, n: u64) -> Duration { reassoc::alg!(d * n) }
fn main() {}
