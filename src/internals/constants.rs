
/// Basically the highest order of polynomial approxmiations.
/// Common limit of loops.
pub (in crate) const GEODESIC_ORDER: usize = 6_usize;

pub (in crate) const WGS84_A: f64 = 6378137.0_f64;

/// Alternative style to improve accuracy (according to Karney's source code)
pub (in crate) const WGS84_F: f64 = 1.0_f64 / ( 298257223563.0_f64 / 1000000000.0_f64 );

/*
 * Series of constants used to define error conditions & limits of the approximation
 *
 */
pub (in crate) const TOL0: f64 = f64::EPSILON;
pub (in crate) const TOL1: f64 = TOL0 * 200.0_f64;
// TINY is the square root of `f64::MIN_POSITIVE` at time of writing rust doesn't consider
// f64::sqrt to be constant time, so a manual value is written here
pub (in crate) const TINY: f64 = 1.4916681462400413e-154;
#[test]
fn assert_tiny_constant_is_correct() {
    assert_eq!(f64::MIN_POSITIVE.sqrt(), TINY);
}
// TOL2 is the square root of TOL0, see comment on TINY for context
pub (in crate) const TOL2: f64 = 1.4901161193847656e-8;
#[test]
fn assert_tol2_constant_is_correct() {
    assert_eq!(TOL0.sqrt(), TOL2);
}
pub (in crate) const TOL_B: f64 = TOL0 * TOL2;
pub (in crate) const X_THRESH: f64 = TOL2 * 1000.0_f64;
pub (in crate) const ITERATIONS: usize = 20_usize;
pub (in crate) const MAX_ITERATIONS: usize = ITERATIONS + (f32::DIGITS as usize) + 10_usize;


/*
 * These constants have to do with merdian arc length calulation
 *
 */

/// Size of the coefficient array of fourier expansion for c1f/c2f/c3f;
pub (in crate) const COEFF_SIZE: usize = 18_usize;

/// Fourier constants for C1 equation
pub (in crate) const C1F_COEFF: [f64; COEFF_SIZE] = [
    // C1[1]/eps^1 
    -1.0, 6.0, -16.0,32.0, 
    // C2[2]/eps^2 
    -9.0, 64.0, -128.0, 2048.0,
    // C3[3]/eps^3 
     9.0, -16.0, 768.0,
    // C4[4]/eps^4 
     3.0, -5.0, 512.0,
    // C5[5]/eps^5 
     -7.0, 1280.0,
    // C6[5]/eps^5 
     -7.0, 2048.0,
];

/// Fourier constants for C2 equation
pub (in crate) const C2F_COEFF: [f64; COEFF_SIZE] = [
    // C2[1]/eps^1 
    1.0, 2.0, 16.0, 32.0,
    // C2[2]/eps^2
    35.0, 64.0, 384.0, 2048.0,
    // C2[3]/eps^3
    15.0, 80.0, 768.0,
    // C2[4]/eps^4
    7.0, 35.0, 512.0,
    // C2[5]/eps^5
    63.0, 1280.0,
    // C2[6]/eps^6
    77.0, 2048.0,
];

/// Fourier constants for C1PF equation
pub (in crate) const C1PF_COEFF: [f64; COEFF_SIZE] = [
    205.0, -432.0, 768.0, 1536.0,
    4005.0, -4736.0, 3840.0, 12288.0,
    -225.0, 116.0, 384.0,
    -7173.0, 2695.0, 7680.0,
    3467.0, 7680.0,
    38081.0, 61440.0,
];
