#![allow(non_snake_case)]
#![allow(clippy::excessive_precision)]

use crate::internals::utils::{constant_polyval};

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
    // todo: replace this
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

pub fn _A2m1f(eps: f64) -> f64 {
    const COEFF: [f64; 5] = [-11.0, -28.0, -192.0, 0.0, 256.0];
    let t: f64 = constant_polyval::<3,5>(&COEFF, eps.powi(2)) / COEFF[4];
    (t - eps) / (1.0 + eps)
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
    fn test__A2m1f() {
        assert_eq!(_A2m1f(0.12), -0.11680607884285714);
    }

    #[test]
    fn test__A1m1f() {
        assert_eq!(_A1m1f(0.12), 0.1404582405272727);
    }

    #[test]
    fn test_astroid() {
        assert_eq!(astroid(21.0, 12.0), 23.44475767500982);
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
