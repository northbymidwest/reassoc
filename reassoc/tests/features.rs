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
