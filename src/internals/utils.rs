

// Evaluate a polynomial
#[inline(always)]
pub (in crate) fn constant_polyval<const N: usize, const S: usize>(p: &[f64;S], x: f64) -> f64 {
    let mut y: f64 = p[0];
    for val in &p[1..=N] {
        y = y * x + val;
    }
    y
}

#[inline(always)]
pub (in crate) fn sum_fourier_fast(epsilon: f64, params: &[f64;18]) -> [f64;7] {
    use crate::internals::subarray::{SubArray};

    let epsilon2: f64 = epsilon  * epsilon;
    let epsilon3: f64 = epsilon2 * epsilon;
    let epsilon4: f64 = epsilon3 * epsilon;
    let epsilon5: f64 = epsilon4 * epsilon;
    let epsilon6: f64 = epsilon5 * epsilon;
    [
        0.0_f64,
        epsilon  * constant_polyval::<2, {18 -  0}>(&params,                            epsilon2) / params[ 3],
        epsilon2 * constant_polyval::<2, {18 -  4}>(&params[SubArray::<{18 - 4},   4>], epsilon2) / params[ 7],
        epsilon3 * constant_polyval::<1, {18 -  8}>(&params[SubArray::<{18 - 8},   8>], epsilon2) / params[10],
        epsilon4 * constant_polyval::<1, {18 - 11}>(&params[SubArray::<{18 - 11}, 11>], epsilon2) / params[13],
        epsilon5 * constant_polyval::<0, {18 - 14}>(&params[SubArray::<{18 - 14}, 14>], epsilon2) / params[15],
        epsilon6 * constant_polyval::<0, {18 - 16}>(&params[SubArray::<{18 - 16}, 16>], epsilon2) / params[17],
    ]
}
