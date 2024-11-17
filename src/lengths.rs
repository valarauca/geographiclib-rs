
//#[cfg(test)] use approx::assert_relative_eq;
//use crate::internals::constants::{C1F_COEFF,C2F_COEFF};
use crate::cached_weights::{Weights,/*CoefficientSelection,*/C1fCoeff,C2fCoeff};
use crate::geomath;


pub(in crate) trait LengthsReturnValue: Sized + Default {
    const DISTANCE: bool = false;
    const REDUCED_LENGTH: bool = false;
    const GEODESIC_SCALE: bool = false;

    #[inline(always)] fn set_s12b(&mut self, _s12b: f64) { }
    #[inline(always)] fn set_m12b(&mut self, _m12b: f64) { }
    #[inline(always)] fn set_m0(&mut self, _m0: f64) { }
    #[inline(always)] fn set_m12(&mut self, _m12: f64) { }
    #[inline(always)] fn set_m21(&mut self, _m21: f64) { }

    #[inline(always)] fn get_s12b(&self) -> f64 { f64::NAN }
    #[inline(always)] fn get_m12b(&self) -> f64 { f64::NAN }
    #[inline(always)] fn get_m0(&self) -> f64 { f64::NAN }
    #[inline(always)] fn get_m12(&self) -> f64 { f64::NAN }
    #[inline(always)] fn get_m21(&self) -> f64 { f64:: NAN }

    #[inline(always)]
    fn invoke(
        weights: &Weights,
        ep2: f64,
        eps: f64,
        sig12: f64,
        ssig1: f64,
        csig1: f64,
        dn1: f64,
        ssig2: f64,
        csig2: f64,
        dn2: f64,
        cbet1: f64,
        cbet2: f64,
    ) -> Self {
        let mut s = Self::default();

        let mut a1 = 0.0;
        let mut a2 = 0.0;
        let mut m0x = 0.0;
        let mut j12 = 0.0;

        if Self::DISTANCE | Self::REDUCED_LENGTH | Self::GEODESIC_SCALE {
            a1 = weights.get_a1m1f(eps);
            if Self::REDUCED_LENGTH | Self::GEODESIC_SCALE {
                a2 = weights.get_a2m1f(eps);
                m0x = a1 - a2;
                a2 += 1.0;
            }
            a1 += 1.0;
        }

        if Self::DISTANCE {
            let b1 = weights.difference_of_meridian_arc_lengths::<C1fCoeff>(eps, ssig1, csig1, ssig2, csig2);
            s.set_s12b(a1 * (sig12 + b1));
            if Self::REDUCED_LENGTH | Self::GEODESIC_SCALE {
                let b2 = weights.difference_of_meridian_arc_lengths::<C2fCoeff>(eps, ssig1, csig1, ssig2, csig2);
                j12 = m0x * sig12 + (a1 * b1 - a2 * b2);
            }
        } else if Self::REDUCED_LENGTH | Self::GEODESIC_SCALE {
            j12 = m0x * sig12 + weights.equation_40(eps, ssig1, csig1, ssig2, csig2, a1, a2);
        } 

        if Self::REDUCED_LENGTH {
            s.set_m0(m0x);
            s.set_m12b(dn2 * (csig1 * ssig2) - dn1 * (ssig1 * csig2) - csig1 * csig2 * j12);
        }

        if Self::GEODESIC_SCALE {
            let csig12 = csig1 * csig2 + ssig1 * ssig2;
            let t = ep2 * (cbet1 - cbet2) * (cbet1 + cbet2) / (dn1 + dn2);
            s.set_m12(csig12 + (t * ssig2 - csig2 * j12) * ssig1 / dn1);
            s.set_m21(csig12 - (t * ssig1 - csig1 * j12) * ssig2 / dn2);
        } 
        s
    }
}

#[derive(Copy,Clone,Default)]
pub struct Distance {
    s12b: f64,
}
impl LengthsReturnValue for Distance {
    const DISTANCE: bool = true;

    #[inline(always)]
    fn set_s12b(&mut self, s12b: f64) {
        self.s12b = s12b;
    }

    #[inline(always)]
    fn get_s12b(&self) -> f64 { 
        self.s12b.clone()
    }
}
#[derive(Copy,Clone,Default)]
pub struct ReducedLengths {
    m12b: f64,
    m0: f64,
}
impl LengthsReturnValue for ReducedLengths {
    const REDUCED_LENGTH: bool = true;

    fn set_m12b(&mut self, m12b: f64) { self.m12b = m12b; }
    fn set_m0(&mut self, m0: f64) { self.m0 = m0 }

    fn get_m12b(&self) -> f64 { self.m12b }
    fn get_m0(&self) -> f64 { self.m0 }
}

#[derive(Copy,Clone,Default)]
pub struct LengthsPlusDistance {
    s12b: f64,
    m12b: f64,
    m0: f64,
}
impl LengthsReturnValue for LengthsPlusDistance {
    const DISTANCE: bool = true;
    const REDUCED_LENGTH: bool = true;

    fn set_s12b(&mut self, s12b: f64) { self.s12b = s12b; }
    fn set_m12b(&mut self, m12b: f64) { self.m12b = m12b; }
    fn set_m0(&mut self, m0: f64) { self.m0 = m0 }

    fn get_s12b(&self) -> f64 { self.s12b }
    fn get_m12b(&self) -> f64 { self.m12b }
    fn get_m0(&self) -> f64 { self.m0 }
}

#[derive(Copy,Clone,Default)]
pub struct None {
    _sized: usize,
}
impl LengthsReturnValue for None { }

#[derive(Copy,Clone,Default)]
pub struct All {
    s12b: f64,
    m12b: f64,
    m0: f64,
    m12: f64,
    m21: f64,
}
impl LengthsReturnValue for All {
    const DISTANCE: bool = true;
    const REDUCED_LENGTH: bool = true;
    const GEODESIC_SCALE: bool = true;

    fn set_s12b(&mut self, s12b: f64) { self.s12b = s12b; }
    fn set_m12b(&mut self, m12b: f64) { self.m12b = m12b; }
    fn set_m0(&mut self, m0: f64) { self.m0 = m0; }
    fn set_m12(&mut self, m12: f64) { self.m12 = m12; }
    fn set_m21(&mut self, m21: f64) { self.m21 = m21; }

    fn get_s12b(&self) -> f64 { self.s12b }
    fn get_m12b(&self) -> f64 { self.m12b }
    fn get_m0(&self) -> f64 { self.m0 }
    fn get_m12(&self) -> f64 { self.m21 }
    fn get_m21(&self) -> f64 { self.m21 }
}
