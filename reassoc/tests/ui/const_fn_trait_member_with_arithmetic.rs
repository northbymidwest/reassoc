//! The `const fn` rule reaches a trait's default bodies, not only an `impl`'s.
//! rustc rejects a `const fn` in a trait on its own account (E0379) and the
//! macro cannot know that, so both errors are expected here. What this pins
//! is that the macro's own error is among them. Without it the arithmetic
//! would be rewritten into calls that are not `const fn`, and the member would
//! be left strict without a word, which is the one thing the container form
//! must never do.
use reassoc::algebraic;

#[algebraic]
trait Scale {
    // Nothing the rewrite would touch: skipped in silence, leaving rustc's
    // own complaint about `const` as the only one.
    const fn unit() -> f32 {
        1.0
    }

    // Arithmetic the rewrite *would* touch: an error naming the way out.
    const fn scale(x: f32) -> f32 {
        x * 2.0
    }
}

fn main() {}
