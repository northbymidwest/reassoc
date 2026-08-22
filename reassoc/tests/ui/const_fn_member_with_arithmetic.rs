//! A `const fn` inside an annotated container cannot have its arithmetic
//! rewritten (`reassoc::ops::*` are not `const fn`). One whose body the
//! rewrite would not touch — `const fn new(x) -> Self { Self(x) }` — is skipped
//! silently; one it *would* touch is an error naming the way out, so the
//! container form never leaves a method strict without saying so.
use reassoc::algebraic;

#[derive(Clone, Copy)]
struct V(f32);

#[algebraic]
impl V {
    pub const fn new(x: f32) -> V {
        V(x) // nothing to rewrite: fine
    }
    pub const fn scale(self, k: f32) -> V {
        V(self.0 * k) // would be rewritten: rejected
    }
}

// The same rule where `items = true` meets a nested `const fn`.
#[algebraic(items = true)]
fn outer(x: f32) -> f32 {
    const fn twice(y: f32) -> f32 {
        y * 2.0
    }
    twice(x)
}

fn main() {}
