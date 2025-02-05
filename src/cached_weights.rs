
use crate::{
    geomath::{
        _A1m1f,
        _A2m1f,
        polyval,
    },
    internals::{
        constants::{GEODESIC_ORDER,C1F_COEFF,C2F_COEFF,C1PF_COEFF,COEFF_SIZE},
        subarray::{SubArray},
        utils::{constant_polyval,sum_fourier_fast},
    },
};

#[allow(non_upper_case_globals)]
const nA3x: usize = 6;
#[allow(non_upper_case_globals)]
const nC3x: usize = 15;
#[allow(non_upper_case_globals)]
const nC4x: usize = 21;

/// This structure stores a lot of expensive to calculate numeric constants.
///
/// There are threee classes of constants within this structure.
///
/// * `a3x`, `c3x`, and `c4x`: These constants are fixed value related to the spheroid
/// * `a1`, and `a2`: These are constants for fourier expansion (related to the
/// spheroid)
///
/// Why cache these constants?
///
/// 
#[derive(Clone,Debug)]
pub (in crate) struct Weights {
    pub (in crate) a3x: [f64; nA3x],
    pub (in crate) c3x: [f64; nC3x],
    pub (in crate) c4x: [f64; nC4x],
    third_flattening: f64,
    a1_fixed: f64,
    a2_fixed: f64,
    c1f_fixed: [f64;7],
    c2f_fixed: [f64;7],
}
impl Weights {
    pub (in crate) fn new(third_flattening: f64) -> Self {

        let a1_fixed = _A1m1f(third_flattening);
        let a2_fixed = _A2m1f(third_flattening);
        let c1f_fixed = sum_fourier_fast(third_flattening, &C1F_COEFF);
        let c2f_fixed = sum_fourier_fast(third_flattening, &C2F_COEFF);

        /*
         * a3x constant calculation
         *
         */
        const COEFF_A3: [f64; 18] = [
            -3.0, 128.0, -2.0, -3.0, 64.0, -1.0, -3.0, -1.0, 16.0, 3.0, -1.0, -2.0, 8.0, 1.0, -1.0, 2.0,
            1.0, 1.0,
        ];
        let mut a3x: [f64; nA3x] = [0.0_f64; nA3x];
        let mut o = 0usize;
        for (k,j) in (0..GEODESIC_ORDER).rev().enumerate() {
            let m = j.min(GEODESIC_ORDER - j - 1);
            a3x[k] = polyval(m, &COEFF_A3[o..], third_flattening)/COEFF_A3[o + m + 1];
            o += m + 2;
        }
    
        /*
         * c3x constant calculation
         *
         */
        const COEFF_C3: [f64; 45] = [
            3.0, 128.0, 2.0, 5.0, 128.0, -1.0, 3.0, 3.0, 64.0, -1.0, 0.0, 1.0, 8.0, -1.0, 1.0, 4.0, 5.0,
            256.0, 1.0, 3.0, 128.0, -3.0, -2.0, 3.0, 64.0, 1.0, -3.0, 2.0, 32.0, 7.0, 512.0, -10.0, 9.0,
            384.0, 5.0, -9.0, 5.0, 192.0, 7.0, 512.0, -14.0, 7.0, 512.0, 21.0, 2560.0,
        ];
        let mut c3x: [f64; nC3x] = [0.0_f64; nC3x];
        let mut o = 0usize;
        let mut k = 0usize;
        for l in 1..GEODESIC_ORDER {
            for j in (l..GEODESIC_ORDER).rev() {
                let m = j.min(GEODESIC_ORDER - j - 1);
                c3x[k] = polyval(m, &COEFF_C3[o..], third_flattening) / COEFF_C3[o + m + 1];
                k += 1;
                o += m + 2;
            }
        }

       
        /*
         * c4x constant calculation
         *
         */
        const COEFF_C4: [f64; 77] = [
            97.0, 15015.0, 1088.0, 156.0, 45045.0, -224.0, -4784.0, 1573.0, 45045.0, -10656.0, 14144.0,
            -4576.0, -858.0, 45045.0, 64.0, 624.0, -4576.0, 6864.0, -3003.0, 15015.0, 100.0, 208.0, 572.0,
            3432.0, -12012.0, 30030.0, 45045.0, 1.0, 9009.0, -2944.0, 468.0, 135135.0, 5792.0, 1040.0,
            -1287.0, 135135.0, 5952.0, -11648.0, 9152.0, -2574.0, 135135.0, -64.0, -624.0, 4576.0, -6864.0,
            3003.0, 135135.0, 8.0, 10725.0, 1856.0, -936.0, 225225.0, -8448.0, 4992.0, -1144.0, 225225.0,
            -1440.0, 4160.0, -4576.0, 1716.0, 225225.0, -136.0, 63063.0, 1024.0, -208.0, 105105.0, 3584.0,
            -3328.0, 1144.0, 315315.0, -128.0, 135135.0, -2560.0, 832.0, 405405.0, 128.0, 99099.0,
        ];
        let mut c4x = [0.0_f64; nC4x];
        let mut o = 0_usize;
        let mut k = 0_usize;
        for l in 0..GEODESIC_ORDER {
            for j in (l..GEODESIC_ORDER).rev() {
                let m = GEODESIC_ORDER - j - 1;
                c4x[k] = polyval(m, &COEFF_C4[o..], third_flattening) / COEFF_C4[o + m + 1];
                k += 1;
                o += m + 2;
            }
        }

        Self { 
            a3x,
            c3x,
            c4x,
            third_flattening,
            a1_fixed,
            a2_fixed,
            c1f_fixed,
            c2f_fixed,
        }
    }

    // equation 24
    pub (in crate) fn a3f(&self, epsilon: f64) -> f64 {
        constant_polyval::<{GEODESIC_ORDER-1},GEODESIC_ORDER>(&self.a3x, epsilon)
    }

    pub (in crate) fn c4x_difference(
        &self,
        epsilon: f64,
        sine_sigma_1: f64, cosine_sigma_1: f64,
        sine_sigma_2: f64, cosine_sigma_2: f64
    ) -> f64 {
        let seed_1: f64 = 2.0 * (cosine_sigma_1 - sine_sigma_1) * (cosine_sigma_1 + sine_sigma_1);
        let seed_2: f64 = 2.0 * (cosine_sigma_2 - sine_sigma_2) * (cosine_sigma_2 + sine_sigma_2);

        let epsilon2: f64 = epsilon  * epsilon;
        let epsilon3: f64 = epsilon2 * epsilon;
        let epsilon4: f64 = epsilon3 * epsilon;
        let epsilon5: f64 = epsilon4 * epsilon;
        
        let y1_1 = 0.0;
        let y0_1 = 0.0;
        let y1_2 = 0.0;
        let y0_2 = 0.0;

        let arr_5 = epsilon5 * constant_polyval::<0, { nC4x - 20 }>(&self.c4x[SubArray::<{nC4x - 20}, 20>], epsilon);
        let y1_1 = seed_1 * y0_1 - y1_1 + arr_5;
        let y1_2 = seed_2 * y0_2 - y1_2 + arr_5;

        let arr_4 = epsilon4 * constant_polyval::<1, { nC4x - 18 }>(&self.c4x[SubArray::<{nC4x - 18}, 18>], epsilon);
        let y0_1 = seed_1 * y1_1 - y0_1 + arr_4;
        let y0_2 = seed_2 * y1_2 - y0_2 + arr_4;

        let arr_3 = epsilon3 * constant_polyval::<2, { nC4x - 15 }>(&self.c4x[SubArray::<{nC4x - 15}, 15>], epsilon);
        let y1_1 = seed_1 * y0_1 - y1_1 + arr_3;
        let y1_2 = seed_2 * y0_2 - y1_2 + arr_3;

        let arr_2 = epsilon2 * constant_polyval::<3, { nC4x - 11 }>(&self.c4x[SubArray::<{nC4x - 11}, 11>], epsilon);
        let y0_1 = seed_1 * y1_1 - y0_1 + arr_2;
        let y0_2 = seed_2 * y1_2 - y0_2 + arr_2;

        let arr_1 = epsilon  * constant_polyval::<4, { nC4x -  6 }>(&self.c4x[SubArray::<{nC4x -  6},  6>], epsilon);
        let y1_1 = seed_1 * y0_1 - y1_1 + arr_1;
        let y1_2 = seed_2 * y0_2 - y1_2 + arr_1;

        let arr_0 = constant_polyval::<5,nC4x>(&self.c4x, epsilon);
        let y0_1 = seed_1 * y1_1 - y0_1 + arr_0;
        let y0_2 = seed_2 * y1_2 - y0_2 + arr_0;

        let val_1 = cosine_sigma_1 * (y0_1 - y1_1);
        let val_2 = cosine_sigma_2 * (y0_2 - y1_2);
        val_2 - val_1
    }

    // calculates the result of I₃(σ₂) - I₃(σ₁), where I₃(σ) is equation 25
    // see: 
    pub (in crate) fn c3x_difference_of_meridian_arc_lengths(
        &self,
        epsilon: f64,
        sine_sigma_1: f64, cosine_sigma_1: f64,
        sine_sigma_2: f64, cosine_sigma_2: f64
    ) -> f64 {
        // these values remain fixed for the entire calculation
        let seed1: f64 = 2.0_f64 * (cosine_sigma_1 - sine_sigma_1) * (cosine_sigma_1 + sine_sigma_1);
        let seed2: f64 = 2.0_f64 * (cosine_sigma_2 - sine_sigma_2) * (cosine_sigma_2 + sine_sigma_2);

        // Pre-calculate these values and place them on the stack
        // we will start with the highest power & work down.
        let epsilon2: f64 = epsilon  * epsilon;
        let epsilon3: f64 = epsilon2 * epsilon;
        let epsilon4: f64 = epsilon3 * epsilon;
        let epsilon5: f64 = epsilon4 * epsilon;

        // initialized these with zero
        let y1_1 = 0.0_f64;
        let y2_1 = 0.0_f64;

        // special case for 5 number of evaluations
        let arr_5 = epsilon5 * constant_polyval::<0, {nC3x - 14}>(&self.c3x[SubArray::<{nC3x - 14}, 14>], epsilon);
        let y1_0 = arr_5;
        let y2_0 = arr_5;

        let arr_4 = epsilon4 * constant_polyval::<1, {nC3x - 12}>(&self.c3x[SubArray::<{nC3x - 12}, 12>], epsilon);
        let y1_1 = seed1 * y1_0 - y1_1 + arr_4;
        let y2_1 = seed2 * y2_0 - y2_1 + arr_4;

        let arr_3 = epsilon3 * constant_polyval::<2, {nC3x -  9}>(&self.c3x[SubArray::<{nC3x -  9},  9>], epsilon);
        let y1_0 = seed1 * y1_1 - y1_0 + arr_3;
        let y2_0 = seed2 * y2_1 - y2_0 + arr_3;

        let arr_2 = epsilon2 * constant_polyval::<3, {nC3x -  5}>(&self.c3x[SubArray::<{nC3x -  5},  5>], epsilon);
        let y1_1 = seed1 * y1_0 - y1_1 + arr_2;
        let y2_1 = seed2 * y2_0 - y2_1 + arr_2;

        let arr_1 = epsilon  * constant_polyval::<4,  nC3x      >(&self.c3x, epsilon);
        let y1_0 = seed1 * y1_1 - y1_0 + arr_1;
        let y2_0 = seed2 * y2_1 - y2_0 + arr_1;

        let sine_series_1: f64 = 2.0 * sine_sigma_1 * cosine_sigma_1 * y1_0;
        let sine_series_2: f64 = 2.0 * sine_sigma_2 * cosine_sigma_2 * y2_0;

        sine_series_2 - sine_series_1
    }

    /*
     * Special Getters
     *
     * Internally the third flattening parameter (n) is used as an
     * initial guess for the expansion parameter (eps/epsilon) in
     * -most- calculations.
     *
     * As this value is constant (for our spheroid), we can calculate
     * these values at construction and skip a step for all future calculations.
     *
     * The only cost is an easy to predict branch (true first loop, false forever after)
     * within out hot loop.
     *
     */

    #[inline(always)]
    pub (in crate) fn get_a1m1f<W: WeightCaps>(&self, epsilon: f64) -> f64 {
        if W::CHECK_THIRD_FLATTENING && epsilon == self.third_flattening {
            self.a1_fixed.clone()
        } else {
            _A1m1f(epsilon)
        }
    }

    #[inline(always)]
    pub (in crate) fn get_a2m1f<W: WeightCaps>(&self, epsilon: f64) -> f64 {
        if W::CHECK_THIRD_FLATTENING && epsilon == self.third_flattening {
            self.a2_fixed.clone()
        } else {
            _A2m1f(epsilon)
        }
    }

    #[inline(always)]
    fn calc_bxf_idx<W: WeightCaps, C: Coeff, const IDX: usize>(&self, epsilon: f64) -> f64 {
        C::get_weight::<W,IDX>(epsilon, self.third_flattening, &self.c1f_fixed, &self.c2f_fixed)
    }

    #[inline(always)]
    pub (in crate) fn calc_single_bxf<W: WeightCaps, C: Coeff>(
        &self,
        epsilon: f64,
        sine_sigma_1: f64,
        cosine_sigma_1: f64,
    ) -> f64 {

        let seed1: f64 = 2.0_f64 * (cosine_sigma_1 - sine_sigma_1) * (cosine_sigma_1 + sine_sigma_1);
        let y1_0 = 0.0_f64;
        let y1_1 = 0.0_f64;

        let arr_6 = self.calc_bxf_idx::<W,C,6>(epsilon);
        let y1_1 = seed1 * y1_0 - y1_1 + arr_6;

        let arr_5 = self.calc_bxf_idx::<W,C,5>(epsilon);
        let y1_0 = seed1 * y1_1 - y1_0 + arr_5;

        let arr_4 = self.calc_bxf_idx::<W,C,4>(epsilon);
        let y1_1 = seed1 * y1_0 - y1_1 + arr_4;

        let arr_3 = self.calc_bxf_idx::<W,C,3>(epsilon);
        let y1_0 = seed1 * y1_1 - y1_0 + arr_3;

        let arr_2 = self.calc_bxf_idx::<W,C,2>(epsilon);
        let y1_1 = seed1 * y1_0 - y1_1 + arr_2;

        let arr_1 = self.calc_bxf_idx::<W,C,1>(epsilon);
        let y1_0 = seed1 * y1_1 - y1_0 + arr_1;

        2.0 * sine_sigma_1 * cosine_sigma_1 * y1_0
    }

    #[inline(always)]
    pub (in crate) fn calc_bxf<W: WeightCaps, C: Coeff>(
        &self, epsilon: f64,
        sine_sigma_1: f64, cosine_sigma_1: f64,
        sine_sigma_2: f64, cosine_sigma_2: f64,
    ) -> f64 {
        let seed1: f64 = 2.0_f64 * (cosine_sigma_1 - sine_sigma_1) * (cosine_sigma_1 + sine_sigma_1);
        let seed2: f64 = 2.0_f64 * (cosine_sigma_2 - sine_sigma_2) * (cosine_sigma_2 + sine_sigma_2);

        // initialized these with zero
        let y1_0 = 0.0_f64;
        let y1_1 = 0.0_f64;
        let y2_0 = 0.0_f64;
        let y2_1 = 0.0_f64;

        let arr_6 = self.calc_bxf_idx::<W,C,6>(epsilon);
        let y1_1 = seed1 * y1_0 - y1_1 + arr_6;
        let y2_1 = seed2 * y2_0 - y2_1 + arr_6;

        let arr_5 = self.calc_bxf_idx::<W,C,5>(epsilon);
        let y1_0 = seed1 * y1_1 - y1_0 + arr_5;
        let y2_0 = seed2 * y2_1 - y2_0 + arr_5;

        let arr_4 = self.calc_bxf_idx::<W,C,4>(epsilon);
        let y1_1 = seed1 * y1_0 - y1_1 + arr_4;
        let y2_1 = seed2 * y2_0 - y2_1 + arr_4;

        let arr_3 = self.calc_bxf_idx::<W,C,3>(epsilon);
        let y1_0 = seed1 * y1_1 - y1_0 + arr_3;
        let y2_0 = seed2 * y2_1 - y2_0 + arr_3;

        let arr_2 = self.calc_bxf_idx::<W,C,2>(epsilon);
        let y1_1 = seed1 * y1_0 - y1_1 + arr_2;
        let y2_1 = seed2 * y2_0 - y2_1 + arr_2;

        let arr_1 = self.calc_bxf_idx::<W,C,1>(epsilon);
        let y1_0 = seed1 * y1_1 - y1_0 + arr_1;
        let y2_0 = seed2 * y2_1 - y2_0 + arr_1;

        let sine_series_1: f64 = 2.0 * sine_sigma_1 * cosine_sigma_1 * y1_0;
        let sine_series_2: f64 = 2.0 * sine_sigma_2 * cosine_sigma_2 * y2_0;
        sine_series_2 - sine_series_1
    }

    /// calculates `J(σ1)-J(σ2)` or equation 40
    /// assumes the results of equation 42 are given to `a2`
    /// assumes the results of equation 17 are given to `a1`
    /// calculates the values of equation 18 & 43
    pub (in crate) fn equation_40<W: WeightCaps>(
        &self,
        epsilon: f64,
        sine_sigma_1: f64, cosine_sigma_1: f64,
        sine_sigma_2: f64, cosine_sigma_2: f64,
        a1: f64, a2: f64,
    ) -> f64 {
        // these values remain fixed for the entire calculation
        let seed1: f64 = 2.0_f64 * (cosine_sigma_1 - sine_sigma_1) * (cosine_sigma_1 + sine_sigma_1);
        let seed2: f64 = 2.0_f64 * (cosine_sigma_2 - sine_sigma_2) * (cosine_sigma_2 + sine_sigma_2);

        // initialized these with zero
        let j1_0 = 0.0_f64;
        let j1_1 = 0.0_f64;
        let j2_0 = 0.0_f64;
        let j2_1 = 0.0_f64;

        let arr_6 = a1 * self.calc_bxf_idx::<W,C1Coeff,6>(epsilon) - a2 * self.calc_bxf_idx::<W,C2Coeff,6>(epsilon);
        let j1_1 = seed1 * j1_0 - j1_1 + arr_6;
        let j2_1 = seed2 * j2_0 - j2_1 + arr_6;
        let arr_5 = a1 * self.calc_bxf_idx::<W,C1Coeff,5>(epsilon) - a2 * self.calc_bxf_idx::<W,C2Coeff,5>(epsilon);
        let j1_0 = seed1 * j1_1 - j1_0 + arr_5;
        let j2_0 = seed2 * j2_1 - j2_0 + arr_5;
        let arr_4 = a1 * self.calc_bxf_idx::<W,C1Coeff,4>(epsilon) - a2 * self.calc_bxf_idx::<W,C2Coeff,4>(epsilon);
        let j1_1 = seed1 * j1_0 - j1_1 + arr_4;
        let j2_1 = seed2 * j2_0 - j2_1 + arr_4;
        let arr_3 = a1 * self.calc_bxf_idx::<W,C1Coeff,3>(epsilon) - a2 * self.calc_bxf_idx::<W,C2Coeff,3>(epsilon);
        let j1_0 = seed1 * j1_1 - j1_0 + arr_3;
        let j2_0 = seed2 * j2_1 - j2_0 + arr_3;
        let arr_2 = a1 * self.calc_bxf_idx::<W,C1Coeff,2>(epsilon) - a2 * self.calc_bxf_idx::<W,C2Coeff,2>(epsilon);
        let j1_1 = seed1 * j1_0 - j1_1 + arr_2;
        let j2_1 = seed2 * j2_0 - j2_1 + arr_2;
        let arr_1 = a1 * self.calc_bxf_idx::<W,C1Coeff,1>(epsilon) - a2 * self.calc_bxf_idx::<W,C2Coeff,1>(epsilon);
        let j1_0 = seed1 * j1_1 - j1_0 + arr_1;
        let j2_0 = seed2 * j2_1 - j2_0 + arr_1;
    
        let sine_series_1: f64 = 2.0 * sine_sigma_1 * cosine_sigma_1 * j1_0;
        let sine_series_2: f64 = 2.0 * sine_sigma_2 * cosine_sigma_2 * j2_0;
    
        sine_series_2 - sine_series_1
    }
}

pub (in crate) trait WeightCaps {
    const C1: bool = false;
    const C1P: bool = false;
    const C2: bool = false;
    const C3: bool = false;
    const C4: bool = false;
    const CHECK_THIRD_FLATTENING: bool = false;
}

pub trait Coeff {
    const IS_C1: bool;
    const IS_C2: bool;
    const IS_C1P: bool;
    const DATA: [f64; 18];

    #[inline(always)]
    fn get_weight<W: WeightCaps, const IDX: usize>(
        epsilon: f64,
        third_flattening: f64,
        c1f_fixed: &[f64;7],
        c2f_fixed: &[f64;7],
    ) -> f64 {

        if IDX >= 7 || IDX == 0 {
            panic!("invalid range");
        }

        // should we not calculate this value?
        if ! ( ( W::C1 && Self::IS_C1 ) || ( W::C2 && Self::IS_C2 ) || ( W::C1P && Self::IS_C1P ) ) {
            return f64::NAN;
        }

        if W::CHECK_THIRD_FLATTENING && ( ! (W::C1P && Self::IS_C1P ) ) && epsilon == third_flattening {
            let arm: &[f64;7] = {
                if W::C1 && Self::IS_C1 {
                    c1f_fixed
                } else if W::C2 && Self::IS_C2 {
                    c2f_fixed
                } else {
                    return f64::NAN;
                }
            };
            return arm[IDX];
        };


        let epsilon2 = epsilon.powi(2);

        if IDX == 6 {
            epsilon.powi(6) * constant_polyval::<0, {COEFF_SIZE - 16}>(&Self::DATA[SubArray::<{COEFF_SIZE - 16}, 16>], epsilon2) / Self::DATA[17]
        } else if IDX == 5 {
            epsilon.powi(5) * constant_polyval::<0, {COEFF_SIZE - 14}>(&Self::DATA[SubArray::<{COEFF_SIZE - 14}, 14>], epsilon2) / Self::DATA[15]
        } else if IDX == 4 {
            epsilon.powi(4) * constant_polyval::<1, {COEFF_SIZE - 11}>(&Self::DATA[SubArray::<{COEFF_SIZE - 11}, 11>], epsilon2) / Self::DATA[13]
        } else if IDX == 3 {
            epsilon.powi(3) * constant_polyval::<1, {COEFF_SIZE -  8}>(&Self::DATA[SubArray::<{COEFF_SIZE - 8},   8>], epsilon2) / Self::DATA[10]
        } else if IDX == 2 {
            epsilon.powi(2) * constant_polyval::<2, {COEFF_SIZE -  4}>(&Self::DATA[SubArray::<{COEFF_SIZE - 4},   4>], epsilon2) / Self::DATA[ 7]
        } else if IDX == 1 {
            epsilon         * constant_polyval::<2, {COEFF_SIZE -  0}>(&Self::DATA,                                    epsilon2) / Self::DATA[ 3]
        } else {
            f64::NAN
        }
    }
}

pub struct C1Coeff;
impl Coeff for C1Coeff {
    const IS_C1: bool = true;
    const IS_C2: bool = false;
    const IS_C1P: bool = false;
    const DATA: [f64;18] = C1F_COEFF;
}

pub struct C2Coeff;
impl Coeff for C2Coeff {
    const IS_C1: bool = false;
    const IS_C2: bool = true;
    const IS_C1P: bool = false;
    const DATA: [f64;18] = C2F_COEFF;
}

pub struct C1pCoeff;
impl Coeff for C1pCoeff {
    const IS_C1: bool = false;
    const IS_C2: bool = false;
    const IS_C1P: bool = true;
    const DATA: [f64;18] = C1PF_COEFF;
}

