
#[derive(Clone,Copy,PartialEq,PartialOrd,Debug)]
pub struct Angle {
    data: [f64;2]
}
impl std::ops::Add<Self> for Angle {
    type Output = Self;
    #[inline(always)]
    fn add(self, other: Self) -> Self {
        Self {
            data: [
                self.sin() * other.cos() + self.cos() * other.sin(),
                self.cos() * other.cos() - self.sin() * other.sin(),
            ]
        }
    }
}
impl std::ops::Sub<Self> for Angle {
    type Output = Self;
    #[inline(always)]
    fn sub(self, other: Self) -> Self {
        Self {
            data: [
                self.sin() * other.cos() - self.cos() * other.sin(),
                self.cos() * other.cos() + self.sin() * other.sin(),
            ]
        }
    }
}
impl Angle {

    #[inline(always)]
    pub(in crate) fn sin(&self) -> f64 { self.data[0] }

    #[inline(always)]
    pub(in crate) fn cos(&self) -> f64 { self.data[1] }

    #[inline(always)]
    pub(in crate) fn atan2(&self) -> f64 {
        self.sin().atan2(self.cos())
    }

    #[inline(always)]
    pub(in crate) fn atan2_to_degrees<S,C>(&self, sin_mod: S, cos_mod: C) -> f64
    where
        S: Into<Option<f64>>,
        C: Into<Option<f64>>,
    {
        use crate::geomath;
        let sin = self.sin();
        let cos = self.cos();
        geomath::atan2(
            sin_mod.into().map(|x| x * sin).unwrap_or_else(|| sin),
            cos_mod.into().map(|y| y * cos).unwrap_or_else(|| cos),
        )
    }


    /// Assumes the input is in radians and this is in eucldian space
    #[inline(always)]
    pub(in crate) fn new_simple(radian: f64) -> Self {
        let (sin,cos) = radian.sin_cos();
        Self {
            data: [sin,cos],
        }
    }

    /// Assumes the input is in radians and this is on an ellipsoid with inverse flattening of f1
    #[inline(always)]
    pub(in crate) fn new_for_ellipsoid(radian: f64, f1: f64) -> Self {
        use crate::{
            geomath::self,
            internals::constants::{TINY},
        };

        let (mut sin,mut cos) = geomath::sincosd(ang);
        sin *= self._f1;
        geomath::norm(&mut sin, &mut cos);
        cos = TINY.max(cos);

        Self { data: [sin,cos] }
    }
}
