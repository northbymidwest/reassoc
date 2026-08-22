//! A `const fn` member with nothing of its own to rewrite is skipped — but a
//! `const fn` nested inside it whose arithmetic *would* be rewritten is still
//! reported, not left strict without a word.
use reassoc::algebraic;

struct K;

#[algebraic]
impl K {
    const fn outer(a: f32) -> f32 {
        const fn inner(b: f32) -> f32 {
            b * 2.0
        }
        inner(a)
    }
}

fn main() {}
