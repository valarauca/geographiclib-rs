#![allow(non_snake_case)]
#![allow(clippy::excessive_precision)]

use crate::internals::utils::{sum_fourier_fast,constant_polyval};
use crate::internals::subarray::SubArray;
use crate::internals::constants::{C1F_COEFF,C2F_COEFF,C1PF_COEFF,COEFF_SIZE};

// Normalize a two-vector
pub fn norm(x: &mut f64, y: &mut f64) {
    let r = x.hypot(*y);
    *x /= r;
    *y /= r;
}

// Error free transformation of a sum
pub fn sum(u: f64, v: f64) -> (f64, f64) {
    let s = u + v;
    let up = s - v;
    let vpp = s - up;
    let up = up - u;
    let vpp = vpp - v;
    let t = -(up + vpp);
    (s, t)
}

// Evaluate a polynomial
pub fn polyval(n: usize, p: &[f64], x: f64) -> f64 {
    let mut y = p[0];
    for val in &p[1..=n] {
        y = y * x + val;
    }
    y
}

// Round an angle so that small values underflow to 0
pub fn ang_round(x: f64) -> f64 {
    // The makes the smallest gap in x = 1/16 - nextafter(1/16, 0) = 1/2^57
    // for reals = 0.7 pm on the earth if x is an angle in degrees.  (This
    // is about 1000 times more resolution than we get with angles around 90
    // degrees.)  We use this to avoid having to deal with near singular
    // cases when x is non-zero but tiny (e.g., 1.0e-200).
    let z = 1.0 / 16.0;
    let mut y = x.abs();
    // The compiler mustn't "simplify" z - (z - y) to y
    if y < z {
        y = z - (z - y);
    };
    if x == 0.0 {
        0.0
    } else if x < 0.0 {
        -y
    } else {
        y
    }
}

/// remainder of x/y in the range [-y/2, y/2]
fn remainder(x: f64, y: f64) -> f64 {
    // z = math.fmod(x, y) if Math.isfinite(x) else Math.nan
    let z = if x.is_finite() { x % y } else { f64::NAN };

    // # On Windows 32-bit with python 2.7, math.fmod(-0.0, 360) = +0.0
    // # This fixes this bug.  See also Math::AngNormalize in the C++ library.
    // # sincosd has a similar fix.
    // z = x if x == 0 else z
    let z = if x == 0.0 { x } else { z };

    // return (z + y if z < -y/2 else
    // (z if z < y/2 else z -y))
    if z < -y / 2.0 {
        z + y
    } else if z < y / 2.0 {
        z
    } else {
        z - y
    }
}

/// reduce angle to (-180,180]
pub fn ang_normalize(x: f64) -> f64 {
    // y = Math.remainder(x, 360)
    // return 180 if y == -180 else y
    let y = remainder(x, 360.0);
    if y == -180.0 {
        180.0
    } else {
        y
    }
}

// Replace angles outside [-90,90] with NaN
pub fn lat_fix(x: f64) -> f64 {
    if x.abs() > 90.0 {
        f64::NAN
    } else {
        x
    }
}

// compute y - x and reduce to [-180,180] accurately
pub fn ang_diff(x: f64, y: f64) -> (f64, f64) {
    let (d, t) = sum(ang_normalize(-x), ang_normalize(y));
    let d = ang_normalize(d);
    if d == 180.0 && t > 0.0 {
        sum(-180.0, t)
    } else {
        sum(d, t)
    }
}

/// Compute sine and cosine of x in degrees
pub fn sincosd(x: f64) -> (f64, f64) {
    let (mut r, q) = libm::remquo(x, 90.0);

    r = r.to_radians();

    let (mut sinx, mut cosx) = r.sin_cos();

    (sinx, cosx) = match q as u32 & 3 {
        0 => (sinx, cosx),
        1 => (cosx, -sinx),
        2 => (-sinx, -cosx),
        3 => (-cosx, sinx),
        _ => unreachable!(),
    };

    // special values from F.10.1.12
    cosx += 0.0;

    // special values from F.10.1.13
    if sinx == 0.0 {
        sinx = sinx.copysign(x);
    }
    (sinx, cosx)
}

// Compute atan2(y, x) with result in degrees
pub fn atan2d(y: f64, x: f64) -> f64 {
    let mut x = x;
    let mut y = y;
    let mut q = if y.abs() > x.abs() {
        std::mem::swap(&mut x, &mut y);
        2.0
    } else {
        0.0
    };
    if x < 0.0 {
        q += 1.0;
        x = -x;
    }
    let mut ang = y.atan2(x).to_degrees();
    if q == 1.0 {
        ang = if y >= 0.0 { 180.0 - ang } else { -180.0 - ang };
    } else if q == 2.0 {
        ang = 90.0 - ang;
    } else if q == 3.0 {
        ang += -90.0;
    }
    ang
}

pub fn eatanhe(x: f64, es: f64) -> f64 {
    if es > 0.0 {
        es * (es * x).atanh()
    } else {
        -es * (es * x).atan()
    }
}

// Functions that used to be inside Geodesic
pub fn sin_cos_series(sinp: bool, sinx: f64, cosx: f64, c: &[f64]) -> f64 {
    let mut k = c.len();
    let mut n: i64 = k as i64 - if sinp { 1 } else { 0 };
    let ar: f64 = 2.0 * (cosx - sinx) * (cosx + sinx);
    let mut y1 = 0.0;
    let mut y0: f64 = if n & 1 != 0 {
        k -= 1;
        c[k]
    } else {
        0.0
    };
    n /= 2;
    while n > 0 {
        n -= 1;
        k -= 1;
        y1 = ar * y0 - y1 + c[k];
        k -= 1;
        y0 = ar * y1 - y0 + c[k];
    }
    if sinp {
        2.0 * sinx * cosx * y0
    } else {
        cosx * (y0 - y1)
    }
}

// Solve astroid equation
pub fn astroid(x: f64, y: f64) -> f64 {
    let p = x.powi(2);
    let q = y.powi(2);
    let r = (p + q - 1.0) / 6.0;
    if !(q == 0.0 && r <= 0.0) {
        let s = p * q / 4.0;
        let r2 = r.powi(2);
        let r3 = r * r2;
        let disc = s * (s + 2.0 * r3);
        let mut u = r;
        if disc >= 0.0 {
            let mut t3 = s + r3;
            t3 += disc.sqrt().copysign(t3);
            let t = t3.cbrt();
            u += t + if t != 0.0 { r2 / t } else { 0.0 };
        } else {
            let ang = (-disc).sqrt().atan2(-(s + r3));
            u += 2.0 * r * (ang / 3.0).cos();
        }
        let v = (u.powi(2) + q).sqrt();
        let uv = if u < 0.0 { q / (v - u) } else { u + v };
        let w = (uv - q) / (2.0 * v);
        uv / ((uv + w.powi(2)).sqrt() + w)
    } else {
        0.0
    }
}

pub fn _A1m1f(eps: f64) -> f64 {
    const COEFF: [f64; 5] = [1.0, 4.0, 64.0, 0.0, 256.0];
    let t: f64 = constant_polyval::<3,5>(&COEFF, eps.powi(2)) / COEFF[4];
    (t + eps) / (1.0 - eps)
}

pub fn _C1f(eps: f64, c: &mut [f64], geodesic_order: usize) {
    let out = fast_c1f(eps);
    for l in 1..=geodesic_order {
        c[l] = out[l];
    }
}

#[inline]
pub (in crate) fn fast_c1f(epsilon: f64) -> [f64;7] {
    sum_fourier_fast(epsilon,&C1F_COEFF)
}


pub fn _C1pf(eps: f64, c: &mut [f64], geodesic_order: usize) {
    let out = fast_c1pf(eps);
    for l in 1..=geodesic_order {
        c[l] = out[l];
    }
}

pub (in crate) fn fast_c1pf(epsilon: f64) -> [f64;7] {
    sum_fourier_fast(epsilon,&C1PF_COEFF)
}

pub fn _A2m1f(eps: f64) -> f64 {
    const COEFF: [f64; 5] = [-11.0, -28.0, -192.0, 0.0, 256.0];
    let t: f64 = constant_polyval::<3,5>(&COEFF, eps.powi(2)) / COEFF[4];
    (t - eps) / (1.0 + eps)
}

pub fn _C2f(eps: f64, c: &mut [f64], geodesic_order: usize) {
    let out = fast_c2f(eps);
    for l in 1..=geodesic_order {
        c[l] = out[l];
    }
}

#[inline]
pub (in crate) fn fast_c2f(epsilon: f64) -> [f64;7] {
    sum_fourier_fast(epsilon,&C2F_COEFF)
}

pub (in crate) fn difference_of_meridian_arc_lengths(
    epsilon: f64,
    sine_sigma_1: f64, cosine_sigma_1: f64,
    sine_sigma_2: f64, cosine_sigma_2: f64,
    coeff: &'static [f64;COEFF_SIZE],
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
    let epsilon6: f64 = epsilon5 * epsilon;

    // initialized these with zero
    let y1_0 = 0.0_f64;
    let y1_1 = 0.0_f64;
    let y2_0 = 0.0_f64;
    let y2_1 = 0.0_f64;

    // The generation of the Fourier expansion coefficients of B1/B2 (by functions C1f & C2f) are
    // generated inline & consummed immediately by the SinCosSeries function who's inner loop
    // is also unrolled here.
    //
    // See the comment at the head of this function for the entire rationale.
    let arr_6 = epsilon6 * constant_polyval::<0, {COEFF_SIZE - 16}>(&coeff[SubArray::<{COEFF_SIZE - 16}, 16>], epsilon2) / coeff[17];
    let y1_1 = seed1 * y1_0 - y1_1 + arr_6;
    let y2_1 = seed2 * y2_0 - y2_1 + arr_6;

    let arr_5 = epsilon5 * constant_polyval::<0, {COEFF_SIZE - 14}>(&coeff[SubArray::<{COEFF_SIZE - 14}, 14>], epsilon2) / coeff[15];
    let y1_0 = seed1 * y1_1 - y1_0 + arr_5;
    let y2_0 = seed2 * y2_1 - y2_0 + arr_5;

    let arr_4 = epsilon4 * constant_polyval::<1, {COEFF_SIZE - 11}>(&coeff[SubArray::<{COEFF_SIZE - 11}, 11>], epsilon2) / coeff[13];
    let y1_1 = seed1 * y1_0 - y1_1 + arr_4;
    let y2_1 = seed2 * y2_0 - y2_1 + arr_4;

    let arr_3 = epsilon3 * constant_polyval::<1, {COEFF_SIZE -  8}>(&coeff[SubArray::<{COEFF_SIZE - 8},   8>], epsilon2) / coeff[10];
    let y1_0 = seed1 * y1_1 - y1_0 + arr_3;
    let y2_0 = seed2 * y2_1 - y2_0 + arr_3;

    let arr_2 = epsilon2 * constant_polyval::<2, {COEFF_SIZE -  4}>(&coeff[SubArray::<{COEFF_SIZE - 4},   4>], epsilon2) / coeff[ 7];
    let y1_1 = seed1 * y1_0 - y1_1 + arr_2;
    let y2_1 = seed2 * y2_0 - y2_1 + arr_2;

    let arr_1 = epsilon  * constant_polyval::<2, {COEFF_SIZE -  0}>(&coeff,                                    epsilon2) / coeff[ 3];
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
pub (in crate) fn equation_40(
    epsilon: f64,
    sine_sigma_1: f64, cosine_sigma_1: f64,
    sine_sigma_2: f64, cosine_sigma_2: f64,
    a1: f64, a2: f64,
) -> f64 {
    // these values remain fixed for the entire calculation
    let seed1: f64 = 2.0_f64 * (cosine_sigma_1 - sine_sigma_1) * (cosine_sigma_1 + sine_sigma_1);
    let seed2: f64 = 2.0_f64 * (cosine_sigma_2 - sine_sigma_2) * (cosine_sigma_2 + sine_sigma_2);

    // precalc these
    let epsilon2: f64 = epsilon  * epsilon;
    let epsilon3: f64 = epsilon2 * epsilon;
    let epsilon4: f64 = epsilon3 * epsilon;
    let epsilon5: f64 = epsilon4 * epsilon;
    let epsilon6: f64 = epsilon5 * epsilon;

    // initialized these with zero
    let y1_0 = 0.0_f64;
    let y1_1 = 0.0_f64;
    let y2_0 = 0.0_f64;
    let y2_1 = 0.0_f64;

    // The generation of the Fourier expansion coefficients of B1/B2 (by functions C1f & C2f) are
    // generated inline & consummed immediately by the SinCosSeries function who's inner loop
    // is also unrolled here.
    //
    // See the comment at the head of this function for the entire rationale.
    let arr_6_1 = epsilon6 * constant_polyval::<0, {COEFF_SIZE - 16}>(&C1F_COEFF[SubArray::<{COEFF_SIZE - 16}, 16>], epsilon2) / C1F_COEFF[17];
    let arr_6_2 = epsilon6 * constant_polyval::<0, {COEFF_SIZE - 16}>(&C2F_COEFF[SubArray::<{COEFF_SIZE - 16}, 16>], epsilon2) / C2F_COEFF[17];
    let arr_6 = a1 * arr_6_1 - a2 * arr_6_2;
    let y1_1 = seed1 * y1_0 - y1_1 + arr_6;
    let y2_1 = seed2 * y2_0 - y2_1 + arr_6;

    let arr_5_1 = epsilon5 * constant_polyval::<0, {COEFF_SIZE - 14}>(&C1F_COEFF[SubArray::<{COEFF_SIZE - 14}, 14>], epsilon2) / C1F_COEFF[15];
    let arr_5_2 = epsilon5 * constant_polyval::<0, {COEFF_SIZE - 14}>(&C2F_COEFF[SubArray::<{COEFF_SIZE - 14}, 14>], epsilon2) / C2F_COEFF[15];
    let arr_5 = a1 * arr_5_1 - a2 * arr_5_2;
    let y1_0 = seed1 * y1_1 - y1_0 + arr_5;
    let y2_0 = seed2 * y2_1 - y2_0 + arr_5;

    let arr_4_1 = epsilon4 * constant_polyval::<1, {COEFF_SIZE - 11}>(&C1F_COEFF[SubArray::<{COEFF_SIZE - 11}, 11>], epsilon2) / C1F_COEFF[13];
    let arr_4_2 = epsilon4 * constant_polyval::<1, {COEFF_SIZE - 11}>(&C2F_COEFF[SubArray::<{COEFF_SIZE - 11}, 11>], epsilon2) / C2F_COEFF[13];
    let arr_4 = a1 * arr_4_1 - a2 * arr_4_2;
    let y1_1 = seed1 * y1_0 - y1_1 + arr_4;
    let y2_1 = seed2 * y2_0 - y2_1 + arr_4;

    let arr_3_1 = epsilon3 * constant_polyval::<1, {COEFF_SIZE -  8}>(&C1F_COEFF[SubArray::<{COEFF_SIZE - 8},   8>], epsilon2) / C1F_COEFF[10];
    let arr_3_2 = epsilon3 * constant_polyval::<1, {COEFF_SIZE -  8}>(&C2F_COEFF[SubArray::<{COEFF_SIZE - 8},   8>], epsilon2) / C2F_COEFF[10];
    let arr_3 = a1 * arr_3_1 - a2 * arr_3_2;
    let y1_0 = seed1 * y1_1 - y1_0 + arr_3;
    let y2_0 = seed2 * y2_1 - y2_0 + arr_3;

    let arr_2_1 = epsilon2 * constant_polyval::<2, {COEFF_SIZE -  4}>(&C1F_COEFF[SubArray::<{COEFF_SIZE - 4},   4>], epsilon2) / C1F_COEFF[ 7];
    let arr_2_2 = epsilon2 * constant_polyval::<2, {COEFF_SIZE -  4}>(&C2F_COEFF[SubArray::<{COEFF_SIZE - 4},   4>], epsilon2) / C2F_COEFF[ 7];
    let arr_2 = a1 * arr_2_1 - a2 * arr_2_2;
    let y1_1 = seed1 * y1_0 - y1_1 + arr_2;
    let y2_1 = seed2 * y2_0 - y2_1 + arr_2;

    let arr_1_1 = epsilon  * constant_polyval::<2, {COEFF_SIZE -  0}>(&C1F_COEFF,                                    epsilon2) / C1F_COEFF[ 3];
    let arr_1_2 = epsilon  * constant_polyval::<2, {COEFF_SIZE -  0}>(&C2F_COEFF,                                    epsilon2) / C2F_COEFF[ 3];
    let arr_1 = a1 * arr_1_1 - a2 * arr_1_2;
    let y1_0 = seed1 * y1_1 - y1_0 + arr_1;
    let y2_0 = seed2 * y2_1 - y2_0 + arr_1;

    let sine_series_1: f64 = 2.0 * sine_sigma_1 * cosine_sigma_1 * y1_0;
    let sine_series_2: f64 = 2.0 * sine_sigma_2 * cosine_sigma_2 * y2_0;

    sine_series_2 - sine_series_1
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    // Results for the assertions are taken by running the python implementation

    #[test]
    fn test_sincosd() {
        let res = sincosd(-77.03196);
        assert_relative_eq!(res.0, -0.9744953925159129);
        assert_relative_eq!(res.1, 0.22440750870961693);

        let res = sincosd(69.48894);
        assert_relative_eq!(res.0, 0.9366045700708676);
        assert_relative_eq!(res.1, 0.3503881837653281);
        let res = sincosd(-1.0);
        assert_relative_eq!(res.0, -0.01745240643728351);
        assert_relative_eq!(res.1, 0.9998476951563913);
    }

    #[test]
    fn test__C2f() {
        let mut c = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        _C2f(0.12, &mut c, 6);
        assert_eq!(
            c,
            vec![
                1.0,
                0.0601087776,
                0.00270653103,
                0.000180486,
                1.4215824e-05,
                1.22472e-06,
                1.12266e-07
            ]
        )
    }

    #[test]
    fn test__A2m1f() {
        assert_eq!(_A2m1f(0.12), -0.11680607884285714);
    }

    #[test]
    fn test__C1pf() {
        let mut c = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        _C1pf(0.12, &mut c, 6);
        assert_eq!(
            c,
            vec![
                1.0,
                0.059517321000000005,
                0.004421053215,
                0.0005074200000000001,
                6.997613759999999e-05,
                1.1233080000000001e-05,
                1.8507366e-06
            ]
        )
    }

    #[test]
    fn test__C1f() {
        let mut c = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        _C1f(0.12, &mut c, 6);
        assert_eq!(
            c,
            vec![
                1.0,
                -0.059676777599999994,
                -0.000893533122,
                -3.57084e-05,
                -2.007504e-06,
                -1.3607999999999999e-07,
                -1.0205999999999999e-08
            ]
        )
    }

    #[test]
    fn test__A1m1f() {
        assert_eq!(_A1m1f(0.12), 0.1404582405272727);
    }

    #[test]
    fn test_astroid() {
        assert_eq!(astroid(21.0, 12.0), 23.44475767500982);
    }

    #[test]
    fn test_sin_cos_series() {
        assert_eq!(
            sin_cos_series(
                false,
                -0.8928657853278468,
                0.45032287238256896,
                &[
                    0.6660771734724675,
                    1.5757752625233906e-05,
                    3.8461688963148916e-09,
                    1.3040960748120204e-12,
                    5.252912023008548e-16,
                    2.367770858285795e-19
                ],
            ),
            0.29993425660538664
        );

        assert_eq!(
            sin_cos_series(
                false,
                -0.8928657853278468,
                0.45032287238256896,
                &[0., 1., 2., 3., 4., 5.],
            ),
            1.8998562852254026
        );
        assert_eq!(
            sin_cos_series(
                true,
                0.2969032234925426,
                0.9549075745221299,
                &[
                    0.0,
                    -0.0003561309485314716,
                    -3.170731714689771e-08,
                    -7.527972480734327e-12,
                    -2.5133854116682488e-15,
                    -1.0025061462383107e-18,
                    -4.462794158625518e-22
                ],
            ),
            -0.00020196665516199853
        );
        assert_eq!(
            sin_cos_series(
                true,
                -0.8928657853278468,
                0.45032287238256896,
                &[
                    0.0,
                    -0.0003561309485314716,
                    -3.170731714689771e-08,
                    -7.527972480734327e-12,
                    -2.5133854116682488e-15,
                    -1.0025061462383107e-18,
                    -4.462794158625518e-22
                ],
            ),
            0.00028635444718997857
        );

        assert_eq!(
            sin_cos_series(true, 0.12, 0.21, &[1.0, 2.0]),
            0.10079999999999999
        );
        assert_eq!(
            sin_cos_series(
                true,
                -0.024679833885152578,
                0.9996954065111039,
                &[
                    0.0,
                    -0.0008355098973052918,
                    -1.7444619952659748e-07,
                    -7.286557795511902e-11,
                    -3.80472772706481e-14,
                    -2.2251271876594078e-17,
                    1.2789961247944744e-20
                ],
            ),
            4.124513511893872e-05
        );
    }

    // corresponding to tests/signtest.cpp
    mod sign_test {
        use super::*;
        fn is_equiv(x: f64, y: f64) -> bool {
            (x.is_nan() && y.is_nan()) || (x == y && x.is_sign_positive() == y.is_sign_positive())
        }

        macro_rules! check_sincosd {
            ($x: expr, $expected_sin: expr, $expected_cos: expr) => {
                let (sinx, cosx) = sincosd($x);
                assert!(
                    is_equiv(sinx, $expected_sin),
                    "sinx({}) = {}, but got {}",
                    $x,
                    $expected_sin,
                    sinx
                );
                assert!(
                    is_equiv(cosx, $expected_cos),
                    "cosx({}) = {}, but got {}",
                    $x,
                    $expected_cos,
                    cosx
                );
            };
        }

        #[test]
        fn sin_cosd() {
            check_sincosd!(f64::NEG_INFINITY, f64::NAN, f64::NAN);
            check_sincosd!(-810.0, -1.0, 0.0);
            check_sincosd!(-720.0, -0.0, 1.0);
            check_sincosd!(-630.0, 1.0, 0.0);
            check_sincosd!(-540.0, -0.0, -1.0);
            check_sincosd!(-450.0, -1.0, 0.0);
            check_sincosd!(-360.0, -0.0, 1.0);
            check_sincosd!(-270.0, 1.0, 0.0);
            check_sincosd!(-180.0, -0.0, -1.0);
            check_sincosd!(-90.0, -1.0, 0.0);
            check_sincosd!(-0.0, -0.0, 1.0);
            check_sincosd!(0.0, 0.0, 1.0);
            check_sincosd!(90.0, 1.0, 0.0);
            check_sincosd!(180.0, 0.0, -1.0);
            check_sincosd!(270.0, -1.0, 0.0);
            check_sincosd!(360.0, 0.0, 1.0);
            check_sincosd!(450.0, 1.0, 0.0);
            check_sincosd!(540.0, 0.0, -1.0);
            check_sincosd!(630.0, -1.0, 0.0);
            check_sincosd!(720.0, 0.0, 1.0);
            check_sincosd!(810.0, 1.0, 0.0);
            check_sincosd!(f64::INFINITY, f64::NAN, f64::NAN);
            check_sincosd!(f64::NAN, f64::NAN, f64::NAN);
        }
    }
}
