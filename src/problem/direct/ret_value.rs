#![allow(non_snake_case)]

use crate::traits::caps::{
    Caps,
    Empty,
    Latitude as CapLat,
    Longitude as CapLon,
    Azimuth as CapAzi,
    ReducedLength as CapRedLen,
    GeodesicScale as CapScale,
    Area as CapArea,
    Distance as CapDist,
};

/// This type is used to control what data is returned from
/// the direct problem.
///
/// The "Direct Geodesic Problem" can be defined roughly
///
/// > I am point (lat1, lon1). I begin travelling at bearing (azi1)
/// > for distance (s12, in metres). Where do I arrive?
///
/// The "arrival location" is defined by (lat2,lon2), with the
/// reverse azimuth (azi2) being the direction you'd travel to
/// reverse the journey. It should be noted this library's calculations
/// are done on an oblate spheroid of rotation, not a sphere. So for
/// certain very long travel distances the opposite azimuth, may not
/// be a simple 180 degree "turn-around".
///
/// The library additionally supports this problem in Arc Mode (a12)
/// where one defines travel in degrees (a12) instead of distance (s12).
///
/// Additional returned information is of somewhat little use outside
/// of benchmarking, academic, and testing contents. Full details:
///
/// * a12: Arc Distance travelled (in degrees).
/// * s12: Distance travelled (in meters).
/// * lat2: Latitude of 2nd point.
/// * lon2: Longitude of 2nd point.
/// * azi2: azimuth of 2nd point to reverse the journey.
/// * S12: the area under the path travelled in squared meters
/// * m12/M12/M21: differential quantities used within the calculation to approximate the local curvature.
///
pub (in crate) trait DirectProblemReturnValue: Default {
    type Features: Caps;

    /// a12 is the arc distance (in degrees) between 2 points
    #[inline(always)]
    fn set_a12(&mut self, _a12: f64) { }

    /// lat2 is the latitude
    #[inline(always)]
    fn set_lat2(&mut self, _lat2: f64) { }
    #[inline(always)]
    fn set_lon2(&mut self, _lon2: f64) { }
    #[inline(always)]
    fn set_azi2(&mut self, _azi2: f64) { }
    #[inline(always)]
    fn set_s12(&mut self, _s12: f64) { }
    #[inline(always)]
    fn set_m12(&mut self, _m12: f64) { }
    #[inline(always)]
    fn set_M12(&mut self, _M12: f64) { }
    #[inline(always)]
    fn set_M21(&mut self, _M21: f64) { }
    #[inline(always)]
    fn set_S12(&mut self, _S12: f64) { }


    #[inline(always)]
    fn get_a12(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_lat2(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_lon2(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_azi2(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_s12(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_m12(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_M12(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_M21(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_S12(&self) -> f64 { f64::NAN }
}


pub(in crate) struct LatLon {
    lat2: f64,
    lon2: f64,
}
impl Default for LatLon {
    fn default() -> Self {
        Self {
            lat2: f64::NAN,
            lon2: f64::NAN,
        }
    }
}
impl DirectProblemReturnValue for LatLon {
    type Features = CapLon<CapLat<Empty>>;

    #[inline(always)]
    fn set_lat2(&mut self, lat2: f64) { self.lat2 = lat2; }
    #[inline(always)]
    fn set_lon2(&mut self, lon2: f64) { self.lon2 = lon2; }

    #[inline(always)]
    fn get_lat2(&self) -> f64 { self.lat2 }
    #[inline(always)]
    fn get_lon2(&self) -> f64 { self.lon2 }
}

pub(in crate) struct Dist<D: DirectProblemReturnValue> {
    s12: f64,
    other: D,
}
impl<D: DirectProblemReturnValue> Default for Dist<D> {
    fn default() -> Self {
        Self {
            s12: f64::NAN,
            other: D::default(),
        }
    }
}
impl<D: DirectProblemReturnValue> DirectProblemReturnValue for Dist<D> {
    type Features = CapDist<D::Features>;

    #[inline(always)]
    fn set_a12(&mut self, a12: f64) {
        self.other.set_a12(a12);
    }
    #[inline(always)]
    fn set_lat2(&mut self, lat2: f64) {
        self.other.set_lat2(lat2);
    }
    #[inline(always)]
    fn set_lon2(&mut self, lon2: f64) {
        self.other.set_lon2(lon2);
    }
    #[inline(always)]
    fn set_azi2(&mut self, azi2: f64) {
        self.other.set_azi2(azi2);
    }
    #[inline(always)]
    fn set_s12(&mut self, s12: f64) {
        self.s12 = s12;
        self.other.set_s12(s12);
    }
    #[inline(always)]
    fn set_m12(&mut self, m12: f64) {
        self.other.set_m12(m12);
    }
    #[inline(always)]
    fn set_M12(&mut self, M12: f64) {
        self.other.set_M12(M12);
    }
    #[inline(always)]
    fn set_M21(&mut self, M21: f64) {
        self.other.set_M21(M21);
    }
    #[inline(always)]
    fn set_S12(&mut self, S12: f64) {
        self.other.set_S12(S12);
    }


    #[inline(always)]
    fn get_a12(&self) -> f64 { self.other.get_a12() }
    #[inline(always)]
    fn get_lat2(&self) -> f64 { self.other.get_lat2() }
    #[inline(always)]
    fn get_lon2(&self) -> f64 { self.other.get_lon2() }
    #[inline(always)]
    fn get_azi2(&self) -> f64 { self.other.get_azi2() }
    #[inline(always)]
    fn get_s12(&self) -> f64 { self.s12 }
    #[inline(always)]
    fn get_m12(&self) -> f64 { self.other.get_m12() }
    #[inline(always)]
    fn get_M12(&self) -> f64 { self.other.get_M12() }
    #[inline(always)]
    fn get_M21(&self) -> f64 { self.other.get_M12() }
    #[inline(always)]
    fn get_S12(&self) -> f64 { self.other.get_S12() }
}

pub(in crate) struct ArcDist<D: DirectProblemReturnValue> {
    a12: f64,
    other: D,
}
impl<D: DirectProblemReturnValue> Default for ArcDist<D> {
    fn default() -> Self {
        Self {
            a12: f64::NAN,
            other: D::default(),
        }
    }
}
impl<D: DirectProblemReturnValue> DirectProblemReturnValue for ArcDist<D> {
    type Features = D::Features;

    #[inline(always)]
    fn set_a12(&mut self, a12: f64) {
        self.a12 = a12;
        self.other.set_a12(a12);
    }
    #[inline(always)]
    fn set_lat2(&mut self, lat2: f64) {
        self.other.set_lat2(lat2);
    }
    #[inline(always)]
    fn set_lon2(&mut self, lon2: f64) {
        self.other.set_lon2(lon2);
    }
    #[inline(always)]
    fn set_azi2(&mut self, azi2: f64) {
        self.other.set_azi2(azi2);
    }
    #[inline(always)]
    fn set_s12(&mut self, s12: f64) {
        self.other.set_s12(s12);
    }
    #[inline(always)]
    fn set_m12(&mut self, m12: f64) {
        self.other.set_m12(m12);
    }
    #[inline(always)]
    fn set_M12(&mut self, M12: f64) {
        self.other.set_M12(M12);
    }
    #[inline(always)]
    fn set_M21(&mut self, M21: f64) {
        self.other.set_M21(M21);
    }
    #[inline(always)]
    fn set_S12(&mut self, S12: f64) {
        self.other.set_S12(S12);
    }


    #[inline(always)]
    fn get_a12(&self) -> f64 { self.a12 }
    #[inline(always)]
    fn get_lat2(&self) -> f64 { self.other.get_lat2() }
    #[inline(always)]
    fn get_lon2(&self) -> f64 { self.other.get_lon2() }
    #[inline(always)]
    fn get_azi2(&self) -> f64 { self.other.get_azi2() }
    #[inline(always)]
    fn get_s12(&self) -> f64 { self.other.get_s12() }
    #[inline(always)]
    fn get_m12(&self) -> f64 { self.other.get_m12() }
    #[inline(always)]
    fn get_M12(&self) -> f64 { self.other.get_M12() }
    #[inline(always)]
    fn get_M21(&self) -> f64 { self.other.get_M12() }
    #[inline(always)]
    fn get_S12(&self) -> f64 { self.other.get_S12() }
}



pub(in crate) struct M12<D: DirectProblemReturnValue> {
    m12: f64,
    other: D,
}
impl<D: DirectProblemReturnValue> Default for M12<D> {
    fn default() -> Self {
        Self {
            m12: f64::NAN,
            other: D::default(),
        }
    }
}
impl<D: DirectProblemReturnValue> DirectProblemReturnValue for M12<D> {
    type Features = CapRedLen<D::Features>;

    #[inline(always)]
    fn set_a12(&mut self, a12: f64) {
        self.other.set_a12(a12);
    }
    #[inline(always)]
    fn set_lat2(&mut self, lat2: f64) {
        self.other.set_lat2(lat2);
    }
    #[inline(always)]
    fn set_lon2(&mut self, lon2: f64) {
        self.other.set_lon2(lon2);
    }
    #[inline(always)]
    fn set_azi2(&mut self, azi2: f64) {
        self.other.set_azi2(azi2);
    }
    #[inline(always)]
    fn set_s12(&mut self, s12: f64) {
        self.other.set_s12(s12);
    }
    #[inline(always)]
    fn set_m12(&mut self, m12: f64) {
        self.m12 = m12;
        self.other.set_m12(m12);
    }
    #[inline(always)]
    fn set_M12(&mut self, M12: f64) {
        self.other.set_M12(M12);
    }
    #[inline(always)]
    fn set_M21(&mut self, M21: f64) {
        self.other.set_M21(M21);
    }
    #[inline(always)]
    fn set_S12(&mut self, S12: f64) {
        self.other.set_S12(S12);
    }


    #[inline(always)]
    fn get_a12(&self) -> f64 { self.other.get_a12() }
    #[inline(always)]
    fn get_lat2(&self) -> f64 { self.other.get_lat2() }
    #[inline(always)]
    fn get_lon2(&self) -> f64 { self.other.get_lon2() }
    #[inline(always)]
    fn get_azi2(&self) -> f64 { self.other.get_azi2() }
    #[inline(always)]
    fn get_s12(&self) -> f64 { self.other.get_s12() }
    #[inline(always)]
    fn get_m12(&self) -> f64 { self.m12 }
    #[inline(always)]
    fn get_M12(&self) -> f64 { self.other.get_M12() }
    #[inline(always)]
    fn get_M21(&self) -> f64 { self.other.get_M12() }
    #[inline(always)]
    fn get_S12(&self) -> f64 { self.other.get_S12() }
}

pub(in crate) struct M12M21<D: DirectProblemReturnValue> {
    M12: f64,
    M21: f64,
    other: D,
}
impl<D: DirectProblemReturnValue> Default for M12M21<D> {
    fn default() -> Self {
        Self {
            M12: f64::NAN,
            M21: f64::NAN,
            other: D::default(),
        }
    }
}
impl<D: DirectProblemReturnValue> DirectProblemReturnValue for M12M21<D> {
    type Features = CapScale<D::Features>;

    #[inline(always)]
    fn set_a12(&mut self, a12: f64) {
        self.other.set_a12(a12);
    }
    #[inline(always)]
    fn set_lat2(&mut self, lat2: f64) {
        self.other.set_lat2(lat2);
    }
    #[inline(always)]
    fn set_lon2(&mut self, lon2: f64) {
        self.other.set_lon2(lon2);
    }
    #[inline(always)]
    fn set_azi2(&mut self, azi2: f64) {
        self.other.set_azi2(azi2);
    }
    #[inline(always)]
    fn set_s12(&mut self, s12: f64) {
        self.other.set_s12(s12);
    }
    #[inline(always)]
    fn set_m12(&mut self, m12: f64) {
        self.other.set_m12(m12);
    }
    #[inline(always)]
    fn set_M12(&mut self, M12: f64) {
        self.M12 = M12;
        self.other.set_M12(M12);
    }
    #[inline(always)]
    fn set_M21(&mut self, M21: f64) {
        self.M21 = M21;
        self.other.set_M21(M21);
    }
    #[inline(always)]
    fn set_S12(&mut self, S12: f64) {
        self.other.set_S12(S12);
    }


    #[inline(always)]
    fn get_a12(&self) -> f64 { self.other.get_a12() }
    #[inline(always)]
    fn get_lat2(&self) -> f64 { self.other.get_lat2() }
    #[inline(always)]
    fn get_lon2(&self) -> f64 { self.other.get_lon2() }
    #[inline(always)]
    fn get_azi2(&self) -> f64 { self.other.get_azi2() }
    #[inline(always)]
    fn get_s12(&self) -> f64 { self.other.get_s12() }
    #[inline(always)]
    fn get_m12(&self) -> f64 { self.other.get_m12() }
    #[inline(always)]
    fn get_M12(&self) -> f64 { self.M12 }
    #[inline(always)]
    fn get_M21(&self) -> f64 { self.M21 }
    #[inline(always)]
    fn get_S12(&self) -> f64 { self.other.get_S12() }
}

pub(in crate) struct Area<D: DirectProblemReturnValue> {
    S12: f64,
    other: D,
}
impl<D: DirectProblemReturnValue> Default for Area<D> {
    fn default() -> Self {
        Self {
            S12: f64::NAN,
            other: D::default(),
        }
    }
}
impl<D: DirectProblemReturnValue> DirectProblemReturnValue for Area<D> {
    type Features = CapArea<D::Features>;

    #[inline(always)]
    fn set_a12(&mut self, a12: f64) {
        self.other.set_a12(a12);
    }
    #[inline(always)]
    fn set_lat2(&mut self, lat2: f64) {
        self.other.set_lat2(lat2);
    }
    #[inline(always)]
    fn set_lon2(&mut self, lon2: f64) {
        self.other.set_lon2(lon2);
    }
    #[inline(always)]
    fn set_azi2(&mut self, azi2: f64) {
        self.other.set_azi2(azi2);
    }
    #[inline(always)]
    fn set_s12(&mut self, s12: f64) {
        self.other.set_s12(s12);
    }
    #[inline(always)]
    fn set_m12(&mut self, m12: f64) {
        self.other.set_m12(m12);
    }
    #[inline(always)]
    fn set_M12(&mut self, M12: f64) {
        self.other.set_M12(M12);
    }
    #[inline(always)]
    fn set_M21(&mut self, M21: f64) {
        self.other.set_M21(M21);
    }
    #[inline(always)]
    fn set_S12(&mut self, S12: f64) {
        self.S12 = S12;
        self.other.set_S12(S12);
    }


    #[inline(always)]
    fn get_a12(&self) -> f64 { self.other.get_a12() }
    #[inline(always)]
    fn get_lat2(&self) -> f64 { self.other.get_lat2() }
    #[inline(always)]
    fn get_lon2(&self) -> f64 { self.other.get_lon2() }
    #[inline(always)]
    fn get_azi2(&self) -> f64 { self.other.get_azi2() }
    #[inline(always)]
    fn get_s12(&self) -> f64 { self.other.get_s12() }
    #[inline(always)]
    fn get_m12(&self) -> f64 { self.other.get_m12() }
    #[inline(always)]
    fn get_M12(&self) -> f64 { self.other.get_M12() }
    #[inline(always)]
    fn get_M21(&self) -> f64 { self.other.get_M21() }
    #[inline(always)]
    fn get_S12(&self) -> f64 { self.S12 }
}

pub(in crate) struct Azi<D: DirectProblemReturnValue> {
    azi2: f64,
    other: D,
}
impl<D: DirectProblemReturnValue> Default for Azi<D> {
    fn default() -> Self {
        Self {
            azi2: f64::NAN,
            other: D::default(),
        }
    }
}
impl<D: DirectProblemReturnValue> DirectProblemReturnValue for Azi<D> {
    type Features = CapAzi<D::Features>;

    #[inline(always)]
    fn set_a12(&mut self, a12: f64) {
        self.other.set_a12(a12);
    }
    #[inline(always)]
    fn set_lat2(&mut self, lat2: f64) {
        self.other.set_lat2(lat2);
    }
    #[inline(always)]
    fn set_lon2(&mut self, lon2: f64) {
        self.other.set_lon2(lon2);
    }
    #[inline(always)]
    fn set_azi2(&mut self, azi2: f64) {
        self.azi2 = azi2;
        self.other.set_azi2(azi2);
    }
    #[inline(always)]
    fn set_s12(&mut self, s12: f64) {
        self.other.set_s12(s12);
    }
    #[inline(always)]
    fn set_m12(&mut self, m12: f64) {
        self.other.set_m12(m12);
    }
    #[inline(always)]
    fn set_M12(&mut self, M12: f64) {
        self.other.set_M12(M12);
    }
    #[inline(always)]
    fn set_M21(&mut self, M21: f64) {
        self.other.set_M21(M21);
    }
    #[inline(always)]
    fn set_S12(&mut self, S12: f64) {
        self.other.set_S12(S12);
    }


    #[inline(always)]
    fn get_a12(&self) -> f64 { self.other.get_a12() }
    #[inline(always)]
    fn get_lat2(&self) -> f64 { self.other.get_lat2() }
    #[inline(always)]
    fn get_lon2(&self) -> f64 { self.other.get_lon2() }
    #[inline(always)]
    fn get_azi2(&self) -> f64 { self.azi2 }
    #[inline(always)]
    fn get_s12(&self) -> f64 { self.other.get_s12() }
    #[inline(always)]
    fn get_m12(&self) -> f64 { self.other.get_m12() }
    #[inline(always)]
    fn get_M12(&self) -> f64 { self.other.get_M12() }
    #[inline(always)]
    fn get_M21(&self) -> f64 { self.other.get_M21() }
    #[inline(always)]
    fn get_S12(&self) -> f64 { self.other.get_S12() }
}
