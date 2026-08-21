use reassoc::alg;

fn main() {
    let (a, b, x) = (1.0f32, 2.0f32, 3.0f32);
    alg!(a + b += x);
}
