use core::time::Duration;
use reassoc::ops::{add, mul};
use reassoc::{passthrough, strict};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec3(f32, f32, f32);

impl core::ops::Add for Vec3 {
    type Output = Vec3;
    fn add(self, o: Vec3) -> Vec3 { Vec3(self.0 + o.0, self.1 + o.1, self.2 + o.2) }
}
impl core::ops::Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, o: Vec3) -> Vec3 { Vec3(self.0 - o.0, self.1 - o.1, self.2 - o.2) }
}
impl core::ops::Mul for Vec3 {
    type Output = Vec3;
    fn mul(self, o: Vec3) -> Vec3 { Vec3(self.0 * o.0, self.1 * o.1, self.2 * o.2) }
}
impl core::ops::Div for Vec3 {
    type Output = Vec3;
    fn div(self, o: Vec3) -> Vec3 { Vec3(self.0 / o.0, self.1 / o.1, self.2 / o.2) }
}
impl core::ops::Rem for Vec3 {
    type Output = Vec3;
    fn rem(self, o: Vec3) -> Vec3 { Vec3(self.0 % o.0, self.1 % o.1, self.2 % o.2) }
}

passthrough!(Vec3);

#[derive(Debug, Clone, Copy, PartialEq)]
struct Scaled(u32);

impl core::ops::Mul<u32> for Scaled {
    type Output = Scaled;
    fn mul(self, n: u32) -> Scaled { Scaled(self.0 * n) }
}

passthrough!(mul: Scaled, u32 => Scaled);

#[test]
fn same_type_passthrough_covers_all_five_operators() {
    let a = Vec3(1.0, 2.0, 3.0);
    assert_eq!(add(a, a), Vec3(2.0, 4.0, 6.0));
    assert_eq!(mul(a, a), Vec3(1.0, 4.0, 9.0));
}

#[test]
fn heterogeneous_passthrough_covers_one_operator() {
    assert_eq!(mul(Scaled(3), 4u32), Scaled(12));
}

#[test]
fn strict_is_an_identity_macro() {
    let (t, sum, y) = (3.0f32, 2.0f32, 1.0f32);
    assert_eq!(strict!((t - sum) - y), 0.0);
    assert_eq!(strict!(Duration::from_secs(1)), Duration::from_secs(1));
}
