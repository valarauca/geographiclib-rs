
use crate::{
    geomath::{
        _A1m1f,
        _A2m1f,
        polyval,
    },
    internals::{
        constants::{GEODESIC_ORDER},
        subarray::{SubArray},
        utils::{constant_polyval},
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
}
impl Weights {
    pub (in crate) fn new(third_flattening: f64) -> Self {

        let a1_fixed = _A1m1f(third_flattening, GEODESIC_ORDER);
        let a2_fixed = _A2m1f(third_flattening, GEODESIC_ORDER);

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
        }
    }

    // equation 24
    pub (in crate) fn a3f(&self, epsilon: f64) -> f64 {
        constant_polyval::<{GEODESIC_ORDER-1},GEODESIC_ORDER>(&self.a3x, epsilon)
    }

    pub (in crate) fn c3f(&self, epsilon: f64) -> [f64;GEODESIC_ORDER] {
        let c3x: &[f64;nC3x] = &self.c3x;
        let mut c = [0.0_f64; GEODESIC_ORDER];
        let mut m = 1.0_f64;
        m *= epsilon;
        c[1] = m * constant_polyval::<4,  nC3x      >(&c3x, epsilon);
        m *= epsilon;
        c[2] = m * constant_polyval::<3, {nC3x -  5}>(&c3x[SubArray::<{nC3x -  5},  5>], epsilon);
        m *= epsilon;
        c[3] = m * constant_polyval::<2, {nC3x -  9}>(&c3x[SubArray::<{nC3x -  9},  9>], epsilon);
        m *= epsilon;
        c[4] = m * constant_polyval::<1, {nC3x - 12}>(&c3x[SubArray::<{nC3x - 12}, 12>], epsilon);
        m *= epsilon;
        c[5] = m * constant_polyval::<0, {nC3x - 14}>(&c3x[SubArray::<{nC3x - 14}, 14>], epsilon);
        c
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


    pub (in crate) fn c4f(&self, epsilon: f64) -> [f64;GEODESIC_ORDER] {
        let c4x: &[f64;nC4x] = &self.c4x;
        let mut c = [0.0_f64; GEODESIC_ORDER];
        let mut m = 1.0_f64;
        c[0] = m * constant_polyval::<5,   nC4x       >(&c4x, epsilon);
        m *= epsilon;
        c[1] = m * constant_polyval::<4, { nC4x -  6 }>(&c4x[SubArray::<{nC4x -  6},  6>], epsilon);
        m *= epsilon;
        c[2] = m * constant_polyval::<3, { nC4x - 11 }>(&c4x[SubArray::<{nC4x - 11}, 11>], epsilon);
        m *= epsilon;
        c[3] = m * constant_polyval::<2, { nC4x - 15 }>(&c4x[SubArray::<{nC4x - 15}, 15>], epsilon);
        m *= epsilon;
        c[4] = m * constant_polyval::<1, { nC4x - 18 }>(&c4x[SubArray::<{nC4x - 18}, 18>], epsilon);
        m *= epsilon;
        c[5] = m * constant_polyval::<0, { nC4x - 20 }>(&c4x[SubArray::<{nC4x - 20}, 20>], epsilon);
        c
    }

    pub (in crate) fn get_c3x<'a>(&'a self) -> &'a [f64; nC3x] {
        &self.c3x
    }

    pub (in crate) fn get_c4x<'a>(&'a self) -> &'a [f64; nC4x] {
        &self.c4x
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

    pub (in crate) fn get_a1m1f(&self, epsilon: f64) -> f64 {
        if epsilon == self.third_flattening {
            self.a1_fixed.clone()
        } else {
            _A1m1f(epsilon, GEODESIC_ORDER)
        }
    }

    pub (in crate) fn get_a2m1f(&self, epsilon: f64) -> f64 {
        if epsilon == self.third_flattening {
            self.a2_fixed.clone()
        } else {
            _A2m1f(epsilon, GEODESIC_ORDER)
        }
    }
}
