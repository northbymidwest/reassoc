//! A type that is not `Copy` opted in without `no_refs`, through the derive and
//! through the macro. The error must lead with the way out — `RefOperand`'s
//! note naming `no_refs` — not with a bare "`Owned: Copy` is not satisfied"
//! from the `Synth*` marker's supertrait, which is what a `Copy` supertrait
//! produced.
use reassoc::passthrough;

#[derive(Clone, reassoc::Passthrough)]
#[passthrough(add)]
struct Derived(String);
impl core::ops::Add for Derived {
    type Output = Derived;
    fn add(self, o: Derived) -> Derived {
        Derived(self.0 + &o.0)
    }
}

#[derive(Clone)]
struct Declared(String);
impl core::ops::Add for Declared {
    type Output = Declared;
    fn add(self, o: Declared) -> Declared {
        Declared(self.0 + &o.0)
    }
}
passthrough!(add: Declared, Declared => Declared);

fn main() {}
