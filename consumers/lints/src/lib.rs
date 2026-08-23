//! What the rewriter emits must be lint-clean under clippy's pedantic set
//! where it used not to be, and must not hide warnings the user's own code
//! deserves. The deny is on the specific lints, not the whole group, so a
//! new pedantic lint about our *test* code cannot break the build.
#![deny(
    clippy::unnecessary_semicolon,
    clippy::semicolon_if_nothing_returned,
    clippy::match_single_binding,
    clippy::needless_return,
    redundant_semicolons,
    unused_parens,
    unused_braces
)]
#![allow(dead_code, clippy::missing_const_for_fn)]

use reassoc::{alg, algebraic};

#[derive(Clone, Copy, Debug, PartialEq, reassoc::Passthrough)]
pub struct V(pub f32);
impl core::ops::Add for V {
    type Output = V;
    fn add(self, o: V) -> V {
        V(self.0 + o.0)
    }
}
impl core::ops::AddAssign for V {
    fn add_assign(&mut self, o: V) {
        self.0 += o.0;
    }
}

/// Every shape of compound statement the rewriter emits, all clean.
#[algebraic]
pub fn clean(mut x: f32, k: f32, v: &mut [f32], p: &mut f32, mut w: V, mut i: usize) -> f32 {
    x += k; // followed by a statement
    *p += k; // a deref place
    v[0] += k; // an index place
    w += V(1.0); // a user type, through its own `AddAssign`
    i += 1; // native (literal rule): keeps its `;`, also clean
    i += i; // rewritten integer compound
    if x > 0.0 {
        x -= k; // last statement of a branch
    } else {
        x *= k;
    }
    for y in v.iter() {
        x += y; // last statement of a loop body
    }
    let mut bump = |z: f32| {
        x += z; // a closure whose body ends in a compound
    };
    bump(k);
    'blk: {
        x /= k; // last statement of a labeled block
        if x > 0.0 {
            break 'blk;
        }
    }
    let t = {
        let mut t = x;
        t += k; // last statement of a block expression with a tail
        t
    };
    alg! {
        x += t; // the block form of `alg!`
        x += k;
    }
    x += k; // last statement of the function body
    x + w.0 + i as f32
}

/// The expression form in statement position, with the caller's `;` after
/// it, and the brace form without one: both clean.
pub fn expression_forms(mut x: f32, k: f32) -> f32 {
    alg!(x += k);
    alg! { x += k }
    x
}

/// Warnings that are the user's own must survive the rewrite: a redundant
/// `;;`, an unnecessary `;` after a native block-like statement, redundant
/// parens. `expect` fails the build if the lint does *not* fire.
#[rustfmt::skip] // rustfmt would fold the `;;`
#[expect(redundant_semicolons)]
#[algebraic]
pub fn users_redundant_semicolon(mut x: f32, k: f32) -> f32 {
    x += k;;
    x
}

/// Both of the user's: the `;` after the `if`, and the `+=` tail without one.
#[expect(clippy::unnecessary_semicolon, clippy::semicolon_if_nothing_returned)]
#[algebraic]
pub fn users_unnecessary_semicolon(mut x: f32, k: f32) -> f32 {
    if x > 0.0 {
        x += k
    };
    x
}

#[expect(unused_parens)]
#[algebraic]
pub fn users_redundant_parens(x: f32, k: f32) -> f32 {
    let y = (x * k);
    y
}

/// A compound assignment written as a block's tail without `;` is the user's
/// own style, and clippy's `semicolon_if_nothing_returned` reports it on
/// native `+=` too; the rewrite keeps that.
#[expect(clippy::semicolon_if_nothing_returned)]
#[algebraic]
pub fn users_tail_without_semicolon(mut x: f32, k: f32) -> f32 {
    if x > 0.0 {
        x += k
    }
    x
}
