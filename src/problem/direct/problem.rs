#![allow(non_snake_case)]

use crate::{
    geodesic::{Geodesic},
    cached_weights::{C1Coeff,C2Coeff,C1pCoeff},
    internals::constants::{TINY},
    traits::caps::{Caps,DistanceIn},
    problem::direct::{
        ret_value::{DirectProblemReturnValue},
        arg_value::{DirectProblemDistanceArg,DirectProblemAzimuth},
    },
    geomath::{self},
};

/// The function that solves the direct problem.
///
/// It is extremely generic (intentionally) to ensure it
/// can handle all scenarios.
///
/// The generic arguments to this function are not type
/// -safe by design. The design intention is this
/// function should under go monomorphization to generate a
/// unique body, unique to that use case. 
///
/// As while it is true to argue "a predicted branch has no
/// cost". If a value is behind a branch that limits
/// optimizations which can occur as there is still going
/// to be a phi node in the IR, which can limit
/// propigation of values and optimizations.
#[allow(dead_code)]
#[inline(always)]
pub fn direct_problem<A,D,R>(
    geod: &Geodesic,
    lat1: f64,
    lon1: f64,
    azi: A,
    dist: D,
) -> R
where
    R: DirectProblemReturnValue + Sized,
    D: DirectProblemDistanceArg + Sized,
    A: DirectProblemAzimuth + Sized,
{
    use crate::traits::caps::{
        Latitude as CapLat,
        Longitude as CapLon,
        Azimuth as CapAzi,
        LongUnroll as CapUnroll,
    };
    /*
     * Apply several defaults
     *
     * In reality this is about the return value it is more
     * we need those flags set to calculate useful information
     * So we force set them.
     *
     */
    direct_problem_core::<A,D,R,CapUnroll<CapAzi<CapLon<CapLat<R::Features>>>>>(geod, lat1, lon1, azi, dist)
}

#[allow(dead_code)]
#[inline(always)]
fn direct_problem_core<A,D,R,C>(
    geod: &Geodesic,
    lat1: f64,
    lon1: f64,
    azi: A,
    dist: D,
) -> R
where
    C: Caps,
    R: DirectProblemReturnValue + Sized,
    D: DirectProblemDistanceArg + Sized,
    A: DirectProblemAzimuth + Sized,
{

    let (salp1, calp1) = azi.get_angle_info();
    let lat1 = geomath::lat_fix(lat1);
    let (sbet1, cbet1) = geod.sincosd_for_ellipsoid(geomath::ang_round(lat1));

    let dn1 = (1.0 + geod._ep2 * sbet1.powi(2)).sqrt();
    let salp0 = salp1 * cbet1;
    let calp0 = calp1.hypot(salp1 * sbet1);
    let mut ssig1 = sbet1;
    let _somg1 = salp0 * sbet1;
    let mut csig1 = if sbet1 != 0.0 || calp1 != 0.0 {
        cbet1 * calp1
    } else {
        1.0
    };
    let _comg1 = csig1;
    geomath::norm(&mut ssig1, &mut csig1);
    let (k2, eps) = geod.local_curvature(calp0);

    let mut _B11 = 0.0;
    let mut _stau1 = 0.0;
    let mut _ctau1 = 0.0;
    if C::C1 {
        _B11 = geod.weights.calc_single_bxf::<C,C1Coeff>(eps,ssig1, csig1);
        let s = _B11.sin();
        let c = _B11.cos();
        _stau1 = ssig1 * c + csig1 * s;
        _ctau1 = csig1 * c - ssig1 * s;
    }

    let mut ret: R = R::default();
    
    let mut B12 = 0.0;
    let mut AB1 = 0.0;
    let mut sig12: f64;
    let mut ssig12: f64;
    let mut csig12: f64;
    let mut ssig2: f64;
    let mut csig2: f64;
    
    let a1 = geomath::_A1m1f(eps);
    let a2 = geomath::_A2m1f(eps);
    let m0x = a1 - a2;
    let a1 = a1 + 1.0;
    let a2 = a2 + 1.0;
    
    if D::ARC_MODE {
        sig12 = dist.get_a12().to_radians();
        let res = geomath::sincosd(dist.get_a12());
        ssig12 = res.0;
        csig12 = res.1;
    } else {
        let tau12 = dist.get_s12() / (geod._b * a1);
    
        let s = tau12.sin();
        let c = tau12.cos();
    
        B12 = -geod.weights.calc_single_bxf::<DistanceIn<C>,C1pCoeff>(
            eps,
            _stau1 * c + _ctau1 * s,
            _ctau1 * c - _stau1 * s);
        sig12 = tau12 - (B12 - _B11);
        ssig12 = sig12.sin();
        csig12 = sig12.cos();
        if geod.f.abs() > 0.01 {
            ssig2 = ssig1 * csig12 + csig1 * ssig12;
            csig2 = csig1 * csig12 - ssig1 * ssig12;
            B12 = geod.weights.calc_single_bxf::<C,C1Coeff>(eps,ssig2, csig2);
            let serr = a1 * (sig12 + (B12 - _B11)) - dist.get_s12() / geod._b;
            sig12 -= serr / (1.0 + k2 * ssig2.powi(2)).sqrt();
            ssig12 = sig12.sin();
            csig12 = sig12.cos();
        }
    };
    ssig2 = ssig1 * csig12 + csig1 * ssig12;
    csig2 = csig1 * csig12 - ssig1 * ssig12;
    let dn2 = (1.0 + k2 * ssig2.powi(2)).sqrt();
    if C::DISTANCE | C::REDUCEDLENGTH | C::GEODESICSCALE {
        if D::ARC_MODE || geod.f.abs() > 0.01 {
            B12 = geod.weights.calc_single_bxf::<C,C1Coeff>(eps,ssig2, csig2);
        }
        AB1 = a1 * (B12 - _B11);
    }
    
    let sbet2 = calp0 * ssig2;
    let mut cbet2 = salp0.hypot(calp0 * csig2);
    if cbet2 == 0.0 {
        cbet2 = TINY;
        csig2 = TINY;
    }
    let salp2 = salp0;
    let calp2 = calp0 * csig2;
    if C::DISTANCE {
        ret.set_s12(
            if D::ARC_MODE {
                geod._b * (a1 * sig12 + AB1)
            } else {
                dist.get_s12()
            }
        );
    }
    if C::LONGITUDE {
        let somg2 = salp0 * ssig2;
        let comg2 = csig2;
        let omg12 = if C::LONG_UNROLL {
            let E = 1.0_f64.copysign(salp0);
            E * (sig12 - (ssig2.atan2(csig2) - ssig1.atan2(csig1))
                + ((E * somg2).atan2(comg2) - (E * _somg1).atan2(_comg1)))
        } else {
            (somg2 * _comg1 - comg2 * _somg1)
                .atan2(comg2 * _comg1 + somg2 * _somg1)
        };
    
        let a3c = -geod.f * salp0 * geod._A3f(eps);
        let lam12 = omg12 + a3c * (sig12 + geod.weights.c3x_difference_of_meridian_arc_lengths(eps, ssig1, csig1, ssig2, csig2));
        let lon12 = lam12.to_degrees();
        ret.set_lon2(
            if C::LONG_UNROLL {
                lon1 + lon12
            } else {
                geomath::ang_normalize(geomath::ang_normalize(lon1) + geomath::ang_normalize(lon12))
            }
        );
    };
    
    if C::LATITUDE {
        ret.set_lat2(geomath::atan2d(sbet2, geod._f1 * cbet2));
    }
    if C::AZIMUTH {
        ret.set_azi2(geomath::atan2d(salp2, calp2));
    }
    if C::REDUCEDLENGTH | C::GEODESICSCALE {
        let AB2 = a2 * geod.weights.calc_bxf::<C,C2Coeff>(eps,ssig1,csig1,ssig2,csig2);
        let J12 = m0x * sig12 + (AB1 - AB2);
        if C::REDUCEDLENGTH {
            ret.set_m12(geod._b
                * ((dn2 * (csig1 * ssig2) - dn1 * (ssig1 * csig2))
                    - csig1 * csig2 * J12));
        }
        if C::GEODESICSCALE {
            let t = k2 * (ssig2 - ssig1) * (ssig2 + ssig1) / (dn1 + dn2);
            ret.set_M12(csig12 + (t * ssig2 - csig2 * J12) * ssig1 / dn1);
            ret.set_M21(csig12 - (t * ssig1 - csig1 * J12) * ssig2 / dn2);
        }
    }
    if C::AREA {
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
        ret.set_S12(geod._c2 * salp12.atan2(calp12) + a4 * diff)
    }

    ret.set_a12(
        if C::ARC_MODE { 
            dist.get_a12()
        } else {
            sig12.to_degrees() 
        }
    );

    ret
}

#[test]
fn test_direct_100points() {
    use approx::assert_relative_eq;

    use crate::test_data::{
        TestData,DirectTest,
        geod_test_100::GeodTest_100,
    };
    use crate::problem::direct::ret_value::{
        LatLon,Azi,ArcDist,M12,Area,Dist,
    };
    use crate::problem::direct::arg_value::AzimuthOnly;
    use crate::traits::caps::{
        Empty,
    };


    let earth = Geodesic::wgs84();

    for point in GeodTest_100.iter() {
        let (v1,v2,v3,_) = point.direct_tests();

        let out: Dist<Azi<ArcDist<M12<Area<LatLon>>>>> = direct_problem(earth, v1.lat1, v1.lon1, v1.azi, v1.dist);
        assert_relative_eq!(v1.lon2, out.get_lon2(), epsilon = 1e-8f64);
        assert_relative_eq!(v1.lat2, out.get_lat2(), epsilon = 1e-8f64);
        assert_relative_eq!(v1.azi2, out.get_azi2(), epsilon = 1e-8f64);
        assert_relative_eq!(v1.s12,  out.get_s12(),  epsilon = 1e-8f64);
        assert_relative_eq!(v1.a12,  out.get_a12(),  epsilon = 1e-8f64);
        assert_relative_eq!(v1.m12,  out.get_m12(),  epsilon = 1e-8f64);

        let out: Dist<Azi<ArcDist<M12<Area<LatLon>>>>> = direct_problem(earth, v3.lat1, v3.lon1, v3.azi, v3.dist);
        assert_relative_eq!(v3.lon2, out.get_lon2(), epsilon = 1e-8f64);
        assert_relative_eq!(v3.lat2, out.get_lat2(), epsilon = 1e-8f64);
        assert_relative_eq!(v3.azi2, out.get_azi2(), epsilon = 1e-8f64);
        assert_relative_eq!(v3.s12,  out.get_s12(),  epsilon = 1e-8f64);
        assert_relative_eq!(v3.a12,  out.get_a12(),  epsilon = 1e-8f64);
        assert_relative_eq!(v3.m12,  out.get_m12(),  epsilon = 1e-8f64);
    }
}

