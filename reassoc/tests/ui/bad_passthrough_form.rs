//! A `passthrough!` invocation that is not one of the documented forms gets an
//! authored error naming them, not `macro_rules!`'s "no rules expected" from
//! deep inside the expansion — and not the recursion limit, which is what the
//! catch-all arm would produce if it re-wrapped an already-tagged form.
#[derive(Clone, Copy)]
struct Odd(f32);

reassoc::passthrough!(mul out Odd, Odd => f32);
reassoc::passthrough!(foreign times: Odd, Odd => Odd);

fn main() {}
