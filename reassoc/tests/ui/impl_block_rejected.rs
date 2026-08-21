use reassoc::algebraic;

struct Foo;

#[algebraic]
impl Foo {
    fn bar(&self) -> i32 {
        1
    }
}

fn main() {}
