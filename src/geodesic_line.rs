#![allow(non_snake_case)]

use crate::geodesic::{self};
use crate::traits::{All};
use crate::cached_weights::{C1Coeff,C2Coeff,C1pCoeff};
use crate::geodesic_capability as caps;
use crate::geomath;
use crate::internals::constants::{TINY};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct GeodesicLine<'a>{
    geod: &'a geodesic::Geodesic,
    eps: f64,
    _B11: f64,
    _calp0: f64,
    _csig1: f64,
    _comg1: f64,
    _ctau1: f64,
    _dn1: f64,
    _k2: f64,
    _salp0: f64,
    _somg1: f64,
    _ssig1: f64,
    _stau1: f64,
    azi1: f64,
    calp1: f64,
    caps: u64,
    lat1: f64,
    lon1: f64,
    salp1: f64,
}

impl<'a> GeodesicLine<'a> {
    pub fn new(
        geod: &'a geodesic::Geodesic,
        lat1: f64,
        lon1: f64,
        azi1: f64,
        caps: Option<u64>,
        salp1: Option<f64>,
        calp1: Option<f64>,
    ) -> Self {
        let caps = match caps {
            None => caps::STANDARD | caps::DISTANCE_IN,
            Some(caps) => caps,
        };
        let salp1 = match salp1 {
            None => f64::NAN,
            Some(salp1) => salp1,
        };
        let calp1 = match calp1 {
            None => f64::NAN,
            Some(calp1) => calp1,
        };

        // This was taken from geodesic, putting it here for convenience

        let caps = caps | caps::LATITUDE | caps::AZIMUTH | caps::LONG_UNROLL;
        let (azi1, salp1, calp1) = if salp1.is_nan() || calp1.is_nan() {
            let azi1 = geomath::ang_normalize(azi1);
            let (salp1, calp1) = geomath::sincosd(geomath::ang_round(azi1));
            (azi1, salp1, calp1)
        } else {
            (azi1, salp1, calp1)
        };
        let lat1 = geomath::lat_fix(lat1);

        let (sbet1, cbet1) = geod.sincosd_for_ellipsoid(geomath::ang_round(lat1));
        let _dn1 = (1.0 + geod._ep2 * sbet1.powi(2)).sqrt();
        let _salp0 = salp1 * cbet1;
        let _calp0 = calp1.hypot(salp1 * sbet1);
        let mut _ssig1 = sbet1;
        let _somg1 = _salp0 * sbet1;
        let mut _csig1 = if sbet1 != 0.0 || calp1 != 0.0 {
            cbet1 * calp1
        } else {
            1.0
        };
        let _comg1 = _csig1;
        geomath::norm(&mut _ssig1, &mut _csig1);
        let (_k2, eps) = geod.local_curvature(_calp0);
        let mut _B11 = 0.0;
        let mut _stau1 = 0.0;
        let mut _ctau1 = 0.0;
        if caps & caps::CAP_C1 != 0 {
            _B11 = geod.weights.calc_single_bxf::<All,C1Coeff>(eps,_ssig1, _csig1);
            let s = _B11.sin();
            let c = _B11.cos();
            _stau1 = _ssig1 * c + _csig1 * s;
            _ctau1 = _csig1 * c - _ssig1 * s;
        }


        GeodesicLine {
            geod,
            _B11,
            _comg1,
            _calp0,
            _csig1,
            _ctau1,
            _dn1,
            _k2,
            _salp0,
            _somg1,
            _ssig1,
            _stau1,
            azi1,
            calp1,
            caps,
            lat1,
            lon1,
            salp1,
            eps,
        }
    }

    /// returns (a12, lat2, lon2, azi2, s12, m12, M12, M21, S12)
    pub fn _gen_position(
        &self,
        arcmode: bool,
        s12_a12: f64,
        outmask: u64,
    ) -> (f64, f64, f64, f64, f64, f64, f64, f64, f64) {
        let mut a12 = f64::NAN;
        let mut lat2 = f64::NAN;
        let mut lon2 = f64::NAN;
        let mut azi2 = f64::NAN;
        let mut s12 = f64::NAN;
        let mut m12 = f64::NAN;
        let mut M12 = f64::NAN;
        let mut M21 = f64::NAN;
        let mut S12 = f64::NAN;
        let outmask = outmask & (self.caps & caps::OUT_MASK);
        if !(arcmode || (self.caps & (caps::OUT_MASK & caps::DISTANCE_IN) != 0)) {
            return (a12, lat2, lon2, azi2, s12, m12, M12, M21, S12);
        }



        let mut B12 = 0.0;
        let mut AB1 = 0.0;
        let mut sig12: f64;
        let mut ssig12: f64;
        let mut csig12: f64;
        let mut ssig2: f64;
        let mut csig2: f64;

        let a1 = geomath::_A1m1f(self.eps);
        let a2 = geomath::_A2m1f(self.eps);
        let m0x = a1 - a2;
        let a1 = a1 + 1.0;
        let a2 = a2 + 1.0;

        if arcmode {
            sig12 = s12_a12.to_radians();
            let res = geomath::sincosd(s12_a12);
            ssig12 = res.0;
            csig12 = res.1;
        } else {
            let tau12 = s12_a12 / (self.geod._b * a1);

            let s = tau12.sin();
            let c = tau12.cos();

            B12 = -self.geod.weights.calc_single_bxf::<All,C1pCoeff>(
                self.eps,
                self._stau1 * c + self._ctau1 * s,
                self._ctau1 * c - self._stau1 * s);
            sig12 = tau12 - (B12 - self._B11);
            ssig12 = sig12.sin();
            csig12 = sig12.cos();
            if self.geod.f.abs() > 0.01 {
                ssig2 = self._ssig1 * csig12 + self._csig1 * ssig12;
                csig2 = self._csig1 * csig12 - self._ssig1 * ssig12;
                B12 = self.geod.weights.calc_single_bxf::<All,C1Coeff>(self.eps,ssig2, csig2);
                let serr = a1 * (sig12 + (B12 - self._B11)) - s12_a12 / self.geod._b;
                sig12 -= serr / (1.0 + self._k2 * ssig2.powi(2)).sqrt();
                ssig12 = sig12.sin();
                csig12 = sig12.cos();
            }
        };
        ssig2 = self._ssig1 * csig12 + self._csig1 * ssig12;
        csig2 = self._csig1 * csig12 - self._ssig1 * ssig12;
        let dn2 = (1.0 + self._k2 * ssig2.powi(2)).sqrt();
        if outmask & (caps::DISTANCE | caps::REDUCEDLENGTH | caps::GEODESICSCALE) != 0 {
            if arcmode || self.geod.f.abs() > 0.01 {
                B12 = self.geod.weights.calc_single_bxf::<All,C1Coeff>(self.eps,ssig2, csig2);
            }
            AB1 = a1 * (B12 - self._B11);
        }

        let sbet2 = self._calp0 * ssig2;
        let mut cbet2 = self._salp0.hypot(self._calp0 * csig2);
        if cbet2 == 0.0 {
            cbet2 = TINY;
            csig2 = TINY;
        }
        let salp2 = self._salp0;
        let calp2 = self._calp0 * csig2;
        if outmask & caps::DISTANCE != 0 {
            s12 = if arcmode {
                self.geod._b * (a1 * sig12 + AB1)
            } else {
                s12_a12
            }
        }
        if outmask & caps::LONGITUDE != 0 {
            let somg2 = self._salp0 * ssig2;
            let comg2 = csig2;
            let omg12 = if outmask & caps::LONG_UNROLL != 0 {
                let E = 1.0_f64.copysign(self._salp0);
                E * (sig12 - (ssig2.atan2(csig2) - self._ssig1.atan2(self._csig1))
                    + ((E * somg2).atan2(comg2) - (E * self._somg1).atan2(self._comg1)))
            } else {
                (somg2 * self._comg1 - comg2 * self._somg1)
                    .atan2(comg2 * self._comg1 + somg2 * self._somg1)
            };

            let a3c = -self.geod.f * self._salp0 * self.geod._A3f(self.eps);
            let lam12 = omg12 + a3c * (sig12 + self.geod.weights.c3x_difference_of_meridian_arc_lengths(self.eps, self._ssig1, self._csig1, ssig2, csig2));
            let lon12 = lam12.to_degrees();
            lon2 = if outmask & caps::LONG_UNROLL != 0 {
                self.lon1 + lon12
            } else {
                geomath::ang_normalize(geomath::ang_normalize(self.lon1) + geomath::ang_normalize(lon12))
            };
        };

        if outmask & caps::LATITUDE != 0 {
            lat2 = geomath::atan2d(sbet2, self.geod._f1 * cbet2);
        }
        if outmask & caps::AZIMUTH != 0 {
            azi2 = geomath::atan2d(salp2, calp2);
        }
        if outmask & (caps::REDUCEDLENGTH | caps::GEODESICSCALE) != 0 {
            let AB2 = a2 * self.geod.weights.calc_bxf::<All,C2Coeff>(self.eps,self._ssig1,self._csig1,ssig2,csig2);
            let J12 = m0x * sig12 + (AB1 - AB2);
            if outmask & caps::REDUCEDLENGTH != 0 {
                m12 = self.geod._b
                    * ((dn2 * (self._csig1 * ssig2) - self._dn1 * (self._ssig1 * csig2))
                        - self._csig1 * csig2 * J12);
            }
            if outmask & caps::GEODESICSCALE != 0 {
                let t = self._k2 * (ssig2 - self._ssig1) * (ssig2 + self._ssig1) / (self._dn1 + dn2);
                M12 = csig12 + (t * ssig2 - csig2 * J12) * self._ssig1 / self._dn1;
                M21 = csig12 - (t * self._ssig1 - self._csig1 * J12) * ssig2 / dn2;
            }
        }
        if outmask & caps::AREA != 0 {
            S12 = area_calc(
                &self.geod,
                self.eps,
                self._salp0,
                self._calp0,
                self.salp1,
                self.calp1,
                salp2,
                calp2,
                self._ssig1,
                self._csig1,
                ssig2,
                csig2,
                ssig12,
                csig12,
            );
            /*
            let salp12: f64;
            let calp12: f64;
            if self._calp0 == 0.0 || self._salp0 == 0.0 {
                salp12 = salp2 * self.calp1 - calp2 * self.salp1;
                calp12 = calp2 * self.calp1 + salp2 * self.salp1;
            } else {
                salp12 = self._calp0
                    * self._salp0
                    * (if csig12 <= 0.0 {
                        self._csig1 * (1.0 - csig12) + ssig12 * self._ssig1
                    } else {
                        ssig12 * (self._csig1 * ssig12 / (1.0 + csig12) + self._ssig1)
                    });
                calp12 = self._salp0.powi(2) + self._calp0.powi(2) * self._csig1 * csig2;
            }
            let diff = self.geod.weights.c4x_difference(self.eps, self._ssig1, self._csig1, ssig2, csig2);
            let a4 = self.geod.a.powi(2) * self._calp0 * self._salp0 * self.geod._e2;
            S12 = self.geod._c2 * salp12.atan2(calp12) + a4 * diff;
            */
        }
        a12 = if arcmode { s12_a12 } else { sig12.to_degrees() };
        (a12, lat2, lon2, azi2, s12, m12, M12, M21, S12)
    }

    // not currently used, but maybe some day
    #[allow(dead_code)]
    pub fn Position(&self, s12: f64, outmask: Option<u64>) -> HashMap<String, f64> {
        let outmask = match outmask {
            Some(outmask) => outmask,
            None => caps::STANDARD,
        };
        let mut result: HashMap<String, f64> = HashMap::new();
        result.insert("lat1".to_string(), self.lat1);
        result.insert("azi1".to_string(), self.azi1);
        result.insert("s12".to_string(), s12);
        let lon1 = if outmask & caps::LONG_UNROLL != 0 {
            self.lon1
        } else {
            geomath::ang_normalize(self.lon1)
        };
        result.insert("lon1".to_string(), lon1);

        let (a12, lat2, lon2, azi2, _s12, m12, M12, M21, S12) =
            self._gen_position(false, s12, outmask);
        let outmask = outmask & caps::OUT_MASK;
        result.insert("a12".to_string(), a12);
        if outmask & caps::LATITUDE != 0 {
            result.insert("lat2".to_string(), lat2);
        }
        if outmask & caps::LONGITUDE != 0 {
            result.insert("lon2".to_string(), lon2);
        }
        if outmask & caps::AZIMUTH != 0 {
            result.insert("azi2".to_string(), azi2);
        }
        if outmask & caps::REDUCEDLENGTH != 0 {
            result.insert("m12".to_string(), m12);
        }
        if outmask & caps::GEODESICSCALE != 0 {
            result.insert("M12".to_string(), M12);
            result.insert("M21".to_string(), M21);
        }
        if outmask & caps::AREA != 0 {
            result.insert("S12".to_string(), S12);
        }
        result
    }
}

pub(in crate) fn area_calc(
    geod: &geodesic::Geodesic,
    eps: f64,
    salp0: f64,
    calp0: f64,
    salp1: f64,
    calp1: f64,
    salp2: f64,
    calp2: f64,
    ssig1: f64,
    csig1: f64,
    ssig2: f64,
    csig2: f64,
    ssig12: f64,
    csig12: f64,
) -> f64 {
    let salp12: f64;
    let calp12: f64;
    if calp0 == 0.0 || salp0 == 0.0 {
        salp12 = salp2 * calp1 - calp2 * salp1;
        calp12 = calp2 * calp1 + salp2 * salp1;
    } else {
        salp12 = calp0
            * salp0
            * (if csig12 <= 0.0 {
                csig1 * (1.0 - csig12) + ssig12 * ssig1
            } else {
                ssig12 * (csig1 * ssig12 / (1.0 + csig12) + ssig1)
            });
        calp12 = salp0.powi(2) + calp0.powi(2) * csig1 * csig2;
    }
    let diff = geod.weights.c4x_difference(eps, ssig1, csig1, ssig2, csig2);
    let a4 = geod.a.powi(2) * calp0 * salp0 * geod._e2;
    geod._c2 * salp12.atan2(calp12) + a4 * diff
}

#[cfg(test)]
mod tests {
    use super::*;
    use geodesic::Geodesic;

    #[test]
    fn test_gen_position() {
        let geod = Geodesic::wgs84();
        let gl = GeodesicLine::new(&geod, 0.0, 0.0, 10.0, None, None, None);
        let res = gl._gen_position(false, 150.0, 3979);
        assert_eq!(res.0, 0.0013520059461334633);
        assert_eq!(res.1, 0.0013359451088740494);
        assert_eq!(res.2, 0.00023398621812867812);
        assert_eq!(res.3, 10.000000002727887);
        assert_eq!(res.4, 150.0);
        assert!(res.5.is_nan());
        assert!(res.6.is_nan());
        assert!(res.7.is_nan());
        assert!(res.8.is_nan());
    }

    #[test]
    fn test_init() {
        let geod = Geodesic::wgs84();
        let gl = GeodesicLine::new(&geod, 0.0, 0.0, 0.0, None, None, None);
        assert_eq!(gl.geod.a, 6378137.0);
        assert_eq!(gl.geod.f, 0.0033528106647474805);
        assert_eq!(gl.geod._b, 6356752.314245179);
        assert_eq!(gl.geod._c2, 40589732499314.76);
        assert_eq!(gl.geod._f1, 0.9966471893352525);
        assert_eq!(gl.caps, 36747);
        assert_eq!(gl.lat1, 0.0);
        assert_eq!(gl.lon1, 0.0);
        assert_eq!(gl.azi1, 0.0);
        assert_eq!(gl.salp1, 0.0);
        assert_eq!(gl.calp1, 1.0);
        assert_eq!(gl._dn1, 1.0);
        assert_eq!(gl._salp0, 0.0);
        assert_eq!(gl._calp0, 1.0);
        assert_eq!(gl._ssig1, 0.0);
        assert_eq!(gl._somg1, 0.0);
        assert_eq!(gl._csig1, 1.0);
        assert_eq!(gl._comg1, 1.0);
        assert_eq!(gl._k2, geod._ep2);
    }
}
