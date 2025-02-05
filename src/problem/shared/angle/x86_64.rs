#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::__m128d;

#[derive(Clone,Copy,PartialEq,PartialOrd,Debug)]
pub struct Angle {
    data: [f64;2],
}
