//! `f16` and `f128` as algebraic floats, behind the nightly-only `f16` and
//! `f128` features: the same routing, literal inference and reference forms as
//! `f32`/`f64`. Each half is gated on its own feature, so either works alone;
//! empty on stable. Not included in the edition-2021 twin (it cannot carry the
//! crate-level feature gates as a module).
#![cfg_attr(feature = "f16", feature(f16))]
#![cfg_attr(feature = "f128", feature(f128))]
#![cfg(any(feature = "f16", feature = "f128"))]

#[cfg(feature = "f16")]
mod half {
    use reassoc::{alg, algebraic};

    #[algebraic]
    fn kernel(a: &[f16], b: &[f16]) -> f16 {
        let mut sum = 0.0; // infers `f16` from the return type, as `f32` does
        for i in 0..a.len().min(b.len()) {
            sum += a[i] * b[i];
        }
        sum * 2.0 - 1.0
    }

    #[test]
    fn f16_dispatches_like_f32() {
        let a: [f16; 2] = [1.0, 2.0];
        let b: [f16; 2] = [3.0, 4.0];
        assert_eq!(kernel(&a, &b) as f32, 21.0);
        let t: f16 = alg!(0.5 * 3.0);
        assert_eq!(t as f32, 1.5);
    }
}

#[cfg(feature = "f128")]
mod quad {
    use reassoc::{alg, algebraic};

    #[algebraic]
    fn kernel(a: &[f128], k: f128) -> f128 {
        let mut acc = 0.0;
        for x in a {
            acc += x * k; // `&f128 * f128`
            acc = &acc + x; // `&f128 + &f128`
        }
        -(acc * 0.5) // literal inference under unary minus
    }

    #[test]
    fn f128_dispatches_like_f64() {
        let c: [f128; 2] = [1.0, 2.0];
        assert_eq!(kernel(&c, 2.0) as f64, -4.5);
        let s: f128 = alg!(1.0 + 2.0);
        assert_eq!(s as f64, 3.0);
    }
}
