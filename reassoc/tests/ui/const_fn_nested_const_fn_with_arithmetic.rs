//! A `const fn` member with nothing of its own to rewrite is skipped, but a
//! `const fn` nested inside it whose arithmetic *would* be rewritten is still
//! reported, not left strict without a word.
//!
//! Both orders, because they pin different halves of the same rule. In `K`
//! the arithmetic is the inner function's, so only the inner one is named. In
//! `J` it is the outer function's, so only the outer one is named: a parent's
//! arithmetic must not condemn a nested `const fn` that has none of its own.
//! Two errors between the two impls, never three.
//!
//! `J`'s arithmetic comes *before* its nested function on purpose.
//! Whether a `const fn` has arithmetic of its own is one flag, which
//! each `const fn` saves and clears on entry. Drop the clearing and
//! the other order still passes, the flag being false anyway by the
//! time the nested one is entered; this order is what catches it.
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

struct J;

#[algebraic]
impl J {
    const fn outer(a: f32, b: f32) -> f32 {
        let scaled = a * b;
        const fn inner(c: f32) -> f32 {
            c
        }
        inner(scaled)
    }
}

fn main() {}
