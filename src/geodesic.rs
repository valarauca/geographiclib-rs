#![allow(non_snake_case)]
#![allow(clippy::excessive_precision)]

use crate::geodesic_capability as caps;
use crate::geodesic_line;
use crate::geomath;
use crate::traits::{Caps,Distance,Empty,ReducedLength,GeodesicScale,Area,Azimuth,CheckN};
use crate::traits::{LengthsReturnValue,M0,S12b,M21M12,M12b};
#[cfg(test)]use crate::traits::{Standard,All};
use crate::cached_weights::{Weights,C1Coeff,C2Coeff};
use crate::internals::constants::{TOL0,TOL1,TOL2,TINY,TOL_B,X_THRESH,ITERATIONS,MAX_ITERATIONS,WGS84_A,WGS84_F};
use std::sync;

use std::f64::consts::{FRAC_1_SQRT_2, PI};

#[derive(Clone, Debug)]
pub struct Geodesic {
    pub(in crate) a: f64,
    pub(in crate) f: f64,
    pub(in crate) _f1: f64,
    pub(in crate) _e2: f64,
    pub(in crate) _ep2: f64,
    _n: f64,
    pub(in crate) _b: f64,
    pub(in crate) _c2: f64,
    _etol2: f64,
    pub(in crate) weights: Box<Weights>,
}

static WGS84_GEOD: sync::OnceLock<Geodesic> = sync::OnceLock::new();

impl Geodesic {
    pub fn wgs84() -> &'static Self {
        WGS84_GEOD.get_or_init(|| Geodesic::new(WGS84_A, WGS84_F))
    }

    pub fn equatorial_radius(&self) -> f64 {
        self.a
    }

    pub fn flattening(&self) -> f64 {
        self.f
    }
}

impl Geodesic {
    pub fn new(a: f64, f: f64) -> Self {

        let _f1 = 1.0 - f;
        let _e2 = f * (2.0 - f);
        let _ep2 = _e2 / _f1.powi(2);
        let _n = f / (2.0 - f);
        let _b = a * _f1;
        let _c2 = (a.powi(2)
            + _b.powi(2)
                * (if _e2 == 0.0 {
                    1.0
                } else {
                    geomath::eatanhe(1.0, (if f < 0.0 { -1.0 } else { 1.0 }) * _e2.abs().sqrt())
                        / _e2
                }))
            / 2.0;
        let _etol2 = 0.1 * TOL2 / (f.abs().max(0.001) * (1.0 - f / 2.0).min(1.0) / 2.0).sqrt();

        let weights = Box::new(Weights::new(_n));

        Geodesic {
            a,
            f,
            _f1,
            _e2,
            _ep2,
            _n,
            _b,
            _c2,
            _etol2,
            weights, 
        }
    }

    pub(in crate) fn _A3f(&self, eps: f64) -> f64 {
        self.weights.a3f(eps)
    }

    pub (in crate) fn sincosd_for_ellipsoid(&self, ang: f64) -> (f64,f64) {
        let (mut sin,mut cos) = geomath::sincosd(ang);
        sin *= self._f1;
        geomath::norm(&mut sin, &mut cos);
        cos = TINY.max(cos);
        (sin,cos)
    }

    /// calculate k2 and epsilon for usage in power series to approximate
    /// the values of integration
    #[inline(always)]
    pub (in crate) fn local_curvature(&self, value: f64) -> (f64,f64) {
        let k2 = value.powi(2) * self._ep2;
        let eps = k2 / (2.0 * (1.0 + (1.0 + k2).sqrt()) + k2);
        (k2, eps)
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate) fn _Lengths<L: LengthsReturnValue, C: Caps>(
        &self,
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
    ) -> L {
        let mut ret: L = L::default();

        let mut A1 = 0.0;
        let mut A2 = 0.0;
        let mut m0x = 0.0;
        let mut J12 = 0.0;

        if C::DISTANCE | C::REDUCEDLENGTH | C::GEODESICSCALE {
            A1 = self.weights.get_a1m1f::<C>(eps);
            if C::REDUCEDLENGTH | C::GEODESICSCALE {
                A2 = self.weights.get_a2m1f::<C>(eps);
                m0x = A1 - A2;
                A2 += 1.0;
            }
            A1 += 1.0;
        }
        if C::DISTANCE {
            let B1 = self.weights.calc_bxf::<C,C1Coeff>(eps, ssig1, csig1, ssig2, csig2);
            ret.set_s12b(A1 * (sig12 + B1));
            if C::REDUCEDLENGTH | C::GEODESICSCALE {
                let B2 = self.weights.calc_bxf::<C,C2Coeff>(eps, ssig1, csig1, ssig2, csig2);
                J12 = m0x * sig12 + (A1 * B1 - A2 * B2);
            }
        } else if C::REDUCEDLENGTH | C::GEODESICSCALE {
            J12 = m0x * sig12 + self.weights.equation_40::<C>(eps, ssig1, csig1, ssig2, csig2, A1, A2);
        }
        if C::REDUCEDLENGTH {
            ret.set_m0(m0x);
            ret.set_m12b(dn2 * (csig1 * ssig2) - dn1 * (ssig1 * csig2) - csig1 * csig2 * J12);
        }
        if C::GEODESICSCALE {
            let csig12 = csig1 * csig2 + ssig1 * ssig2;
            let t = self._ep2 * (cbet1 - cbet2) * (cbet1 + cbet2) / (dn1 + dn2);
            ret.set_m12(csig12 + (t * ssig2 - csig2 * J12) * ssig1 / dn1);
            ret.set_m21(csig12 - (t * ssig1 - csig1 * J12) * ssig2 / dn2);
        }
        ret
    }

    #[allow(clippy::too_many_arguments)]
    pub fn _InverseStart(
        &self,
        sbet1: f64,
        cbet1: f64,
        dn1: f64,
        sbet2: f64,
        cbet2: f64,
        dn2: f64,
        lam12: f64,
        slam12: f64,
        clam12: f64,
    ) -> (f64, f64, f64, f64, f64, f64) {
        let mut sig12 = -1.0;
        let mut salp2 = f64::NAN;
        let mut calp2 = f64::NAN;
        let mut dnm = f64::NAN;

        /*
        let mut somg12: f64;
        let mut comg12: f64;
        */

        let sbet12 = sbet2 * cbet1 - cbet2 * sbet1;
        let cbet12 = cbet2 * cbet1 + sbet2 * sbet1;

        let mut sbet12a = sbet2 * cbet1;
        sbet12a += cbet2 * sbet1;

        let shortline = cbet12 >= 0.0 && sbet12 < 0.5 && cbet2 * lam12 < 0.5;
        let (mut somg12, mut comg12) = if shortline {
            // sbetm2 = sin^2(beta1+beta2/2)
            // the expression:
            //   let sbetm2 = 0.5 * ( 1.0 - ( ( cbet1 * cbet2 ) - (sbet1 * sbet2 )));
            // also passes all tests
            let mut sbetm2 = (sbet1 + sbet2).powi(2);
            sbetm2 /= sbetm2 + (cbet1 + cbet2).powi(2);
            dnm = (1.0 + self._ep2 * sbetm2).sqrt();
            let omg12 = lam12 / (self._f1 * dnm);
            omg12.sin_cos()
        } else {
            (slam12,clam12)
        };

        let mut salp1 = cbet2 * somg12;

        let temp = cbet2 * sbet1 * somg12.powi(2) / (1.0 + comg12.abs());
        let mut calp1 = if comg12 >= 0.0 {
            sbet12 + temp
        } else {
            sbet12a - temp
        };

        let ssig12 = salp1.hypot(calp1);
        let csig12 = sbet1 * sbet2 + cbet1 * cbet2 * comg12;

        if shortline && ssig12 < self._etol2 {
            salp2 = cbet1 * somg12;
            calp2 = sbet12
                - cbet1
                    * sbet2
                    * (if comg12 >= 0.0 {
                        somg12.powi(2) / (1.0 + comg12)
                    } else {
                        1.0 - comg12
                    });
            geomath::norm(&mut salp2, &mut calp2);
            sig12 = ssig12.atan2(csig12);
        } else if self._n.abs() > 0.1
            || csig12 >= 0.0
            || ssig12 >= 6.0 * self._n.abs() * PI * cbet1.powi(2)
        {
        } else {
            let x: f64;
            let y: f64;
            let betscale: f64;
            let lamscale: f64;
            let lam12x = (-slam12).atan2(-clam12);
            if self.f >= 0.0 {
                let (_,eps) = self.local_curvature(sbet1);
                lamscale = self.f * cbet1 * self._A3f(eps) * PI;
                betscale = lamscale * cbet1;
                x = lam12x / lamscale;
                y = sbet12a / betscale;
            } else {
                let cbet12a = cbet2 * cbet1 - sbet2 * sbet1;
                let bet12a = sbet12a.atan2(cbet12a);
                let res = self._Lengths::<M0<M12b>,CheckN<ReducedLength<Empty>>>(
                    self._n,
                    PI + bet12a,
                    sbet1,
                    -cbet1,
                    dn1,
                    sbet2,
                    cbet2,
                    dn2,
                    f64::NAN,
                    f64::NAN
                );
                let m12b = res.get_m12b();
                let m0 = res.get_m0();
                x = -1.0 + m12b / (cbet1 * cbet2 * m0 * PI);
                betscale = if x < -0.01 {
                    sbet12a / x
                } else {
                    -self.f * cbet1.powi(2) * PI
                };
                lamscale = betscale / cbet1;
                y = lam12x / lamscale;
            }
            if y > -TOL1 && x > -1.0 - X_THRESH {
                if self.f >= 0.0 {
                    salp1 = (-x).min(1.0);
                    calp1 = -(1.0 - salp1.powi(2)).sqrt()
                } else {
                    calp1 = x.max(if x > -TOL1 { 0.0 } else { -1.0 });
                    salp1 = (1.0 - calp1.powi(2)).sqrt();
                }
            } else {
                let k = geomath::astroid(x, y);
                let omg12a = lamscale
                    * if self.f >= 0.0 {
                        -x * k / (1.0 + k)
                    } else {
                        -y * (1.0 + k) / k
                    };
                somg12 = omg12a.sin();
                comg12 = -(omg12a.cos());
                salp1 = cbet2 * somg12;
                calp1 = sbet12a - cbet2 * sbet1 * somg12.powi(2) / (1.0 - comg12);
            }
        }

        if salp1 > 0.0 || salp1.is_nan() {
            geomath::norm(&mut salp1, &mut calp1);
        } else {
            salp1 = 1.0;
            calp1 = 0.0;
        };
        (sig12, salp1, calp1, salp2, calp2, dnm)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn _Lambda12(
        &self,
        sbet1: f64,
        cbet1: f64,
        dn1: f64,
        sbet2: f64,
        cbet2: f64,
        dn2: f64,
        salp1: f64,
        calp1: f64,
        slam120: f64,
        clam120: f64,
        diffp: bool,
    ) -> (f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64) {
        let calp1 = if sbet1 == 0.0 && calp1 == 0.0 {
            -TINY
        } else {
            calp1
        };
        let salp0 = salp1 * cbet1;
        let calp0 = calp1.hypot(salp1 * sbet1);

        let mut ssig1 = sbet1;
        let somg1 = salp0 * sbet1;
        let mut csig1 = calp1 * cbet1;
        let comg1 = calp1 * cbet1;
        geomath::norm(&mut ssig1, &mut csig1);

        let salp2 = if cbet2 != cbet1 { salp0 / cbet2 } else { salp1 };
        let calp2 = if cbet2 != cbet1 || sbet2.abs() != -sbet1 {
            ((calp1 * cbet1).powi(2)
                + if cbet1 < -sbet1 {
                    (cbet2 - cbet1) * (cbet1 + cbet2)
                } else {
                    (sbet1 - sbet2) * (sbet1 + sbet2)
                })
            .sqrt()
                / cbet2
        } else {
            calp1.abs()
        };
        let mut ssig2 = sbet2;
        let somg2 = salp0 * sbet2;
        let mut csig2 = calp2 * cbet2;
        let comg2 = calp2 * cbet2;
        geomath::norm(&mut ssig2, &mut csig2);

        let sig12 = ((csig1 * ssig2 - ssig1 * csig2).max(0.0)).atan2(csig1 * csig2 + ssig1 * ssig2);
        let somg12 = (comg1 * somg2 - somg1 * comg2).max(0.0);
        let comg12 = comg1 * comg2 + somg1 * somg2;
        let eta = (somg12 * clam120 - comg12 * slam120).atan2(comg12 * clam120 + somg12 * slam120);

        let (_,eps) = self.local_curvature(calp0);
        let B312 = self.weights.c3x_difference_of_meridian_arc_lengths(eps, ssig1, csig1, ssig2, csig2);
        let domg12 = -self.f * self._A3f(eps) * salp0 * (sig12 + B312);
        let lam12 = eta + domg12;

        let dlam12 = if diffp {
            if calp2 == 0.0 {
                -2.0 * self._f1 * dn1 / sbet1
            } else {
                let res = self._Lengths::<M12b, ReducedLength<Empty>>(
                    eps,
                    sig12,
                    ssig1,
                    csig1,
                    dn1,
                    ssig2,
                    csig2,
                    dn2,
                    f64::NAN,
                    f64::NAN,
                );
                let m12b = res.get_m12b();
                m12b * (self._f1 / (calp2 * cbet2))
            }
        } else {
            f64::NAN
        };
        (
            lam12, salp2, calp2, sig12, ssig1, csig1, ssig2, csig2, eps, domg12, dlam12,
        )
    }

    // returns (a12, s12, azi1, azi2, m12, M12, M21, S12)
    pub fn _gen_inverse_azi<C: Caps>(
        &self,
        lat1: f64,
        lon1: f64,
        lat2: f64,
        lon2: f64,
    ) -> (f64, f64, f64, f64, f64, f64, f64, f64) {
        let mut azi1 = f64::NAN;
        let mut azi2 = f64::NAN;

        let (a12, s12, salp1, calp1, salp2, calp2, m12, M12, M21, S12) =
            self._gen_inverse::<C>(lat1, lon1, lat2, lon2);
        if C::AZIMUTH {
            azi1 = geomath::atan2d(salp1, calp1);
            azi2 = geomath::atan2d(salp2, calp2);
        }
        (a12, s12, azi1, azi2, m12, M12, M21, S12)
    }

    // returns (a12, s12, salp1, calp1, salp2, calp2, m12, M12, M21, S12)
    pub fn _gen_inverse<C: Caps>(
        &self,
        lat1: f64,
        lon1: f64,
        lat2: f64,
        lon2: f64,
    ) -> (f64, f64, f64, f64, f64, f64, f64, f64, f64, f64) {
        let mut lat1 = lat1;
        let mut lat2 = lat2;
        let mut a12 = f64::NAN;
        let mut s12 = f64::NAN;
        let mut m12 = f64::NAN;
        let mut M12 = f64::NAN;
        let mut M21 = f64::NAN;
        let mut S12 = f64::NAN;

        let (mut lon12, mut lon12s) = geomath::ang_diff(lon1, lon2);
        let mut lonsign = if lon12 >= 0.0 { 1.0 } else { -1.0 };

        lon12 = lonsign * geomath::ang_round(lon12);
        lon12s = geomath::ang_round((180.0 - lon12) - lonsign * lon12s);
        let lam12 = lon12.to_radians();
        let slam12: f64;
        let mut clam12: f64;
        if lon12 > 90.0 {
            let res = geomath::sincosd(lon12s);
            slam12 = res.0;
            clam12 = res.1;
            clam12 = -clam12;
        } else {
            let res = geomath::sincosd(lon12);
            slam12 = res.0;
            clam12 = res.1;
        };
        lat1 = geomath::ang_round(geomath::lat_fix(lat1));
        lat2 = geomath::ang_round(geomath::lat_fix(lat2));

        let swapp = if lat1.abs() < lat2.abs() { -1.0 } else { 1.0 };
        if swapp < 0.0 {
            lonsign *= -1.0;
            std::mem::swap(&mut lat2, &mut lat1);
        }
        let latsign = if lat1 < 0.0 { 1.0 } else { -1.0 };
        lat1 *= latsign;
        lat2 *= latsign;

        let (sbet1, cbet1) = self.sincosd_for_ellipsoid(lat1);
        let (mut sbet2, mut cbet2) = self.sincosd_for_ellipsoid(lat2);

        if cbet1 < -sbet1 {
            if cbet2 == cbet1 {
                sbet2 = if sbet2 < 0.0 { sbet1 } else { -sbet1 };
            }
        } else if sbet2.abs() == -sbet1 {
            cbet2 = cbet1;
        }

        let dn1 = (1.0 + self._ep2 * sbet1.powi(2)).sqrt();
        let dn2 = (1.0 + self._ep2 * sbet2.powi(2)).sqrt();

        let mut meridian = lat1 == -90.0 || slam12 == 0.0;
        let mut calp1 = 0.0;
        let mut salp1 = 0.0;
        let mut calp2 = 0.0;
        let mut salp2 = 0.0;
        let mut ssig1 = 0.0;
        let mut csig1 = 0.0;
        let mut ssig2 = 0.0;
        let mut csig2 = 0.0;
        let mut sig12: f64;
        let mut s12x = 0.0;
        let mut m12x = 0.0;

        if meridian {
            calp1 = clam12;
            salp1 = slam12;
            calp2 = 1.0;
            salp2 = 0.0;

            ssig1 = sbet1;
            csig1 = calp1 * cbet1;
            ssig2 = sbet2;
            csig2 = calp2 * cbet2;

            sig12 = ((csig1 * ssig2 - ssig1 * csig2).max(0.0)).atan2(csig1 * csig2 + ssig1 * ssig2);
            let res = self._Lengths::<S12b<M21M12<M12b>>, CheckN<Distance<ReducedLength<C>>>>(
                self._n,
                sig12,
                ssig1,
                csig1,
                dn1,
                ssig2,
                csig2,
                dn2,
                cbet1,
                cbet2,
            );
            s12x = res.get_s12b();
            m12x = res.get_m12b();
            M12 = res.get_m12();
            M21 = res.get_m21();

            if sig12 < 1.0 || m12x >= 0.0 {
                if sig12 < 3.0 * TINY {
                    sig12 = 0.0;
                    m12x = 0.0;
                    s12x = 0.0;
                }
                m12x *= self._b;
                s12x *= self._b;
                a12 = sig12.to_degrees();
            } else {
                meridian = false;
            }
        }

        let mut somg12 = 2.0;
        let mut comg12 = 0.0;
        let mut omg12 = 0.0;
        let dnm: f64;
        let mut eps = 0.0;
        if !meridian && sbet1 == 0.0 && (self.f <= 0.0 || lon12s >= self.f * 180.0) {
            calp1 = 0.0;
            calp2 = 0.0;
            salp1 = 1.0;
            salp2 = 1.0;

            s12x = self.a * lam12;
            sig12 = lam12 / self._f1;
            omg12 = lam12 / self._f1;
            m12x = self._b * sig12.sin();
            if C::GEODESICSCALE {
                M12 = sig12.cos();
                M21 = sig12.cos();
            }
            a12 = lon12 / self._f1;
        } else if !meridian {
            let res = self._InverseStart(
                sbet1, cbet1, dn1, sbet2, cbet2, dn2, lam12, slam12, clam12,
            );
            sig12 = res.0;
            salp1 = res.1;
            calp1 = res.2;
            salp2 = res.3;
            calp2 = res.4;
            dnm = res.5;

            if sig12 >= 0.0 {
                s12x = sig12 * self._b * dnm;
                m12x = dnm.powi(2) * self._b * (sig12 / dnm).sin();
                if C::GEODESICSCALE {
                    M12 = (sig12 / dnm).cos();
                    M21 = (sig12 / dnm).cos();
                }
                a12 = sig12.to_degrees();
                omg12 = lam12 / (self._f1 * dnm);
            } else {
                let mut tripn = false;
                let mut tripb = false;
                let mut salp1a = TINY;
                let mut calp1a = 1.0;
                let mut salp1b = TINY;
                let mut calp1b = -1.0;
                let mut domg12 = 0.0;
                for numit in 0..MAX_ITERATIONS {
                    let res = self._Lambda12(
                        sbet1,
                        cbet1,
                        dn1,
                        sbet2,
                        cbet2,
                        dn2,
                        salp1,
                        calp1,
                        slam12,
                        clam12,
                        numit < ITERATIONS,
                    );
                    let v = res.0;
                    salp2 = res.1;
                    calp2 = res.2;
                    sig12 = res.3;
                    ssig1 = res.4;
                    csig1 = res.5;
                    ssig2 = res.6;
                    csig2 = res.7;
                    eps = res.8;
                    domg12 = res.9;
                    let dv = res.10;

                    if tripb
                        || v.abs() < if tripn { 8.0 } else { 1.0 } * TOL0
                        || v.abs().is_nan()
                    {
                        break;
                    };
                    if v > 0.0 && (numit > ITERATIONS || calp1 / salp1 > calp1b / salp1b) {
                        salp1b = salp1;
                        calp1b = calp1;
                    } else if v < 0.0 && (numit > ITERATIONS || calp1 / salp1 < calp1a / salp1a) {
                        salp1a = salp1;
                        calp1a = calp1;
                    }
                    if numit < ITERATIONS && dv > 0.0 {
                        let dalp1 = -v / dv;
                        let sdalp1 = dalp1.sin();
                        let cdalp1 = dalp1.cos();
                        let nsalp1 = salp1 * cdalp1 + calp1 * sdalp1;
                        if nsalp1 > 0.0 && dalp1.abs() < PI {
                            calp1 = calp1 * cdalp1 - salp1 * sdalp1;
                            salp1 = nsalp1;
                            geomath::norm(&mut salp1, &mut calp1);
                            tripn = v.abs() <= 16.0 * TOL0;
                            continue;
                        }
                    }

                    salp1 = (salp1a + salp1b) / 2.0;
                    calp1 = (calp1a + calp1b) / 2.0;
                    geomath::norm(&mut salp1, &mut calp1);
                    tripn = false;
                    tripb = (salp1a - salp1).abs() + (calp1a - calp1) < TOL_B
                        || (salp1 - salp1b).abs() + (calp1 - calp1b) < TOL_B;
                }
                let res = if C::REDUCEDLENGTH | C::GEODESICSCALE {
                    self._Lengths::<S12b<M21M12<M12b>>, Distance<C>>(eps, sig12, ssig1, csig1, dn1, ssig2, csig2, dn2, cbet1, cbet2)
                } else {
                    self._Lengths::<S12b<M21M12<M12b>>, C>(eps, sig12, ssig1, csig1, dn1, ssig2, csig2, dn2, cbet1, cbet2)
                };
                s12x = res.get_s12b();
                m12x = res.get_m12b();
                M12 = res.get_m12();
                M21 = res.get_m21();

                m12x *= self._b;
                s12x *= self._b;
                a12 = sig12.to_degrees();
                if C::AREA {
                    let sdomg12 = domg12.sin();
                    let cdomg12 = domg12.cos();
                    somg12 = slam12 * cdomg12 - clam12 * sdomg12;
                    comg12 = clam12 * cdomg12 + slam12 * sdomg12;
                }
            }
        }
        if C::DISTANCE {
            s12 = 0.0 + s12x;
        }
        if C::REDUCEDLENGTH {
            m12 = 0.0 + m12x;
        }
        if C::AREA {
            let salp0 = salp1 * cbet1;
            let calp0 = calp1.hypot(salp1 * sbet1);
            if calp0 != 0.0 && salp0 != 0.0 {
                ssig1 = sbet1;
                csig1 = calp1 * cbet1;
                ssig2 = sbet2;
                csig2 = calp2 * cbet2;
                (_,eps) = self.local_curvature(calp0);
                let A4 = self.a.powi(2) * calp0 * salp0 * self._e2;
                geomath::norm(&mut ssig1, &mut csig1);
                geomath::norm(&mut ssig2, &mut csig2);
                S12 = A4 * self.weights.c4x_difference(eps,ssig1,csig1,ssig2,csig2);
            } else {
                S12 = 0.0;
            }

            if !meridian && somg12 > 1.0 {
                somg12 = omg12.sin();
                comg12 = omg12.cos();
            }

            // We're diverging from Karney's implementation here
            // which uses the hardcoded constant: -0.7071 for FRAC_1_SQRT_2
            let alp12: f64;
            if !meridian && comg12 > -FRAC_1_SQRT_2 && sbet2 - sbet1 < 1.75 {
                let domg12 = 1.0 + comg12;
                let dbet1 = 1.0 + cbet1;
                let dbet2 = 1.0 + cbet2;
                alp12 = 2.0
                    * (somg12 * (sbet1 * dbet2 + sbet2 * dbet1))
                        .atan2(domg12 * (sbet1 * sbet2 + dbet1 * dbet2));
            } else {
                let mut salp12 = salp2 * calp1 - calp2 * salp1;
                let mut calp12 = calp2 * calp1 + salp2 * salp1;

                if salp12 == 0.0 && calp12 < 0.0 {
                    salp12 = TINY * calp1;
                    calp12 = -1.0;
                }
                alp12 = salp12.atan2(calp12);
            }
            S12 += self._c2 * alp12;
            S12 *= swapp * lonsign * latsign;
            S12 += 0.0;
        }

        if swapp < 0.0 {
            std::mem::swap(&mut salp2, &mut salp1);

            std::mem::swap(&mut calp2, &mut calp1);

            if C::GEODESICSCALE {
                std::mem::swap(&mut M21, &mut M12);
            }
        }
        salp1 *= swapp * lonsign;
        calp1 *= swapp * latsign;
        salp2 *= swapp * lonsign;
        calp2 *= swapp * latsign;
        (a12, s12, salp1, calp1, salp2, calp2, m12, M12, M21, S12)
    }

    ///  returns (a12, lat2, lon2, azi2, s12, m12, M12, M21, S12)
    pub fn _gen_direct(
        &self,
        lat1: f64,
        lon1: f64,
        azi1: f64,
        arcmode: bool,
        s12_a12: f64,
        mut outmask: u64,
    ) -> (f64, f64, f64, f64, f64, f64, f64, f64, f64) {
        if !arcmode {
            outmask |= caps::DISTANCE_IN
        };

        let line =
            geodesic_line::GeodesicLine::new(self, lat1, lon1, azi1, Some(outmask), None, None);
        line._gen_position(arcmode, s12_a12, outmask)
    }

    /// Get the area of the geodesic in square meters
    pub fn area(&self) -> f64 {
        self._c2 * 4.0 * std::f64::consts::PI
    }
}

/// Place a second point, given the first point, an azimuth, and a distance.
///
/// # Arguments
///   - lat1 - Latitude of 1st point [degrees] [-90.,90.]
///   - lon1 - Longitude of 1st point [degrees] [-180., 180.]
///   - azi1 - Azimuth at 1st point [degrees] [-180., 180.]
///   - s12 - Distance from 1st to 2nd point [meters] Value may be negative
///
/// # Returns
///
/// There are a variety of outputs associated with this calculation. We save computation by
/// only calculating the outputs you need. See the following impls which return different subsets of
/// the following outputs:
///
///  - lat2 latitude of point 2 (degrees).
///  - lon2 longitude of point 2 (degrees).
///  - azi2 (forward) azimuth at point 2 (degrees).
///  - m12 reduced length of geodesic (meters).
///  - M12 geodesic scale of point 2 relative to point 1 (dimensionless).
///  - M21 geodesic scale of point 1 relative to point 2 (dimensionless).
///  - S12 area under the geodesic (meters<sup>2</sup>).
///  - a12 arc length between point 1 and point 2 (degrees).
///
///  If either point is at a pole, the azimuth is defined by keeping the
///  longitude fixed, writing lat = ±(90° − ε), and taking the limit ε → 0+.
///  An arc length greater that 180° signifies a geodesic which is not a
///  shortest path. (For a prolate ellipsoid, an additional condition is
///  necessary for a shortest path: the longitudinal extent must not
///  exceed of 180°.)
/// ```rust
/// // Example, determine the point 10000 km NE of JFK:
/// use geographiclib_rs::{Geodesic, DirectGeodesic};
///
/// let g = Geodesic::wgs84();
/// let (lat, lon, az) = g.direct(40.64, -73.78, 45.0, 10e6);
///
/// use approx::assert_relative_eq;
/// assert_relative_eq!(lat, 32.621100463725796);
/// assert_relative_eq!(lon, 49.052487092959836);
/// assert_relative_eq!(az,  140.4059858768007);
/// ```
pub trait DirectGeodesic<T> {
    fn direct(&self, lat1: f64, lon1: f64, azi1: f64, s12: f64) -> T;
}

impl DirectGeodesic<(f64, f64)> for Geodesic {
    /// See the documentation for the DirectGeodesic trait.
    ///
    /// # Returns
    ///  - lat2 latitude of point 2 (degrees).
    ///  - lon2 longitude of point 2 (degrees).
    fn direct(&self, lat1: f64, lon1: f64, azi1: f64, s12: f64) -> (f64, f64) {
        let capabilities = caps::LATITUDE | caps::LONGITUDE;
        let (_a12, lat2, lon2, _azi2, _s12, _m12, _M12, _M21, _S12) =
            self._gen_direct(lat1, lon1, azi1, false, s12, capabilities);

        (lat2, lon2)
    }
}

impl DirectGeodesic<(f64, f64, f64)> for Geodesic {
    /// See the documentation for the DirectGeodesic trait.
    ///
    /// # Returns
    ///  - lat2 latitude of point 2 (degrees).
    ///  - lon2 longitude of point 2 (degrees).
    ///  - azi2 (forward) azimuth at point 2 (degrees).
    fn direct(&self, lat1: f64, lon1: f64, azi1: f64, s12: f64) -> (f64, f64, f64) {
        let capabilities = caps::LATITUDE | caps::LONGITUDE | caps::AZIMUTH;
        let (_a12, lat2, lon2, azi2, _s12, _m12, _M12, _M21, _S12) =
            self._gen_direct(lat1, lon1, azi1, false, s12, capabilities);

        (lat2, lon2, azi2)
    }
}

impl DirectGeodesic<(f64, f64, f64, f64)> for Geodesic {
    /// See the documentation for the DirectGeodesic trait.
    ///
    /// # Returns
    ///  - lat2 latitude of point 2 (degrees).
    ///  - lon2 longitude of point 2 (degrees).
    ///  - azi2 (forward) azimuth at point 2 (degrees).
    ///  - m12 reduced length of geodesic (meters).
    fn direct(&self, lat1: f64, lon1: f64, azi1: f64, s12: f64) -> (f64, f64, f64, f64) {
        let capabilities = caps::LATITUDE | caps::LONGITUDE | caps::AZIMUTH | caps::REDUCEDLENGTH;
        let (_a12, lat2, lon2, azi2, _s12, m12, _M12, _M21, _S12) =
            self._gen_direct(lat1, lon1, azi1, false, s12, capabilities);

        (lat2, lon2, azi2, m12)
    }
}

impl DirectGeodesic<(f64, f64, f64, f64, f64)> for Geodesic {
    /// See the documentation for the DirectGeodesic trait.
    ///
    /// # Returns
    ///  - lat2 latitude of point 2 (degrees).
    ///  - lon2 longitude of point 2 (degrees).
    ///  - azi2 (forward) azimuth at point 2 (degrees).
    ///  - M12 geodesic scale of point 2 relative to point 1 (dimensionless).
    ///  - M21 geodesic scale of point 1 relative to point 2 (dimensionless).
    fn direct(&self, lat1: f64, lon1: f64, azi1: f64, s12: f64) -> (f64, f64, f64, f64, f64) {
        let capabilities = caps::LATITUDE | caps::LONGITUDE | caps::AZIMUTH | caps::GEODESICSCALE;
        let (_a12, lat2, lon2, azi2, _s12, _m12, M12, M21, _S12) =
            self._gen_direct(lat1, lon1, azi1, false, s12, capabilities);

        (lat2, lon2, azi2, M12, M21)
    }
}

impl DirectGeodesic<(f64, f64, f64, f64, f64, f64)> for Geodesic {
    /// See the documentation for the DirectGeodesic trait.
    ///
    /// # Returns
    ///  - lat2 latitude of point 2 (degrees).
    ///  - lon2 longitude of point 2 (degrees).
    ///  - azi2 (forward) azimuth at point 2 (degrees).
    ///  - m12 reduced length of geodesic (meters).
    ///  - M12 geodesic scale of point 2 relative to point 1 (dimensionless).
    ///  - M21 geodesic scale of point 1 relative to point 2 (dimensionless).
    fn direct(&self, lat1: f64, lon1: f64, azi1: f64, s12: f64) -> (f64, f64, f64, f64, f64, f64) {
        let capabilities = caps::LATITUDE
            | caps::LONGITUDE
            | caps::AZIMUTH
            | caps::REDUCEDLENGTH
            | caps::GEODESICSCALE;
        let (_a12, lat2, lon2, azi2, _s12, m12, M12, M21, _S12) =
            self._gen_direct(lat1, lon1, azi1, false, s12, capabilities);

        (lat2, lon2, azi2, m12, M12, M21)
    }
}

impl DirectGeodesic<(f64, f64, f64, f64, f64, f64, f64, f64)> for Geodesic {
    /// See the documentation for the DirectGeodesic trait.
    ///
    /// # Returns
    ///  - lat2 latitude of point 2 (degrees).
    ///  - lon2 longitude of point 2 (degrees).
    ///  - azi2 (forward) azimuth at point 2 (degrees).
    ///  - m12 reduced length of geodesic (meters).
    ///  - M12 geodesic scale of point 2 relative to point 1 (dimensionless).
    ///  - M21 geodesic scale of point 1 relative to point 2 (dimensionless).
    ///  - S12 area under the geodesic (meters<sup>2</sup>).
    ///  - a12 arc length between point 1 and point 2 (degrees).
    fn direct(
        &self,
        lat1: f64,
        lon1: f64,
        azi1: f64,
        s12: f64,
    ) -> (f64, f64, f64, f64, f64, f64, f64, f64) {
        let capabilities = caps::LATITUDE
            | caps::LONGITUDE
            | caps::AZIMUTH
            | caps::REDUCEDLENGTH
            | caps::GEODESICSCALE
            | caps::AREA;
        let (a12, lat2, lon2, azi2, _s12, m12, M12, M21, S12) =
            self._gen_direct(lat1, lon1, azi1, false, s12, capabilities);

        (lat2, lon2, azi2, m12, M12, M21, S12, a12)
    }
}

/// Measure the distance (and other values) between two points.
///
/// # Arguments
/// - lat1 latitude of point 1 (degrees).
/// - lon1 longitude of point 1 (degrees).
/// - lat2 latitude of point 2 (degrees).
/// - lon2 longitude of point 2 (degrees).
///
/// # Returns
///
/// There are a variety of outputs associated with this calculation. We save computation by
/// only calculating the outputs you need. See the following impls which return different subsets of
/// the following outputs:
///
/// - s12 distance between point 1 and point 2 (meters).
/// - azi1 azimuth at point 1 (degrees).
/// - azi2 (forward) azimuth at point 2 (degrees).
/// - m12 reduced length of geodesic (meters).
/// - M12 geodesic scale of point 2 relative to point 1 (dimensionless).
/// - M21 geodesic scale of point 1 relative to point 2 (dimensionless).
/// - S12 area under the geodesic (meters<sup>2</sup>).
/// - a12 arc length between point 1 and point 2 (degrees).
///
///  `lat1` and `lat2` should be in the range [&minus;90&deg;, 90&deg;].
///  The values of `azi1` and `azi2` returned are in the range
///  [&minus;180&deg;, 180&deg;].
///
/// If either point is at a pole, the azimuth is defined by keeping the
/// longitude fixed, writing `lat` = &plusmn;(90&deg; &minus; &epsilon;),
/// and taking the limit &epsilon; &rarr; 0+.
///
/// The solution to the inverse problem is found using Newton's method.  If
/// this fails to converge (this is very unlikely in geodetic applications
/// but does occur for very eccentric ellipsoids), then the bisection method
/// is used to refine the solution.
///
/// ```rust
/// // Example, determine the distance between two points
/// use geographiclib_rs::{Geodesic, InverseGeodesic};
///
/// let g = Geodesic::wgs84();
/// let p1 = (34.095925, -118.2884237);
/// let p2 = (59.4323439, 24.7341649);
/// let s12: f64 = g.inverse(p1.0, p1.1, p2.0, p2.1);
///
/// use approx::assert_relative_eq;
/// assert_relative_eq!(s12, 9094718.72751138);
/// ```
pub trait InverseGeodesic<T> {
    fn inverse(&self, lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> T;
}

impl InverseGeodesic<f64> for Geodesic {
    /// See the documentation for the InverseGeodesic trait.
    ///
    /// # Returns
    /// - s12 distance between point 1 and point 2 (meters).
    fn inverse(&self, lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        let (_a12, s12, _azi1, _azi2, _m12, _M12, _M21, _S12) =
            self._gen_inverse_azi::<Distance<Empty>>(lat1, lon1, lat2, lon2);

        s12
    }
}

impl InverseGeodesic<(f64, f64)> for Geodesic {
    /// See the documentation for the InverseGeodesic trait.
    ///
    /// # Returns
    /// - s12 distance between point 1 and point 2 (meters).
    /// - a12 arc length between point 1 and point 2 (degrees).
    fn inverse(&self, lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> (f64, f64) {
        let (a12, s12, _azi1, _azi2, _m12, _M12, _M21, _S12) =
            self._gen_inverse_azi::<Distance<Empty>>(lat1, lon1, lat2, lon2);

        (s12, a12)
    }
}

impl InverseGeodesic<(f64, f64, f64)> for Geodesic {
    /// See the documentation for the InverseGeodesic trait.
    ///
    /// # Returns
    /// - azi1 azimuth at point 1 (degrees).
    /// - azi2 (forward) azimuth at point 2 (degrees).
    /// - a12 arc length between point 1 and point 2 (degrees).
    fn inverse(&self, lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> (f64, f64, f64) {
        let (a12, _s12, azi1, azi2, _m12, _M12, _M21, _S12) =
            self._gen_inverse_azi::<Azimuth<Empty>>(lat1, lon1, lat2, lon2);

        (azi1, azi2, a12)
    }
}

impl InverseGeodesic<(f64, f64, f64, f64)> for Geodesic {
    /// See the documentation for the InverseGeodesic trait.
    ///
    /// # Returns
    /// - s12 distance between point 1 and point 2 (meters).
    /// - azi1 azimuth at point 1 (degrees).
    /// - azi2 (forward) azimuth at point 2 (degrees).
    /// - a12 arc length between point 1 and point 2 (degrees).
    fn inverse(&self, lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> (f64, f64, f64, f64) {
        let (a12, s12, azi1, azi2, _m12, _M12, _M21, _S12) =
            self._gen_inverse_azi::<Distance<Azimuth<Empty>>>(lat1, lon1, lat2, lon2);

        (s12, azi1, azi2, a12)
    }
}

impl InverseGeodesic<(f64, f64, f64, f64, f64)> for Geodesic {
    /// See the documentation for the InverseGeodesic trait.
    ///
    /// # Returns
    /// - s12 distance between point 1 and point 2 (meters).
    /// - azi1 azimuth at point 1 (degrees).
    /// - azi2 (forward) azimuth at point 2 (degrees).
    /// - m12 reduced length of geodesic (meters).
    /// - a12 arc length between point 1 and point 2 (degrees).
    fn inverse(&self, lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> (f64, f64, f64, f64, f64) {
        let (a12, s12, azi1, azi2, m12, _M12, _M21, _S12) =
            self._gen_inverse_azi::<ReducedLength<Distance<Azimuth<Empty>>>>(lat1, lon1, lat2, lon2);

        (s12, azi1, azi2, m12, a12)
    }
}

impl InverseGeodesic<(f64, f64, f64, f64, f64, f64)> for Geodesic {
    /// See the documentation for the InverseGeodesic trait.
    ///
    /// # Returns
    /// - s12 distance between point 1 and point 2 (meters).
    /// - azi1 azimuth at point 1 (degrees).
    /// - azi2 (forward) azimuth at point 2 (degrees).
    /// - M12 geodesic scale of point 2 relative to point 1 (dimensionless).
    /// - M21 geodesic scale of point 1 relative to point 2 (dimensionless).
    /// - a12 arc length between point 1 and point 2 (degrees).
    fn inverse(
        &self,
        lat1: f64,
        lon1: f64,
        lat2: f64,
        lon2: f64,
    ) -> (f64, f64, f64, f64, f64, f64) {
        let (a12, s12, azi1, azi2, _m12, M12, M21, _S12) =
            self._gen_inverse_azi::<Distance<Azimuth<GeodesicScale<Empty>>>>(lat1, lon1, lat2, lon2);

        (s12, azi1, azi2, M12, M21, a12)
    }
}

impl InverseGeodesic<(f64, f64, f64, f64, f64, f64, f64)> for Geodesic {
    /// See the documentation for the InverseGeodesic trait.
    ///
    /// # Returns
    /// - s12 distance between point 1 and point 2 (meters).
    /// - azi1 azimuth at point 1 (degrees).
    /// - azi2 (forward) azimuth at point 2 (degrees).
    /// - m12 reduced length of geodesic (meters).
    /// - M12 geodesic scale of point 2 relative to point 1 (dimensionless).
    /// - M21 geodesic scale of point 1 relative to point 2 (dimensionless).
    /// - a12 arc length between point 1 and point 2 (degrees).
    fn inverse(
        &self,
        lat1: f64,
        lon1: f64,
        lat2: f64,
        lon2: f64,
    ) -> (f64, f64, f64, f64, f64, f64, f64) {
        let (a12, s12, azi1, azi2, m12, M12, M21, _S12) =
            self._gen_inverse_azi::<Distance<Azimuth<ReducedLength<GeodesicScale<Empty>>>>>(lat1, lon1, lat2, lon2);

        (s12, azi1, azi2, m12, M12, M21, a12)
    }
}

impl InverseGeodesic<(f64, f64, f64, f64, f64, f64, f64, f64)> for Geodesic {
    /// See the documentation for the InverseGeodesic trait.
    ///
    /// # Returns
    /// - s12 distance between point 1 and point 2 (meters).
    /// - azi1 azimuth at point 1 (degrees).
    /// - azi2 (forward) azimuth at point 2 (degrees).
    /// - m12 reduced length of geodesic (meters).
    /// - M12 geodesic scale of point 2 relative to point 1 (dimensionless).
    /// - M21 geodesic scale of point 1 relative to point 2 (dimensionless).
    /// - S12 area under the geodesic (meters<sup>2</sup>).
    /// - a12 arc length between point 1 and point 2 (degrees).
    fn inverse(
        &self,
        lat1: f64,
        lon1: f64,
        lat2: f64,
        lon2: f64,
    ) -> (f64, f64, f64, f64, f64, f64, f64, f64) {
        let (a12, s12, azi1, azi2, m12, M12, M21, S12) =
            self._gen_inverse_azi::<Distance<Azimuth<ReducedLength<GeodesicScale<Area<Empty>>>>>>(lat1, lon1, lat2, lon2);

        (s12, azi1, azi2, m12, M12, M21, S12, a12)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geodesic_line::GeodesicLine;
    use approx::assert_relative_eq;
    use std::io::BufRead;

    #[allow(clippy::type_complexity)]
    const TESTCASES: &[(f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64)] = &[
        (
            35.60777,
            -139.44815,
            111.098748429560326,
            -11.17491,
            -69.95921,
            129.289270889708762,
            8935244.5604818305,
            80.50729714281974,
            6273170.2055303837,
            0.16606318447386067,
            0.16479116945612937,
            12841384694976.432,
        ),
        (
            55.52454,
            106.05087,
            22.020059880982801,
            77.03196,
            197.18234,
            109.112041110671519,
            4105086.1713924406,
            36.892740690445894,
            3828869.3344387607,
            0.80076349608092607,
            0.80101006984201008,
            61674961290615.615,
        ),
        (
            -21.97856,
            142.59065,
            -32.44456876433189,
            41.84138,
            98.56635,
            -41.84359951440466,
            8394328.894657671,
            75.62930491011522,
            6161154.5773110616,
            0.24816339233950381,
            0.24930251203627892,
            -6637997720646.717,
        ),
        (
            -66.99028,
            112.2363,
            173.73491240878403,
            -12.70631,
            285.90344,
            2.512956620913668,
            11150344.2312080241,
            100.278634181155759,
            6289939.5670446687,
            -0.17199490274700385,
            -0.17722569526345708,
            -121287239862139.744,
        ),
        (
            -17.42761,
            173.34268,
            -159.033557661192928,
            -15.84784,
            5.93557,
            -20.787484651536988,
            16076603.1631180673,
            144.640108810286253,
            3732902.1583877189,
            -0.81273638700070476,
            -0.81299800519154474,
            97825992354058.708,
        ),
        (
            32.84994,
            48.28919,
            150.492927788121982,
            -56.28556,
            202.29132,
            48.113449399816759,
            16727068.9438164461,
            150.565799985466607,
            3147838.1910180939,
            -0.87334918086923126,
            -0.86505036767110637,
            -72445258525585.010,
        ),
        (
            6.96833,
            52.74123,
            92.581585386317712,
            -7.39675,
            206.17291,
            90.721692165923907,
            17102477.2496958388,
            154.147366239113561,
            2772035.6169917581,
            -0.89991282520302447,
            -0.89986892177110739,
            -1311796973197.995,
        ),
        (
            -50.56724,
            -16.30485,
            -105.439679907590164,
            -33.56571,
            -94.97412,
            -47.348547835650331,
            6455670.5118668696,
            58.083719495371259,
            5409150.7979815838,
            0.53053508035997263,
            0.52988722644436602,
            41071447902810.047,
        ),
        (
            -58.93002,
            -8.90775,
            140.965397902500679,
            -8.91104,
            133.13503,
            19.255429433416599,
            11756066.0219864627,
            105.755691241406877,
            6151101.2270708536,
            -0.26548622269867183,
            -0.27068483874510741,
            -86143460552774.735,
        ),
        (
            -68.82867,
            -74.28391,
            93.774347763114881,
            -50.63005,
            -8.36685,
            34.65564085411343,
            3956936.926063544,
            35.572254987389284,
            3708890.9544062657,
            0.81443963736383502,
            0.81420859815358342,
            -41845309450093.787,
        ),
        (
            -10.62672,
            -32.0898,
            -86.426713286747751,
            5.883,
            -134.31681,
            -80.473780971034875,
            11470869.3864563009,
            103.387395634504061,
            6184411.6622659713,
            -0.23138683500430237,
            -0.23155097622286792,
            4198803992123.548,
        ),
        (
            -21.76221,
            166.90563,
            29.319421206936428,
            48.72884,
            213.97627,
            43.508671946410168,
            9098627.3986554915,
            81.963476716121964,
            6299240.9166992283,
            0.13965943368590333,
            0.14152969707656796,
            10024709850277.476,
        ),
        (
            -19.79938,
            -174.47484,
            71.167275780171533,
            -11.99349,
            -154.35109,
            65.589099775199228,
            2319004.8601169389,
            20.896611684802389,
            2267960.8703918325,
            0.93427001867125849,
            0.93424887135032789,
            -3935477535005.785,
        ),
        (
            -11.95887,
            -116.94513,
            92.712619830452549,
            4.57352,
            7.16501,
            78.64960934409585,
            13834722.5801401374,
            124.688684161089762,
            5228093.177931598,
            -0.56879356755666463,
            -0.56918731952397221,
            -9919582785894.853,
        ),
        (
            -87.85331,
            85.66836,
            -65.120313040242748,
            66.48646,
            16.09921,
            -4.888658719272296,
            17286615.3147144645,
            155.58592449699137,
            2635887.4729110181,
            -0.90697975771398578,
            -0.91095608883042767,
            42667211366919.534,
        ),
        (
            1.74708,
            128.32011,
            -101.584843631173858,
            -11.16617,
            11.87109,
            -86.325793296437476,
            12942901.1241347408,
            116.650512484301857,
            5682744.8413270572,
            -0.44857868222697644,
            -0.44824490340007729,
            10763055294345.653,
        ),
        (
            -25.72959,
            -144.90758,
            -153.647468693117198,
            -57.70581,
            -269.17879,
            -48.343983158876487,
            9413446.7452453107,
            84.664533838404295,
            6356176.6898881281,
            0.09492245755254703,
            0.09737058264766572,
            74515122850712.444,
        ),
        (
            -41.22777,
            122.32875,
            14.285113402275739,
            -7.57291,
            130.37946,
            10.805303085187369,
            3812686.035106021,
            34.34330804743883,
            3588703.8812128856,
            0.82605222593217889,
            0.82572158200920196,
            -2456961531057.857,
        ),
        (
            11.01307,
            138.25278,
            79.43682622782374,
            6.62726,
            247.05981,
            103.708090215522657,
            11911190.819018408,
            107.341669954114577,
            6070904.722786735,
            -0.29767608923657404,
            -0.29785143390252321,
            17121631423099.696,
        ),
        (
            -29.47124,
            95.14681,
            -163.779130441688382,
            -27.46601,
            -69.15955,
            -15.909335945554969,
            13487015.8381145492,
            121.294026715742277,
            5481428.9945736388,
            -0.51527225545373252,
            -0.51556587964721788,
            104679964020340.318,
        ),
    ];

    #[test]
    fn test_inverse_and_direct() -> Result<(), String> {
        // See python/test_geodesic.py
        let geod = Geodesic::wgs84();
        let (_a12, s12, _azi1, _azi2, _m12, _M12, _M21, _S12) =
            geod._gen_inverse_azi::<Standard>(0.0, 0.0, 1.0, 1.0);
        assert_eq!(s12, 156899.56829134026);

        // Test inverse
        for (lat1, lon1, azi1, lat2, lon2, azi2, s12, a12, m12, M12, M21, S12) in TESTCASES.iter() {
            let (
                computed_a12,
                computed_s12,
                computed_azi1,
                computed_azi2,
                computed_m12,
                computed_M12,
                computed_M21,
                computed_S12,
            ) = geod._gen_inverse_azi::<All>(*lat1, *lon1, *lat2, *lon2);
            assert_relative_eq!(computed_azi1, azi1, epsilon = 1e-13f64);
            assert_relative_eq!(computed_azi2, azi2, epsilon = 1e-13f64);
            assert_relative_eq!(computed_s12, s12, epsilon = 1e-8f64);
            assert_relative_eq!(computed_a12, a12, epsilon = 1e-13f64);
            assert_relative_eq!(computed_m12, m12, epsilon = 1e-8f64);
            assert_relative_eq!(computed_M12, M12, epsilon = 1e-15f64);
            assert_relative_eq!(computed_M21, M21, epsilon = 1e-15f64);
            assert_relative_eq!(computed_S12, S12, epsilon = 0.1f64);
        }

        // Test direct
        for (lat1, lon1, azi1, lat2, lon2, azi2, s12, a12, m12, M12, M21, S12) in TESTCASES.iter() {
            let (
                computed_a12,
                computed_lat2,
                computed_lon2,
                computed_azi2,
                _computed_s12,
                computed_m12,
                computed_M12,
                computed_M21,
                computed_S12,
            ) = geod._gen_direct(
                *lat1,
                *lon1,
                *azi1,
                false,
                *s12,
                caps::ALL | caps::LONG_UNROLL,
            );
            assert_relative_eq!(computed_lat2, lat2, epsilon = 1e-13f64);
            assert_relative_eq!(computed_lon2, lon2, epsilon = 1e-13f64);
            assert_relative_eq!(computed_azi2, azi2, epsilon = 1e-13f64);
            assert_relative_eq!(computed_a12, a12, epsilon = 1e-13f64);
            assert_relative_eq!(computed_m12, m12, epsilon = 1e-8f64);
            assert_relative_eq!(computed_M12, M12, epsilon = 1e-15f64);
            assert_relative_eq!(computed_M21, M21, epsilon = 1e-15f64);
            assert_relative_eq!(computed_S12, S12, epsilon = 0.1f64);
        }
        Ok(())
    }

    #[test]
    fn test_arcdirect() {
        // Corresponds with ArcDirectCheck from Java, or test_arcdirect from Python
        let geod = Geodesic::wgs84();
        for (lat1, lon1, azi1, lat2, lon2, azi2, s12, a12, m12, M12, M21, S12) in TESTCASES.iter() {
            let (
                _computed_a12,
                computed_lat2,
                computed_lon2,
                computed_azi2,
                computed_s12,
                computed_m12,
                computed_M12,
                computed_M21,
                computed_S12,
            ) = geod._gen_direct(
                *lat1,
                *lon1,
                *azi1,
                true,
                *a12,
                caps::ALL | caps::LONG_UNROLL,
            );
            assert_relative_eq!(computed_lat2, lat2, epsilon = 1e-13);
            assert_relative_eq!(computed_lon2, lon2, epsilon = 1e-13);
            assert_relative_eq!(computed_azi2, azi2, epsilon = 1e-13);
            assert_relative_eq!(computed_s12, s12, epsilon = 1e-8);
            assert_relative_eq!(computed_m12, m12, epsilon = 1e-8);
            assert_relative_eq!(computed_M12, M12, epsilon = 1e-15);
            assert_relative_eq!(computed_M21, M21, epsilon = 1e-15);
            assert_relative_eq!(computed_S12, S12, epsilon = 0.1);
        }
    }

    #[test]
    fn test_geninverse() {
        let geod = Geodesic::wgs84();
        let res = geod._gen_inverse::<Standard>(0.0, 0.0, 1.0, 1.0);
        assert_eq!(res.0, 1.4141938478710363);
        assert_eq!(res.1, 156899.56829134026);
        assert_eq!(res.2, 0.7094236375834774);
        assert_eq!(res.3, 0.7047823085448635);
        assert_eq!(res.4, 0.7095309793242709);
        assert_eq!(res.5, 0.7046742434480923);
        assert!(res.6.is_nan());
        assert!(res.7.is_nan());
        assert!(res.8.is_nan());
        assert!(res.9.is_nan());
    }

    #[test]
    fn test_inverse_start() {
        let geod = Geodesic::wgs84();
        let res = geod._InverseStart(
            -0.017393909556108908,
            0.9998487145115275,
            1.0000010195104125,
            -0.0,
            1.0,
            1.0,
            0.017453292519943295,
            0.01745240643728351,
            0.9998476951563913,
        );
        assert_eq!(res.0, -1.0);
        assert_relative_eq!(res.1, 0.7095310092765433, epsilon = 1e-13);
        assert_relative_eq!(res.2, 0.7046742132893822, epsilon = 1e-13);
        assert!(res.3.is_nan());
        assert!(res.4.is_nan());
        assert_eq!(res.5, 1.0000002548969817);

        let res = geod._InverseStart(
            -0.017393909556108908,
            0.9998487145115275,
            1.0000010195104125,
            -0.0,
            1.0,
            1.0,
            0.017453292519943295,
            0.01745240643728351,
            0.9998476951563913,
        );
        assert_eq!(res.0, -1.0);
        assert_relative_eq!(res.1, 0.7095310092765433, epsilon = 1e-13);
        assert_relative_eq!(res.2, 0.7046742132893822, epsilon = 1e-13);
        assert!(res.3.is_nan());
        assert!(res.4.is_nan());
        assert_eq!(res.5, 1.0000002548969817);
    }

    #[test]
    fn test_lambda12() {
        let geod = Geodesic::wgs84();
        let res1 = geod._Lambda12(
            -0.017393909556108908,
            0.9998487145115275,
            1.0000010195104125,
            -0.0,
            1.0,
            1.0,
            0.7095310092765433,
            0.7046742132893822,
            0.01745240643728351,
            0.9998476951563913,
            true,
        );
        assert_eq!(res1.0, 1.4834408705897495e-09);
        assert_eq!(res1.1, 0.7094236675312185);
        assert_eq!(res1.2, 0.7047822783999007);
        assert_eq!(res1.3, 0.024682339962725352);
        assert_eq!(res1.4, -0.024679833885152578);
        assert_eq!(res1.5, 0.9996954065111039);
        assert_eq!(res1.6, -0.0);
        assert_eq!(res1.7, 1.0);
        assert_relative_eq!(res1.8, 0.0008355095326524276, epsilon = 1e-13);
        assert_eq!(res1.9, -5.8708496511415445e-05);
        assert_eq!(res1.10, 0.034900275148485);

        let res2 = geod._Lambda12(
            -0.017393909556108908,
            0.9998487145115275,
            1.0000010195104125,
            -0.0,
            1.0,
            1.0,
            0.7095309793242709,
            0.7046742434480923,
            0.01745240643728351,
            0.9998476951563913,
            true,
        );
        assert_eq!(res2.0, 6.046459990680098e-17);
        assert_eq!(res2.1, 0.7094236375834774);
        assert_eq!(res2.2, 0.7047823085448635);
        assert_eq!(res2.3, 0.024682338906797385);
        assert_eq!(res2.4, -0.02467983282954624);
        assert_eq!(res2.5, 0.9996954065371639);
        assert_eq!(res2.6, -0.0);
        assert_eq!(res2.7, 1.0);
        assert_relative_eq!(res2.8, 0.0008355096040059597, epsilon = 1e-18);
        assert_eq!(res2.9, -5.870849152149326e-05);
        assert_eq!(res2.10, 0.03490027216297455);
    }

    #[test]
    fn test_lengths() {
        use crate::traits::S12bAlone;
        // Results taken from the python implementation
        let geod = Geodesic::wgs84();
        let res1 = geod._Lengths::<M0<M12b>,ReducedLength<Empty>>(
            0.0008355095326524276,
            0.024682339962725352,
            -0.024679833885152578,
            0.9996954065111039,
            1.0000010195104125,
            -0.0,
            1.0,
            1.0,
            0.9998487145115275,
            1.0,
        );
        assert!(res1.get_s12b().is_nan());
        assert_eq!(res1.get_m12b(), 0.024679842274314294);
        assert_eq!(res1.get_m0(), 0.0016717180169067588);
        assert!(res1.get_m12().is_nan());
        assert!(res1.get_m21().is_nan());

        let res2 = geod._Lengths::<M0<M12b>,ReducedLength<Empty>>(
            0.0008355096040059597,
            0.024682338906797385,
            -0.02467983282954624,
            0.9996954065371639,
            1.0000010195104125,
            -0.0,
            1.0,
            1.0,
            0.9998487145115275,
            1.0,
        );
        assert!(res1.get_s12b().is_nan());
        assert_eq!(res2.get_m12b(), 0.02467984121870759);
        assert_eq!(res2.get_m0(), 0.0016717181597332804);
        assert!(res1.get_m12().is_nan());
        assert!(res1.get_m21().is_nan());

        let res3 = geod._Lengths::<S12bAlone,Distance<Empty>>(
            0.0008355096040059597,
            0.024682338906797385,
            -0.02467983282954624,
            0.9996954065371639,
            1.0000010195104125,
            -0.0,
            1.0,
            1.0,
            0.9998487145115275,
            1.0,
        );
        assert_eq!(res3.get_s12b(), 0.024682347295447677);
        assert!(res3.get_m12b().is_nan());
        assert!(res3.get_m0().is_nan());
        assert!(res3.get_m12().is_nan());
        assert!(res3.get_m21().is_nan());

        let res = geod._Lengths::<S12bAlone,Distance<Empty>>(
            0.0007122620325664751,
            1.405117407023628,
            -0.8928657853278468,
            0.45032287238256896,
            1.0011366173804046,
            0.2969032234925426,
            0.9549075745221299,
            1.0001257451360057,
            0.8139459053827204,
            0.9811634781422108,
        );
        assert_eq!(res.get_s12b(), 1.4056304412645388);
        assert!(res3.get_m12b().is_nan());
        assert!(res3.get_m0().is_nan());
        assert!(res3.get_m12().is_nan());
        assert!(res3.get_m21().is_nan());
    }

    #[test]
    fn test_goed__A3f() {
        let geod = Geodesic::wgs84();
        assert_eq!(geod._A3f(0.12), 0.9363788874000158);
    }

    #[test]
    fn test_geod_init() {
        // Check that after the init the variables are correctly set.
        // Actual values are taken from the python implementation
        let geod = Geodesic::wgs84();
        assert_eq!(geod.a, 6378137.0, "geod.a wrong");
        assert_eq!(geod.f, 0.0033528106647474805, "geod.f wrong");
        assert_eq!(geod._f1, 0.9966471893352525, "geod._f1 wrong");
        assert_eq!(geod._e2, 0.0066943799901413165, "geod._e2 wrong");
        assert_eq!(geod._ep2, 0.006739496742276434, "geod._ep2 wrong");
        assert_eq!(geod._n, 0.0016792203863837047, "geod._n wrong");
        assert_eq!(geod._b, 6356752.314245179, "geod._b wrong");
        assert_eq!(geod._c2, 40589732499314.76, "geod._c2 wrong");
        assert_eq!(geod._etol2, 3.6424611488788524e-08, "geod._etol2 wrong");
        assert_eq!(
            geod.weights.a3x,
            [
                -0.0234375,
                -0.046927475637074494,
                -0.06281503005876607,
                -0.2502088451303832,
                -0.49916038980680816,
                1.0
            ],
            "geod._A3x wrong"
        );

        assert_eq!(
            geod.weights.c3x,
            [
                0.0234375,
                0.03908873781853724,
                0.04695366939653196,
                0.12499964752736174,
                0.24958019490340408,
                0.01953125,
                0.02345061890926862,
                0.046822392185686165,
                0.062342661206936094,
                0.013671875,
                0.023393770302437927,
                0.025963026642854565,
                0.013671875,
                0.01362595881755982,
                0.008203125
            ],
            "geod._C3x wrong"
        );
        assert_eq!(
            geod.weights.c4x,
            [
                0.00646020646020646,
                0.0035037627212872787,
                0.034742279454780166,
                -0.01921732223244865,
                -0.19923321555984239,
                0.6662190894642603,
                0.000111000111000111,
                0.003426620602971002,
                -0.009510765372597735,
                -0.01893413691235592,
                0.0221370239510936,
                0.0007459207459207459,
                -0.004142006291321442,
                -0.00504225176309005,
                0.007584982177746079,
                -0.0021565735851450138,
                -0.001962613370670692,
                0.0036104265913438913,
                -0.0009472009472009472,
                0.0020416649913317735,
                0.0012916376552740189
            ],
            "geod._C4x wrong"
        );
    }

    // The test_std_geodesic_* tests below are based on Karney's GeodSolve unit
    // tests, found in many geographiclib variants.
    // The versions below are mostly adapted from their Java counterparts,
    // which use a testing structure more similar to Rust than do the C++ versions.
    // Note that the Java tests often incorporate more than one of the C++ tests,
    // and take their name from the lowest-numbered test in the set.
    // These tests use that convention as well.

    #[test]
    fn test_std_geodesic_geodsolve0() {
        let geod = Geodesic::wgs84();
        let (s12, azi1, azi2, _a12) = geod.inverse(40.6, -73.8, 49.01666667, 2.55);
        assert_relative_eq!(azi1, 53.47022, epsilon = 0.5e-5);
        assert_relative_eq!(azi2, 111.59367, epsilon = 0.5e-5);
        assert_relative_eq!(s12, 5853226.0, epsilon = 0.5);
    }

    #[test]
    fn test_std_geodesic_geodsolve1() {
        let geod = Geodesic::wgs84();
        let (lat2, lon2, azi2) = geod.direct(40.63972222, -73.77888889, 53.5, 5850e3);
        assert_relative_eq!(lat2, 49.01467, epsilon = 0.5e-5);
        assert_relative_eq!(lon2, 2.56106, epsilon = 0.5e-5);
        assert_relative_eq!(azi2, 111.62947, epsilon = 0.5e-5);
    }

    #[test]
    fn test_std_geodesic_geodsolve2() {
        // Check fix for antipodal prolate bug found 2010-09-04
        let geod = Geodesic::new(6.4e6, -1f64 / 150.0);
        let (s12, azi1, azi2, _a12) = geod.inverse(0.07476, 0.0, -0.07476, 180.0);
        assert_relative_eq!(azi1, 90.00078, epsilon = 0.5e-5);
        assert_relative_eq!(azi2, 90.00078, epsilon = 0.5e-5);
        assert_relative_eq!(s12, 20106193.0, epsilon = 0.5);
        let (s12, azi1, azi2, _a12) = geod.inverse(0.1, 0.0, -0.1, 180.0);
        assert_relative_eq!(azi1, 90.00105, epsilon = 0.5e-5);
        assert_relative_eq!(azi2, 90.00105, epsilon = 0.5e-5);
        assert_relative_eq!(s12, 20106193.0, epsilon = 0.5);
    }

    #[test]
    fn test_std_geodesic_geodsolve4() {
        // Check fix for short line bug found 2010-05-21
        let geod = Geodesic::wgs84();
        let s12: f64 = geod.inverse(36.493349428792, 0.0, 36.49334942879201, 0.0000008);
        assert_relative_eq!(s12, 0.072, epsilon = 0.5e-3);
    }

    #[test]
    fn test_std_geodesic_geodsolve5() {
        // Check fix for point2=pole bug found 2010-05-03
        let geod = Geodesic::wgs84();
        let (lat2, lon2, azi2) = geod.direct(0.01777745589997, 30.0, 0.0, 10e6);
        assert_relative_eq!(lat2, 90.0, epsilon = 0.5e-5);
        if lon2 < 0.0 {
            assert_relative_eq!(lon2, -150.0, epsilon = 0.5e-5);
            assert_relative_eq!(azi2.abs(), 180.0, epsilon = 0.5e-5);
        } else {
            assert_relative_eq!(lon2, 30.0, epsilon = 0.5e-5);
            assert_relative_eq!(azi2, 0.0, epsilon = 0.5e-5);
        }
    }

    #[test]
    fn test_std_geodesic_geodsolve6() {
        // Check fix for volatile sbet12a bug found 2011-06-25 (gcc 4.4.4
        // x86 -O3).  Found again on 2012-03-27 with tdm-mingw32 (g++ 4.6.1).
        let geod = Geodesic::wgs84();
        let s12: f64 = geod.inverse(
            88.202499451857,
            0.0,
            -88.202499451857,
            179.981022032992859592,
        );
        assert_relative_eq!(s12, 20003898.214, epsilon = 0.5e-3);
        let s12: f64 = geod.inverse(
            89.333123580033,
            0.0,
            -89.333123580032997687,
            179.99295812360148422,
        );
        assert_relative_eq!(s12, 20003926.881, epsilon = 0.5e-3);
    }

    #[test]
    fn test_std_geodesic_geodsolve9() {
        // Check fix for volatile x bug found 2011-06-25 (gcc 4.4.4 x86 -O3)
        let geod = Geodesic::wgs84();
        let s12: f64 = geod.inverse(
            56.320923501171,
            0.0,
            -56.320923501171,
            179.664747671772880215,
        );
        assert_relative_eq!(s12, 19993558.287, epsilon = 0.5e-3);
    }

    #[test]
    fn test_std_geodesic_geodsolve10() {
        // Check fix for adjust tol1_ bug found 2011-06-25 (Visual Studio
        // 10 rel + debug)
        let geod = Geodesic::wgs84();
        let s12: f64 = geod.inverse(
            52.784459512564,
            0.0,
            -52.784459512563990912,
            179.634407464943777557,
        );
        assert_relative_eq!(s12, 19991596.095, epsilon = 0.5e-3);
    }

    #[test]
    fn test_std_geodesic_geodsolve11() {
        // Check fix for bet2 = -bet1 bug found 2011-06-25 (Visual Studio
        // 10 rel + debug)
        let geod = Geodesic::wgs84();
        let s12: f64 = geod.inverse(
            48.522876735459,
            0.0,
            -48.52287673545898293,
            179.599720456223079643,
        );
        assert_relative_eq!(s12, 19989144.774, epsilon = 0.5e-3);
    }

    #[test]
    fn test_std_geodesic_geodsolve12() {
        // Check fix for inverse geodesics on extreme prolate/oblate
        // ellipsoids Reported 2012-08-29 Stefan Guenther
        // <stefan.gunther@embl.de>; fixed 2012-10-07
        let geod = Geodesic::new(89.8, -1.83);
        let (s12, azi1, azi2, _a12) = geod.inverse(0.0, 0.0, -10.0, 160.0);
        assert_relative_eq!(azi1, 120.27, epsilon = 1e-2);
        assert_relative_eq!(azi2, 105.15, epsilon = 1e-2);
        assert_relative_eq!(s12, 266.7, epsilon = 1e-1);
    }

    #[test]
    fn test_std_geodesic_geodsolve14() {
        // Check fix for inverse ignoring lon12 = nan
        let geod = Geodesic::wgs84();
        let (s12, azi1, azi2, _a12) = geod.inverse(0.0, 0.0, 1.0, f64::NAN);
        assert!(azi1.is_nan());
        assert!(azi2.is_nan());
        assert!(s12.is_nan());
    }

    #[test]
    fn test_std_geodesic_geodsolve15() {
        // Initial implementation of Math::eatanhe was wrong for e^2 < 0.  This
        // checks that this is fixed.
        let geod = Geodesic::new(6.4e6, -1f64 / 150.0);
        let (_lat2, _lon2, _azi2, _m12, _M12, _M21, S12, _a12) = geod.direct(1.0, 2.0, 3.0, 4.0);
        assert_relative_eq!(S12, 23700.0, epsilon = 0.5);
    }

    #[test]
    fn test_std_geodesic_geodsolve17() {
        // Check fix for LONG_UNROLL bug found on 2015-05-07
        let geod = Geodesic::new(6.4e6, -1f64 / 150.0);
        let (_a12, lat2, lon2, azi2, _s12, _m12, _M12, _M21, _S12) = geod._gen_direct(
            40.0,
            -75.0,
            -10.0,
            false,
            2e7,
            caps::STANDARD | caps::LONG_UNROLL,
        );
        assert_relative_eq!(lat2, -39.0, epsilon = 1.0);
        assert_relative_eq!(lon2, -254.0, epsilon = 1.0);
        assert_relative_eq!(azi2, -170.0, epsilon = 1.0);

        let line = GeodesicLine::new(&geod, 40.0, -75.0, -10.0, None, None, None);
        let (_a12, lat2, lon2, azi2, _s12, _m12, _M12, _M21, _S12) =
            line._gen_position(false, 2e7, caps::STANDARD | caps::LONG_UNROLL);
        assert_relative_eq!(lat2, -39.0, epsilon = 1.0);
        assert_relative_eq!(lon2, -254.0, epsilon = 1.0);
        assert_relative_eq!(azi2, -170.0, epsilon = 1.0);

        let (lat2, lon2, azi2) = geod.direct(40.0, -75.0, -10.0, 2e7);
        assert_relative_eq!(lat2, -39.0, epsilon = 1.0);
        assert_relative_eq!(lon2, 105.0, epsilon = 1.0);
        assert_relative_eq!(azi2, -170.0, epsilon = 1.0);

        let (_a12, lat2, lon2, azi2, _s12, _m12, _M12, _M21, _S12) =
            line._gen_position(false, 2e7, caps::STANDARD);
        assert_relative_eq!(lat2, -39.0, epsilon = 1.0);
        assert_relative_eq!(lon2, 105.0, epsilon = 1.0);
        assert_relative_eq!(azi2, -170.0, epsilon = 1.0);
    }

    #[test]
    fn test_std_geodesic_geodsolve26() {
        // Check 0/0 problem with area calculation on sphere 2015-09-08
        let geod = Geodesic::new(6.4e6, 0.0);
        let (_a12, _s12, _salp1, _calp1, _salp2, _calp2, _m12, _M12, _M21, S12) =
            geod._gen_inverse::<Area<Empty>>(1.0, 2.0, 3.0, 4.0);
        assert_relative_eq!(S12, 49911046115.0, epsilon = 0.5);
    }

    #[test]
    fn test_std_geodesic_geodsolve28() {
        // Check for bad placement of assignment of r.a12 with |f| > 0.01 (bug in
        // Java implementation fixed on 2015-05-19).
        let geod = Geodesic::new(6.4e6, 0.1);
        let (a12, _lat2, _lon2, _azi2, _s12, _m12, _M12, _M21, _S12) =
            geod._gen_direct(1.0, 2.0, 10.0, false, 5e6, caps::STANDARD);
        assert_relative_eq!(a12, 48.55570690, epsilon = 0.5e-8);
    }

    #[test]
    fn test_std_geodesic_geodsolve29() {
        // Check longitude unrolling with inverse calculation 2015-09-16
        let geod = Geodesic::wgs84();
        let (_a12, s12, _salp1, _calp1, _salp2, _calp2, _m12, _M12, _M21, _S12) =
            geod._gen_inverse::<Standard>(0.0, 539.0, 0.0, 181.0);
        // Note: This is also supposed to check adjusted longitudes, but geographiclib-rs
        //       doesn't seem to support that as of 2021/01/18.
        // assert_relative_eq!(lon1, 179, epsilon = 1e-10);
        // assert_relative_eq!(lon2, -179, epsilon = 1e-10);
        assert_relative_eq!(s12, 222639.0, epsilon = 0.5);
        let (_a12, s12, _salp1, _calp1, _salp2, _calp2, _m12, _M12, _M21, _S12) =
            geod._gen_inverse::<Standard>(0.0, 539.0, 0.0, 181.0);
        // assert_relative_eq!(lon1, 539, epsilon = 1e-10);
        // assert_relative_eq!(lon2, 541, epsilon = 1e-10);
        assert_relative_eq!(s12, 222639.0, epsilon = 0.5);
    }

    #[test]
    fn test_std_geodesic_geodsolve33() {
        // Check max(-0.0,+0.0) issues 2015-08-22 (triggered by bugs in Octave --
        // sind(-0.0) = +0.0 -- and in some version of Visual Studio --
        // fmod(-0.0, 360.0) = +0.0.
        let geod = Geodesic::wgs84();
        let (s12, azi1, azi2, _a12) = geod.inverse(0.0, 0.0, 0.0, 179.0);
        assert_relative_eq!(azi1, 90.0, epsilon = 0.5e-5);
        assert_relative_eq!(azi2, 90.0, epsilon = 0.5e-5);
        assert_relative_eq!(s12, 19926189.0, epsilon = 0.5);
        let (s12, azi1, azi2, _a12) = geod.inverse(0.0, 0.0, 0.0, 179.5);
        assert_relative_eq!(azi1, 55.96650, epsilon = 0.5e-5);
        assert_relative_eq!(azi2, 124.03350, epsilon = 0.5e-5);
        assert_relative_eq!(s12, 19980862.0, epsilon = 0.5);
        let (s12, azi1, azi2, _a12) = geod.inverse(0.0, 0.0, 0.0, 180.0);
        assert_relative_eq!(azi1, 0.0, epsilon = 0.5e-5);
        assert_relative_eq!(azi2.abs(), 180.0, epsilon = 0.5e-5);
        assert_relative_eq!(s12, 20003931.0, epsilon = 0.5);
        let (s12, azi1, azi2, _a12) = geod.inverse(0.0, 0.0, 1.0, 180.0);
        assert_relative_eq!(azi1, 0.0, epsilon = 0.5e-5);
        assert_relative_eq!(azi2.abs(), 180.0, epsilon = 0.5e-5);
        assert_relative_eq!(s12, 19893357.0, epsilon = 0.5);

        let geod = Geodesic::new(6.4e6, 0.0);
        let (s12, azi1, azi2, _a12) = geod.inverse(0.0, 0.0, 0.0, 179.0);
        assert_relative_eq!(azi1, 90.0, epsilon = 0.5e-5);
        assert_relative_eq!(azi2, 90.0, epsilon = 0.5e-5);
        assert_relative_eq!(s12, 19994492.0, epsilon = 0.5);
        let (s12, azi1, azi2, _a12) = geod.inverse(0.0, 0.0, 0.0, 180.0);
        assert_relative_eq!(azi1, 0.0, epsilon = 0.5e-5);
        assert_relative_eq!(azi2.abs(), 180.0, epsilon = 0.5e-5);
        assert_relative_eq!(s12, 20106193.0, epsilon = 0.5);
        let (s12, azi1, azi2, _a12) = geod.inverse(0.0, 0.0, 1.0, 180.0);
        assert_relative_eq!(azi1, 0.0, epsilon = 0.5e-5);
        assert_relative_eq!(azi2.abs(), 180.0, epsilon = 0.5e-5);
        assert_relative_eq!(s12, 19994492.0, epsilon = 0.5);

        let geod = Geodesic::new(6.4e6, -1.0 / 300.0);
        let (s12, azi1, azi2, _a12) = geod.inverse(0.0, 0.0, 0.0, 179.0);
        assert_relative_eq!(azi1, 90.0, epsilon = 0.5e-5);
        assert_relative_eq!(azi2, 90.0, epsilon = 0.5e-5);
        assert_relative_eq!(s12, 19994492.0, epsilon = 0.5);
        let (s12, azi1, azi2, _a12) = geod.inverse(0.0, 0.0, 0.0, 180.0);
        assert_relative_eq!(azi1, 90.0, epsilon = 0.5e-5);
        assert_relative_eq!(azi2, 90.0, epsilon = 0.5e-5);
        assert_relative_eq!(s12, 20106193.0, epsilon = 0.5);
        let (s12, azi1, azi2, _a12) = geod.inverse(0.0, 0.0, 0.5, 180.0);
        assert_relative_eq!(azi1, 33.02493, epsilon = 0.5e-5);
        assert_relative_eq!(azi2, 146.97364, epsilon = 0.5e-5);
        assert_relative_eq!(s12, 20082617.0, epsilon = 0.5);
        let (s12, azi1, azi2, _a12) = geod.inverse(0.0, 0.0, 1.0, 180.0);
        assert_relative_eq!(azi1, 0.0, epsilon = 0.5e-5);
        assert_relative_eq!(azi2.abs(), 180.0, epsilon = 0.5e-5);
        assert_relative_eq!(s12, 20027270.0, epsilon = 0.5);
    }

    #[test]
    fn test_std_geodesic_geodsolve55() {
        // Check fix for nan + point on equator or pole not returning all nans in
        // Geodesic::Inverse, found 2015-09-23.
        let geod = Geodesic::wgs84();
        let (s12, azi1, azi2, _a12) = geod.inverse(f64::NAN, 0.0, 0.0, 90.0);
        assert!(azi1.is_nan());
        assert!(azi2.is_nan());
        assert!(s12.is_nan());
        let (s12, azi1, azi2, _a12) = geod.inverse(f64::NAN, 0.0, 90.0, 3.0);
        assert!(azi1.is_nan());
        assert!(azi2.is_nan());
        assert!(s12.is_nan());
    }

    #[test]
    fn test_std_geodesic_geodsolve59() {
        // Check for points close with longitudes close to 180 deg apart.
        let geod = Geodesic::wgs84();
        let (s12, azi1, azi2, _a12) = geod.inverse(5.0, 0.00000000000001, 10.0, 180.0);
        assert_relative_eq!(azi1, 0.000000000000035, epsilon = 1.5e-14);
        assert_relative_eq!(azi2, 179.99999999999996, epsilon = 1.5e-14);
        assert_relative_eq!(s12, 18345191.174332713, epsilon = 5e-9);
    }

    #[test]
    fn test_std_geodesic_geodsolve61() {
        // Make sure small negative azimuths are west-going
        let geod = Geodesic::wgs84();
        let (_a12, lat2, lon2, azi2, _s12, _m12, _M12, _M21, _S12) = geod._gen_direct(
            45.0,
            0.0,
            -0.000000000000000003,
            false,
            1e7,
            caps::STANDARD | caps::LONG_UNROLL,
        );
        assert_relative_eq!(lat2, 45.30632, epsilon = 0.5e-5);
        assert_relative_eq!(lon2, -180.0, epsilon = 0.5e-5);
        assert_relative_eq!(azi2.abs(), 180.0, epsilon = 0.5e-5);
        // geographiclib-rs does not appear to support Geodesic.inverse_line or
        // or GeodesicLine.position as of 2021/01/18.
        // let line = geod.inverse_line(45, 0, 80, -0.000000000000000003);
        // let res = line.position(1e7, caps::STANDARD | caps::LONG_UNROLL);
        // assert_relative_eq!(lat2, 45.30632, epsilon = 0.5e-5);
        // assert_relative_eq!(lon2, -180, epsilon = 0.5e-5);
        // assert_relative_eq!(azi2.abs(), 180, epsilon = 0.5e-5);
    }

    // #[test]
    // fn test_std_geodesic_geodsolve65() {
    //     // Check for bug in east-going check in GeodesicLine (needed to check for
    //     // sign of 0) and sign error in area calculation due to a bogus override
    //     // of the code for alp12.  Found/fixed on 2015-12-19.
    //     // These tests rely on Geodesic.inverse_line, which is not supported by
    //     // geographiclib-rs as of 2021/01/18.
    // }

    // #[test]
    // fn test_std_geodesic_geodsolve69() {
    //     // Check for InverseLine if line is slightly west of S and that s13 is
    //     // correctly set.
    //     // These tests rely on Geodesic.inverse_line, which is not supported by
    //     // geographiclib-rs as of 2021/01/18.
    // }

    // #[test]
    // fn test_std_geodesic_geodsolve71() {
    //     // Check that DirectLine sets s13.
    //     // These tests rely on Geodesic.direct_line, which is not supported by
    //     // geographiclib-rs as of 2021/01/18.
    // }

    #[test]
    fn test_std_geodesic_geodsolve73() {
        // Check for backwards from the pole bug reported by Anon on 2016-02-13.
        // This only affected the Java implementation.  It was introduced in Java
        // version 1.44 and fixed in 1.46-SNAPSHOT on 2016-01-17.
        // Also the + sign on azi2 is a check on the normalizing of azimuths
        // (converting -0.0 to +0.0).
        let geod = Geodesic::wgs84();
        let (lat2, lon2, azi2) = geod.direct(90.0, 10.0, 180.0, -1e6);
        assert_relative_eq!(lat2, 81.04623, epsilon = 0.5e-5);
        assert_relative_eq!(lon2, -170.0, epsilon = 0.5e-5);
        assert_relative_eq!(azi2, 0.0, epsilon = 0.5e-5);
        assert!(azi2.is_sign_positive());
    }

    #[test]
    fn test_std_geodesic_geodsolve74() {
        // Check fix for inaccurate areas, bug introduced in v1.46, fixed
        // 2015-10-16.
        let geod = Geodesic::wgs84();
        let (a12, s12, azi1, azi2, m12, M12, M21, S12) =
            geod._gen_inverse_azi::<All>(54.1589, 15.3872, 54.1591, 15.3877);
        assert_relative_eq!(azi1, 55.723110355, epsilon = 5e-9);
        assert_relative_eq!(azi2, 55.723515675, epsilon = 5e-9);
        assert_relative_eq!(s12, 39.527686385, epsilon = 5e-9);
        assert_relative_eq!(a12, 0.000355495, epsilon = 5e-9);
        assert_relative_eq!(m12, 39.527686385, epsilon = 5e-9);
        assert_relative_eq!(M12, 0.999999995, epsilon = 5e-9);
        assert_relative_eq!(M21, 0.999999995, epsilon = 5e-9);
        assert_relative_eq!(S12, 286698586.30197, epsilon = 5e-4);
    }

    #[test]
    fn test_std_geodesic_geodsolve76() {
        // The distance from Wellington and Salamanca (a classic failure of
        // Vincenty)
        let geod = Geodesic::wgs84();
        let (s12, azi1, azi2, _a12) = geod.inverse(
            -(41.0 + 19.0 / 60.0),
            174.0 + 49.0 / 60.0,
            40.0 + 58.0 / 60.0,
            -(5.0 + 30.0 / 60.0),
        );
        assert_relative_eq!(azi1, 160.39137649664, epsilon = 0.5e-11);
        assert_relative_eq!(azi2, 19.50042925176, epsilon = 0.5e-11);
        assert_relative_eq!(s12, 19960543.857179, epsilon = 0.5e-6);
    }

    #[test]
    fn test_std_geodesic_geodsolve78() {
        // An example where the NGS calculator fails to converge
        let geod = Geodesic::wgs84();
        let (s12, azi1, azi2, _a12) = geod.inverse(27.2, 0.0, -27.1, 179.5);
        assert_relative_eq!(azi1, 45.82468716758, epsilon = 0.5e-11);
        assert_relative_eq!(azi2, 134.22776532670, epsilon = 0.5e-11);
        assert_relative_eq!(s12, 19974354.765767, epsilon = 0.5e-6);
    }

    #[test]
    fn test_std_geodesic_geodsolve80() {
        // Some tests to add code coverage: computing scale in special cases + zero
        // length geodesic (includes GeodSolve80 - GeodSolve83).
        let geod = Geodesic::wgs84();
        let (_a12, _s12, _salp1, _calp1, _salp2, _calp2, _m12, M12, M21, _S12) =
            geod._gen_inverse::<GeodesicScale<Empty>>(0.0, 0.0, 0.0, 90.0);
        assert_relative_eq!(M12, -0.00528427534, epsilon = 0.5e-10);
        assert_relative_eq!(M21, -0.00528427534, epsilon = 0.5e-10);

        let (_a12, _s12, _salp1, _calp1, _salp2, _calp2, _m12, M12, M21, _S12) =
            geod._gen_inverse::<GeodesicScale<Empty>>(0.0, 0.0, 1e-6, 1e-6);
        assert_relative_eq!(M12, 1.0, epsilon = 0.5e-10);
        assert_relative_eq!(M21, 1.0, epsilon = 0.5e-10);

        let (a12, s12, azi1, azi2, m12, M12, M21, S12) =
            geod._gen_inverse_azi::<All>(20.001, 0.0, 20.001, 0.0);
        assert_relative_eq!(a12, 0.0, epsilon = 1e-13);
        assert_relative_eq!(s12, 0.0, epsilon = 1e-8);
        assert_relative_eq!(azi1, 180.0, epsilon = 1e-13);
        assert_relative_eq!(azi2, 180.0, epsilon = 1e-13);
        assert_relative_eq!(m12, 0.0, epsilon = 1e-8);
        assert_relative_eq!(M12, 1.0, epsilon = 1e-15);
        assert_relative_eq!(M21, 1.0, epsilon = 1e-15);
        assert_relative_eq!(S12, 0.0, epsilon = 1e-10);

        let (a12, s12, azi1, azi2, m12, M12, M21, S12) =
            geod._gen_inverse_azi::<All>(90.0, 0.0, 90.0, 180.0);
        assert_relative_eq!(a12, 0.0, epsilon = 1e-13);
        assert_relative_eq!(s12, 0.0, epsilon = 1e-8);
        assert_relative_eq!(azi1, 0.0, epsilon = 1e-13);
        assert_relative_eq!(azi2, 180.0, epsilon = 1e-13);
        assert_relative_eq!(m12, 0.0, epsilon = 1e-8);
        assert_relative_eq!(M12, 1.0, epsilon = 1e-15);
        assert_relative_eq!(M21, 1.0, epsilon = 1e-15);
        assert_relative_eq!(S12, 127516405431022.0, epsilon = 0.5);

        // An incapable line which can't take distance as input
        let line = GeodesicLine::new(&geod, 1.0, 2.0, 90.0, Some(caps::LATITUDE), None, None);
        let (a12, _lat2, _lon2, _azi2, _s12, _m12, _M12, _M21, _S12) =
            line._gen_position(false, 1000.0, caps::CAP_NONE);
        assert!(a12.is_nan());
    }

    #[test]
    fn test_std_geodesic_geodsolve84() {
        // Tests for python implementation to check fix for range errors with
        // {fmod,sin,cos}(inf) (includes GeodSolve84 - GeodSolve91).
        let geod = Geodesic::wgs84();
        let (lat2, lon2, azi2) = geod.direct(0.0, 0.0, 90.0, f64::INFINITY);
        assert!(lat2.is_nan());
        assert!(lon2.is_nan());
        assert!(azi2.is_nan());
        let (lat2, lon2, azi2) = geod.direct(0.0, 0.0, 90.0, f64::NAN);
        assert!(lat2.is_nan());
        assert!(lon2.is_nan());
        assert!(azi2.is_nan());
        let (lat2, lon2, azi2) = geod.direct(0.0, 0.0, f64::INFINITY, 1000.0);
        assert!(lat2.is_nan());
        assert!(lon2.is_nan());
        assert!(azi2.is_nan());
        let (lat2, lon2, azi2) = geod.direct(0.0, 0.0, f64::NAN, 1000.0);
        assert!(lat2.is_nan());
        assert!(lon2.is_nan());
        assert!(azi2.is_nan());
        let (lat2, lon2, azi2) = geod.direct(0.0, f64::INFINITY, 90.0, 1000.0);
        assert_eq!(lat2, 0.0);
        assert!(lon2.is_nan());
        assert_eq!(azi2, 90.0);
        let (lat2, lon2, azi2) = geod.direct(0.0, f64::NAN, 90.0, 1000.0);
        assert_eq!(lat2, 0.0);
        assert!(lon2.is_nan());
        assert_eq!(azi2, 90.0);
        let (lat2, lon2, azi2) = geod.direct(f64::INFINITY, 0.0, 90.0, 1000.0);
        assert!(lat2.is_nan());
        assert!(lon2.is_nan());
        assert!(azi2.is_nan());
        let (lat2, lon2, azi2) = geod.direct(f64::NAN, 0.0, 90.0, 1000.0);
        assert!(lat2.is_nan());
        assert!(lon2.is_nan());
        assert!(azi2.is_nan());
    }

    // *_geodtest_* tests are based on Karney's GeodTest*.dat test datasets.
    // A description of these files' content can be found at:
    //     https://geographiclib.sourceforge.io/html/geodesic.html#testgeod
    // Here are some key excerpts...
    //    This consists of a set of geodesics for the WGS84 ellipsoid.
    //     Each line of the test set gives 10 space delimited numbers
    //         latitude at point 1, lat1 (degrees, exact)
    //         longitude at point 1, lon1 (degrees, always 0)
    //         azimuth at point 1, azi1 (clockwise from north in degrees, exact)
    //         latitude at point 2, lat2 (degrees, accurate to 10−18 deg)
    //         longitude at point 2, lon2 (degrees, accurate to 10−18 deg)
    //         azimuth at point 2, azi2 (degrees, accurate to 10−18 deg)
    //         geodesic distance from point 1 to point 2, s12 (meters, exact)
    //         arc distance on the auxiliary sphere, a12 (degrees, accurate to 10−18 deg)
    //         reduced length of the geodesic, m12 (meters, accurate to 0.1 pm)
    //         the area under the geodesic, S12 (m2, accurate to 1 mm2)

    static FULL_TEST_PATH: &str = "test_fixtures/test_data_unzipped/GeodTest.dat";
    static SHORT_TEST_PATH: &str = "test_fixtures/test_data_unzipped/GeodTest-short.dat";
    static BUILTIN_TEST_PATH: &str = "test_fixtures/GeodTest-100.dat";
    fn test_input_path() -> &'static str {
        if cfg!(feature = "test_full") {
            FULL_TEST_PATH
        } else if cfg!(feature = "test_short") {
            SHORT_TEST_PATH
        } else {
            BUILTIN_TEST_PATH
        }
    }

    fn geodtest_basic<T>(path: &str, f: T)
    where
        T: Fn(usize, &(f64, f64, f64, f64, f64, f64, f64, f64, f64, f64)),
    {
        let dir_base = std::env::current_dir().expect("Failed to determine current directory");
        let path_base = dir_base.as_path();
        let pathbuf = std::path::Path::new(path_base).join(path);
        let path = pathbuf.as_path();
        let file = match std::fs::File::open(path) {
            Ok(val) => val,
            Err(_error) => {
                let path_str = path
                    .to_str()
                    .expect("Failed to convert GeodTest path to string during error reporting");
                panic!("Failed to open test input file. Run `script/download-test-data.sh` to download test input to: {}\nFor details see https://geographiclib.sourceforge.io/html/geodesic.html#testgeod", path_str)
            }
        };
        let reader = std::io::BufReader::new(file);
        reader.lines().enumerate().for_each(|(i, line)| {
            let line_safe = line.expect("Failed to read line");
            let items: Vec<f64> = line_safe
                .split(' ')
                .enumerate()
                .map(|(j, item)| match item.parse::<f64>() {
                    Ok(parsed) => parsed,
                    Err(_error) => {
                        panic!("Error parsing item {} on line {}: {}", j + 1, i + 1, item)
                    }
                })
                .collect();
            assert_eq!(items.len(), 10);
            let tuple = (
                items[0], items[1], items[2], items[3], items[4], items[5], items[6], items[7],
                items[8], items[9],
            );
            f(i + 1, &tuple); // report 1-based line number rather than 0-based
        });
    }

    #[test]
    fn test_geodtest_geodesic_direct12() {
        let g = std::sync::Arc::new(std::sync::Mutex::new(Geodesic::wgs84()));

        geodtest_basic(
            test_input_path(),
            |_line_num, &(lat1, lon1, azi1, lat2, lon2, azi2, s12, a12, m12, S12)| {
                let g = g.lock().unwrap();
                let (lat2_out, lon2_out, azi2_out, m12_out, _M12_out, _M21_out, S12_out, a12_out) =
                    g.direct(lat1, lon1, azi1, s12);
                assert_relative_eq!(lat2, lat2_out, epsilon = 1e-13);
                assert_relative_eq!(lon2, lon2_out, epsilon = 2e-8);
                assert_relative_eq!(azi2, azi2_out, epsilon = 2e-8);
                assert_relative_eq!(m12, m12_out, epsilon = 9e-9);
                assert_relative_eq!(S12, S12_out, epsilon = 2e4); // Note: unreasonable tolerance
                assert_relative_eq!(a12, a12_out, epsilon = 9e-14);
            },
        );
    }

    #[test]
    fn test_geodtest_geodesic_direct21() {
        let g = std::sync::Arc::new(std::sync::Mutex::new(Geodesic::wgs84()));

        geodtest_basic(
            test_input_path(),
            |_line_num, &(lat1, lon1, azi1, lat2, lon2, azi2, s12, a12, m12, S12)| {
                let g = g.lock().unwrap();
                // Reverse some values for 2->1 instead of 1->2
                let (lat1, lon1, azi1, lat2, lon2, azi2, s12, a12, m12, S12) =
                    (lat2, lon2, azi2, lat1, lon1, azi1, -s12, -a12, -m12, -S12);
                let (lat2_out, lon2_out, azi2_out, m12_out, _M12_out, _M21_out, S12_out, a12_out) =
                    g.direct(lat1, lon1, azi1, s12);
                assert_relative_eq!(lat2, lat2_out, epsilon = 8e-14);
                assert_relative_eq!(lon2, lon2_out, epsilon = 4e-6);
                assert_relative_eq!(azi2, azi2_out, epsilon = 4e-6);
                assert_relative_eq!(m12, m12_out, epsilon = 1e-8);
                assert_relative_eq!(S12, S12_out, epsilon = 3e6); // Note: unreasonable tolerance
                assert_relative_eq!(a12, a12_out, epsilon = 9e-14);
            },
        );
    }

    #[test]
    fn test_geodtest_geodesic_inverse12() {
        let g = std::sync::Arc::new(std::sync::Mutex::new(Geodesic::wgs84()));

        geodtest_basic(
            test_input_path(),
            |line_num, &(lat1, lon1, azi1, lat2, lon2, azi2, s12, a12, m12, S12)| {
                let g = g.lock().unwrap();
                let (s12_out, azi1_out, azi2_out, m12_out, _M12_out, _M21_out, S12_out, a12_out) =
                    g.inverse(lat1, lon1, lat2, lon2);
                assert_relative_eq!(s12, s12_out, epsilon = 8e-9);
                assert_relative_eq!(azi1, azi1_out, epsilon = 2e-2);
                assert_relative_eq!(azi2, azi2_out, epsilon = 2e-2);
                assert_relative_eq!(m12, m12_out, epsilon = 5e-5);
                // Our area calculation differs significantly (~1e7) from the value in GeodTest.dat for
                // line 400001, BUT our value also perfectly matches the value returned by GeographicLib
                // (C++) 1.51. Here's the problem line, for reference:
                // 4.199535552987 0 90 -4.199535552987 179.398106343454992238 90 19970505.608097404994 180 0
                if line_num != 400001 {
                    assert_relative_eq!(S12, S12_out, epsilon = 3e10); // Note: unreasonable tolerance
                }
                assert_relative_eq!(a12, a12_out, epsilon = 2e-10);
            },
        );
    }

    #[test]
    fn test_turnaround() {
        let g = Geodesic::wgs84();

        let start = (0.0, 0.0);
        let destination = (0.0, 1.0);

        let (distance, azi1, _, _) = g.inverse(start.0, start.1, destination.0, destination.1);

        // Confirm that we've gone due-east
        assert_eq!(azi1, 90.0);

        // Turn around by adding 180 degrees to the azimuth
        let turn_around = azi1 + 180.0;

        // Confirm that turn around is due west
        assert_eq!(turn_around, 270.0);

        // Test that we can turn around and get back to the starting point.
        let (lat, lon) = g.direct(destination.0, destination.1, turn_around, distance);
        assert_relative_eq!(lat, start.0, epsilon = 1.0e-3);
        assert_relative_eq!(lon, start.1, epsilon = 1.0e-3);
    }
}
