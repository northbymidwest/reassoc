struct Grid(Vec<f32>);
impl Grid { fn sum(&self) -> f32 { self.0.iter().sum() } }
#[reassoc::algebraic]
fn f(g: &Grid) -> f32 { g.sum() }
fn main() {}
