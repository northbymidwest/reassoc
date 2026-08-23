use core::time::Duration;
use reassoc::ops::{add, div, mul};

#[test]
fn duration_heterogeneous_pairs_work_in_core() {
    let d = Duration::from_secs(2);
    assert_eq!(mul(d, 3u32), Duration::from_secs(6));
    assert_eq!(div(d, 2u32), Duration::from_secs(1));
    assert_eq!(add(d, d), Duration::from_secs(4));
    // `u32 * Duration`: the integer-left blanket, with a non-literal left.
    let n = 3u32;
    assert_eq!(mul(n, d), Duration::from_secs(6));
    assert_eq!(reassoc::alg!(n * d), Duration::from_secs(6));
}

#[cfg(feature = "alloc")]
#[test]
fn string_concatenation_works() {
    let s = String::from("a");
    assert_eq!(add(s, "b"), "ab");
    // Native `String + &String` works only because rustc deref-coerces the
    // operand once the impl is unique; a generic dispatch function never
    // does, so these need impls of their own.
    let other = String::from("b");
    assert_eq!(add(String::from("a"), &other), "ab");
    let boxed: Box<str> = "b".into();
    assert_eq!(add(String::from("a"), &boxed), "ab");
}

#[cfg(feature = "alloc")]
#[test]
fn string_compound_assignment_through_a_reference_or_index() {
    use reassoc::algebraic;
    struct Acc {
        name: String,
    }
    impl Acc {
        #[algebraic]
        fn tag(&mut self, s: &str, t: &String) {
            // A `&mut self` field is not a simple place: this goes through
            // `ops::add_assign` and `String`'s own `AddAssign`, in place.
            self.name += s;
            self.name += t;
        }
    }
    #[algebraic]
    fn tag_all(names: &mut [String], s: &str) {
        names[0] += s;
    }
    let mut a = Acc { name: "n".into() };
    a.tag("!", &"?".to_string());
    let mut names = [String::from("a")];
    tag_all(&mut names, "x");
    assert_eq!((a.name.as_str(), names[0].as_str()), ("n!?", "ax"));
}

#[cfg(feature = "std")]
#[test]
fn instant_minus_instant_is_a_duration() {
    use reassoc::ops::sub;
    let t = std::time::Instant::now();
    assert_eq!(sub(t, t), Duration::ZERO);
}

#[cfg(feature = "std")]
#[test]
fn system_time_plus_duration_works() {
    let t = std::time::SystemTime::UNIX_EPOCH;
    assert_eq!(
        add(t, Duration::from_secs(1)),
        std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(1)
    );
}

#[test]
fn wrapping_subtraction_and_the_narrow_integers() {
    use core::num::Wrapping;
    use reassoc::ops::{add, mul, sub};
    assert_eq!(sub(Wrapping(0u8), Wrapping(1u8)), Wrapping(255u8));
    assert_eq!(add(1i8, 2i8), 3i8);
    assert_eq!(add(1i16, 2i16), 3i16);
    assert_eq!(mul(3u16, 4u16), 12u16);
    assert_eq!(add(1i128, 2i128), 3i128);
    assert_eq!(mul(3u128, 4u128), 12u128);
    assert_eq!(sub(5isize, 2isize), 3isize);
}

#[cfg(feature = "alloc")]
#[test]
fn string_plus_a_cow_goes_through_as_ref() {
    use alloc_or_std::borrow::Cow;
    let c: Cow<str> = Cow::Borrowed("b");
    assert_eq!(add(String::from("a"), &c), "ab");
    let owned: Cow<str> = Cow::Owned(String::from("c"));
    assert_eq!(add(String::from("a"), &owned), "ac");
}

#[cfg(feature = "alloc")]
extern crate alloc as alloc_or_std;

#[cfg(feature = "std")]
#[test]
fn system_time_minus_duration_works() {
    use reassoc::ops::sub;
    let t = std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(10);
    assert_eq!(
        sub(t, Duration::from_secs(1)),
        std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(9)
    );
}

/// `u32 / NonZero<u32>` and `%` exist natively (the idiom for a division
/// that cannot trap), with `/=` and `%=`, by value only: no reference forms
/// and no `NonZero * NonZero`, so none are emitted here either.
#[test]
fn unsigned_division_by_nonzero_matches_native() {
    use core::num::NonZero;
    use reassoc::algebraic;
    #[algebraic]
    fn go(mut x: u32, n: NonZero<u32>, y: usize, m: NonZero<usize>) -> (u32, u32, u32, usize) {
        let q = x / n;
        let r = x % n;
        x /= n;
        x %= NonZero::new(2).unwrap();
        (q, r, x, y % m)
    }
    let n = NonZero::new(3u32).unwrap();
    assert_eq!(go(7, n, 9, NonZero::new(4).unwrap()), (2, 1, 0, 1));
    let _ = (
        div(7u8, NonZero::new(2u8).unwrap()),
        div(7u128, NonZero::new(2u128).unwrap()),
    );
}

/// Native `+=` on a `String` deref-coerces its right operand once the impl is
/// unique: `&Cow<str>`, `&Box<str>`, `&&str`, `&&String`, `&Rc<str>`,
/// `&Arc<str>`, `&mut String`, `&mut str` are all accepted. Dispatch never
/// coerces, so each needs an in-place impl of its own; `+` accepts the same
/// set through `AsRef<str>`.
#[cfg(feature = "alloc")]
#[test]
fn string_in_place_accepts_every_reference_native_would_coerce() {
    use alloc_or_std::borrow::Cow;
    use alloc_or_std::rc::Rc;
    use alloc_or_std::sync::Arc;
    use reassoc::algebraic;
    // The reference shapes are the point of the test.
    #[allow(clippy::borrowed_box, clippy::ptr_arg, clippy::too_many_arguments)]
    #[algebraic]
    fn go(
        mut s: String,
        c: &Cow<str>,
        b: &Box<str>,
        r: &&str,
        o: &&String,
        rc: &Rc<str>,
        arc: &Arc<str>,
        m: &mut String,
        ms: &mut str,
    ) -> String {
        s += c;
        s += b;
        s += r;
        s += o;
        s += rc;
        s += arc;
        // A `&mut` right operand is moved into the dispatch call rather than
        // implicitly reborrowed as native `+=` would; reborrow to reuse it.
        s += &mut *m;
        s += &mut *ms;
        s = s + &mut *m;
        s = s + ms;
        s + m
    }
    let owned = String::from("d");
    let mut m = String::from("g");
    let mut ms = String::from("h");
    let s = go(
        String::new(),
        &Cow::Borrowed("a"),
        &"b".into(),
        &"c",
        &&owned,
        &Rc::from("e"),
        &Arc::from("f"),
        &mut m,
        ms.as_mut_str(),
    );
    assert_eq!(s, "abcdefghghg");
}
