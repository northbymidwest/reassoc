//! Integer impls, generated with the crate's own public `passthrough!`.

macro_rules! plain_int {
    ($($t:ty)*) => {$( $crate::passthrough!($t); )*};
}

plain_int!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);
