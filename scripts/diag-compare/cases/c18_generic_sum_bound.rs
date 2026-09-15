#[reassoc::algebraic]
fn total<T: core::iter::Sum<T>>(it: impl Iterator<Item = T>) -> T { it.sum() }
fn main() {}
