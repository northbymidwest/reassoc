struct S<'a> { it: &'a mut core::slice::Iter<'a, f32> }
impl S<'_> {
    #[reassoc::algebraic]
    fn drain(&mut self) -> f32 { self.it.sum() }
}
fn main() {}
