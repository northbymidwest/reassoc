//! `ops::*` under `const-fn` (nightly): the same functions, `const fn` with
//! `[const]` bounds, so a rewritten `const fn` body can call them. Kept in
//! its own file because `[const]` is gated at parse time.

use crate::traits::{
    AddAssignRhs, AddRhs, DivAssignRhs, DivRhs, MulAssignRhs, MulRhs, RemAssignRhs, RemRhs,
    SubAssignRhs, SubRhs,
};

#[inline(always)]
#[track_caller]
pub const fn add<A, B: [const] AddRhs<A, O, T>, O, T>(a: A, b: B) -> O {
    b.add_rhs(a)
}

#[inline(always)]
#[track_caller]
pub const fn sub<A, B: [const] SubRhs<A, O, T>, O, T>(a: A, b: B) -> O {
    b.sub_rhs(a)
}

#[inline(always)]
#[track_caller]
pub const fn mul<A, B: [const] MulRhs<A, O, T>, O, T>(a: A, b: B) -> O {
    b.mul_rhs(a)
}

#[inline(always)]
#[track_caller]
pub const fn div<A, B: [const] DivRhs<A, O, T>, O, T>(a: A, b: B) -> O {
    b.div_rhs(a)
}

#[inline(always)]
#[track_caller]
pub const fn rem<A, B: [const] RemRhs<A, O, T>, O, T>(a: A, b: B) -> O {
    b.rem_rhs(a)
}

#[inline(always)]
#[track_caller]
pub const fn add_assign<A, B: [const] AddAssignRhs<A, T>, T>(a: &mut A, b: B) {
    b.add_assign_rhs(a)
}

#[inline(always)]
#[track_caller]
pub const fn sub_assign<A, B: [const] SubAssignRhs<A, T>, T>(a: &mut A, b: B) {
    b.sub_assign_rhs(a)
}

#[inline(always)]
#[track_caller]
pub const fn mul_assign<A, B: [const] MulAssignRhs<A, T>, T>(a: &mut A, b: B) {
    b.mul_assign_rhs(a)
}

#[inline(always)]
#[track_caller]
pub const fn div_assign<A, B: [const] DivAssignRhs<A, T>, T>(a: &mut A, b: B) {
    b.div_assign_rhs(a)
}

#[inline(always)]
#[track_caller]
pub const fn rem_assign<A, B: [const] RemAssignRhs<A, T>, T>(a: &mut A, b: B) {
    b.rem_assign_rhs(a)
}
