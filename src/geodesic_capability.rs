use crate::cached_weights::{Weights};
use crate::geomath;

pub const CAP_NONE: u64 = 0;
pub const CAP_C1: u64 = 1 << 0;
#[allow(non_upper_case_globals)]
pub const CAP_C1p: u64 = 1 << 1;
pub const CAP_C2: u64 = 1 << 2;
pub const CAP_C3: u64 = 1 << 3;
pub const CAP_C4: u64 = 1 << 4;
pub const CAP_ALL: u64 = 0x1F;
pub const CAP_MASK: u64 = CAP_ALL; //0111_1111_1000_0000
pub const OUT_ALL: u64 = 0x7F80;
// Includes LONG_UNROLL
pub const OUT_MASK: u64 = 0xFF80; // b1111_1111_1000_000
pub const EMPTY: u64 = 0;
pub const LATITUDE: u64 = 1 << 7 | CAP_NONE;        // b000_1000_0000
pub const LONGITUDE: u64 = 1 << 8 | CAP_C3;         // b001_0000_1000
pub const AZIMUTH: u64 = 1 << 9 | CAP_NONE;         // b010_0000_0000
pub const DISTANCE: u64 = 1 << 10 | CAP_C1;         // b100_0000_0001
pub const STANDARD: u64 = LATITUDE | LONGITUDE | AZIMUTH | DISTANCE;
pub const DISTANCE_IN: u64 = 1 << 11 | CAP_C1 | CAP_C1p;
pub const REDUCEDLENGTH: u64 = 1 << 12 | CAP_C1 | CAP_C2;
pub const GEODESICSCALE: u64 = 1 << 13 | CAP_C1 | CAP_C2;
pub const AREA: u64 = 1 << 14 | CAP_C4;
pub const LONG_UNROLL: u64 = 1 << 15;
// Does not include LONG_UNROLL
pub const ALL: u64 = OUT_ALL | CAP_ALL;

pub (in crate) trait Capabilities {
    const C1: bool = false;
    const C1p: bool = false;
    const C2: bool = false;
    const C3: bool = false;
    const C4: bool = false;

    const LATITUDE: bool = false;
    const LONGITUDE: bool = false;
    const AZIMUTH: bool = false;
    const DISTANCE: bool = false;
    const DISTANCE_IN: bool = false;
    const REDUCED_LENGTH: bool = false;
    const GEODESIC_SCALE: bool = false;
    const AREA: bool = false;

    /*
     * gen_inverse functions
     *
     */
    #[inline(always)]
    fn scale_update(sig12: f64, m12: &mut f64, m21: &mut f64 ) {
        if Self::GEODESIC_SCALE {
            *m12 = sig12.cos();
            *m21 = sig12.cos();
        }
    }

    #[inline(always)]
    fn scale_update2(sig12: f64, dnm: f64, m12: &mut f64, m21: &mut f64) {
        if Self::GEODESIC_SCALE {
            *m12 = (sig12 / dnm).cos();
            *m21 = (sig12 / dnm).cos();
        }
    }

    #[inline(always)]
    fn somg_update(domg12: f64, slam12: f64, clam12: f64, somg12: &mut f64, comg12: &mut f64) {
        if Self::AREA {
            let (sdomg12,cdomg12) = domg12.sin_cos();
            *somg12 = slam12 * cdomg12 - clam12 * sdomg12;
            *comg12 = clam12 * cdomg12 + slam12 * sdomg12;
        }
    }

    #[inline(always)]
    fn distance_update(s12x: f64, s12: &mut f64) {
        if Self::DISTANCE {
            *s12 = 0.0 + s12x;
        }
    }

    #[inline(always)]
    fn m12_update(m12x: f64, m12: &mut f64) {
        if Self::REDUCED_LENGTH {
            *m12 = 0.0 + m12x;
        }
    }

    // part of _gen_inverse_azi
    #[inline(always)]
    fn azimuth_generation(salp: f64, calp: f64, azi: &mut f64) {
        if Self::AZIMUTH {
            *azi = geomath::atan2d(salp,calp);
        }
    }

    /*
     * Reduced Length Functions
     *
     */
    #[inline(always)]
    fn calc_a1_a2_and_m0x(
        weights: &Weights,
        eps: f64,
        a1: &mut f64,
        a2: &mut f64,
        m0x: &mut f64,
    ) {
        if Self::DISTANCE | Self::REDUCED_LENGTH | Self::GEODESIC_SCALE {
            *a1 = weights.get_a1m1f(eps);
            if Self::REDUCED_LENGTH | Self::GEODESIC_SCALE {
                *a2 = weights.get_a2m1f(eps);
                *m0x = *a1 - *a2;
                *a2 += 1.0;
            }
            *a1 += 1.0;
        }
    }

    #[inline(always)]
    fn calc_j12_and_s12b(
        weights: &Weights,
        eps: f64,
        ssig1: f64,
        csig1: f64,
        ssig2: f64,
        csig2: f64,
        sig12: f64,
        m0x: f64,
        a1: f64,
        a2: f64,
        s12b: &mut f64,
        j12: &mut f64,
    ) {
        use crate::cached_weights::{C1fCoeff,C2fCoeff};
        if Self::DISTANCE {
            let b1 = weights.difference_of_meridian_arc_lengths::<C1fCoeff>(eps, ssig1, csig1, ssig2, csig2);
            *s12b = a1 * (sig12 + b1);
            if Self::REDUCED_LENGTH | Self::GEODESIC_SCALE {
                let b2 = weights.difference_of_meridian_arc_lengths::<C2fCoeff>(eps, ssig1, csig1, ssig2, csig2);
                *j12 = m0x * sig12 + (a1 * b1 - a2 * b2);
            }
        } else if Self::REDUCED_LENGTH | Self::GEODESIC_SCALE {
            *j12 = m0x * sig12 + weights.equation_40(eps, ssig1, csig1, ssig2, csig2, a1, a2);
        }
    }

    #[inline(always)]
    fn update_m0_and_m12b(
        ssig1: f64,
        csig1: f64,
        dn1: f64,
        ssig2: f64,
        csig2: f64,
        dn2: f64,
        j12: f64,
        m0x: f64,
        m0: &mut f64,
        m12b: &mut f64,
    ) {
        if Self::REDUCED_LENGTH {
            *m0 = m0x;
            *m12b = dn2 * (csig1 * ssig2) - dn1 * (ssig1 * csig2) - csig1 * csig2 * j12;
        }
    }

    #[inline(always)]
    fn update_m12_m21(
        ep2: f64,
        ssig1: f64,
        csig1: f64,
        dn1: f64,
        ssig2: f64,
        csig2: f64,
        dn2: f64,
        cbet1: f64,
        cbet2: f64,
        j12: f64,
        m12: &mut f64,
        m21: &mut f64,
    ) {
        if Self::GEODESIC_SCALE {
            let csig12 = csig1 * csig2 + ssig1 * ssig2;
            let t = ep2 * (cbet1 - cbet2) * (cbet1 + cbet2) / (dn1 + dn2);
            *m12 = csig12 + (t * ssig2 - csig2 * j12) * ssig1 / dn1;
            *m21 = csig12 - (t * ssig1 - csig1 * j12) * ssig2 / dn2;
        }
    }

    /*
     * geodesic_line
     *
     */
    fn case_1(
        eps: f64,
        c1a: &mut [f64],
        ssig1: f64,
        csig1: f64,
        a1m1: &mut f64,
        b11: &mut f64,
        stau1: &mut f64,
        ctau1: &mut f64,
    ) {
        if Self::C1 {
            *a1m1 = geomath::_A1m1f(eps);
            geomath::_C1f(eps, c1a, 6);
            *b11 = geomath::sin_cos_series(true, ssig1, csig1, c1a);
            let (s,c) = b11.sin_cos();
            *stau1 = ssig1 * c + csig1 * s;
            *ctau1 = csig1 * c - ssig1 * s;
        }
    }
}

// Used for a particular case in _gen_inverse
pub struct DistanceWrapper<C: Capabilities> {
    _marker: std::marker::PhantomData<C>
}
impl<C: Capabilities> Capabilities for DistanceWrapper<C> {
    const C1: bool = C::C1 | (C::REDUCED_LENGTH | C::GEODESIC_SCALE);
    const C1p: bool = C::C1p;
    const C2: bool = C::C2 | (C::REDUCED_LENGTH | C::GEODESIC_SCALE);
    const C3: bool = C::C3;
    const C4: bool = C::C4;

    const LATITUDE: bool = C::LATITUDE;
    const LONGITUDE: bool = C::LONGITUDE;
    const AZIMUTH: bool = C::AZIMUTH;
    const DISTANCE_IN: bool = C::DISTANCE_IN;
    const DISTANCE: bool = C::DISTANCE | (C::REDUCED_LENGTH | C::GEODESIC_SCALE);
    const REDUCED_LENGTH: bool = C::REDUCED_LENGTH;
    const GEODESIC_SCALE: bool = C::GEODESIC_SCALE;
    const AREA: bool = C::AREA;
}

pub struct DRLWrapper<C: Capabilities> {
    _marker: std::marker::PhantomData<C>
}
impl<C: Capabilities> Capabilities for DRLWrapper<C> {
    const C1: bool = true | C::C1;
    const C1p: bool = true | C::C1p;
    const C2: bool = true | C::C2;
    const C3: bool = C::C3;
    const C4: bool = C::C4;

    const LATITUDE: bool = C::LATITUDE;
    const LONGITUDE: bool = C::LONGITUDE;
    const AZIMUTH: bool = C::AZIMUTH;
    const DISTANCE_IN: bool = C::DISTANCE_IN;
    const DISTANCE: bool = true;
    const REDUCED_LENGTH: bool = true;
    const GEODESIC_SCALE: bool = C::GEODESIC_SCALE;
    const AREA: bool = C::AREA;
}


