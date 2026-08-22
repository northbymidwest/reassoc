use core::time::Duration;
use reassoc::ops::{add, div, mul};

#[test]
fn duration_heterogeneous_pairs_work_in_core() {
    let d = Duration::from_secs(2);
    assert_eq!(mul(d, 3u32), Duration::from_secs(6));
    assert_eq!(div(d, 2u32), Duration::from_secs(1));
    assert_eq!(add(d, d), Duration::from_secs(4));
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
