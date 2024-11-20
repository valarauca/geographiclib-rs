
use crate::cached_weights::WeightCaps;


pub trait Caps {
    const LATITUDE: bool = false;
    const LONGITUDE: bool = false;
    const AZIMUTH: bool = false;
    const DISTANCE: bool = false;
    const DISTANCE_IN: bool = false;
    const REDUCEDLENGTH: bool = false;
    const GEODESICSCALE: bool = false;
    const AREA: bool = false;

    const C1: bool = Self::DISTANCE | Self::DISTANCE_IN | Self::REDUCEDLENGTH | Self::GEODESICSCALE;
    const C1P: bool = Self::DISTANCE_IN;
    const C2: bool = Self::REDUCEDLENGTH | Self::GEODESICSCALE;
    const C3: bool = Self::LONGITUDE;
    const C4: bool = Self::AREA;
    const CHECK_THIRD_FLATTENING: bool = false;
}
impl<C: Caps> WeightCaps for C {
    const C1: bool = Self::DISTANCE | Self::DISTANCE_IN | Self::REDUCEDLENGTH | Self::GEODESICSCALE;
    const C1P: bool = Self::DISTANCE_IN;
    const C2: bool = Self::REDUCEDLENGTH | Self::GEODESICSCALE;
    const C3: bool = Self::LONGITUDE;
    const C4: bool = Self::AREA;
    const CHECK_THIRD_FLATTENING: bool = false;
}

pub struct Empty;
impl Caps for Empty { }

pub struct Latitude<C: Caps> {
    _data: std::marker::PhantomData<C>,
}
impl<C: Caps> Caps for Latitude<C> {
    const LATITUDE: bool = true | C::LATITUDE;
    const LONGITUDE: bool = C::LONGITUDE;
    const AZIMUTH: bool = C::AZIMUTH;
    const DISTANCE: bool = C::DISTANCE;
    const DISTANCE_IN: bool = C::DISTANCE_IN;
    const REDUCEDLENGTH: bool = C::REDUCEDLENGTH;
    const GEODESICSCALE: bool = C::GEODESICSCALE;
    const AREA: bool = C::AREA;
}

pub struct Longitude<C: Caps> {
    _data: std::marker::PhantomData<C>,
}
impl<C: Caps> Caps for Longitude<C> {
    const LATITUDE: bool = C::LATITUDE;
    const LONGITUDE: bool = true | C::LONGITUDE;
    const AZIMUTH: bool = C::AZIMUTH;
    const DISTANCE: bool = C::DISTANCE;
    const DISTANCE_IN: bool = C::DISTANCE_IN;
    const REDUCEDLENGTH: bool = C::REDUCEDLENGTH;
    const GEODESICSCALE: bool = C::GEODESICSCALE;
    const AREA: bool = C::AREA;
}

pub struct Azimuth<C: Caps> {
    _data: std::marker::PhantomData<C>,
}
impl<C: Caps> Caps for Azimuth<C> {
    const LATITUDE: bool = C::LATITUDE;
    const LONGITUDE: bool = C::LONGITUDE;
    const AZIMUTH: bool = true | C::AZIMUTH;
    const DISTANCE: bool = C::DISTANCE;
    const DISTANCE_IN: bool = C::DISTANCE_IN;
    const REDUCEDLENGTH: bool = C::REDUCEDLENGTH;
    const GEODESICSCALE: bool = C::GEODESICSCALE;
    const AREA: bool = C::AREA;
}

pub struct Distance<C: Caps> {
    _data: std::marker::PhantomData<C>,
}
impl<C: Caps> Caps for Distance<C> {
    const LATITUDE: bool = C::LATITUDE;
    const LONGITUDE: bool = C::LONGITUDE;
    const AZIMUTH: bool = C::AZIMUTH;
    const DISTANCE: bool = true | C::DISTANCE;
    const DISTANCE_IN: bool = C::DISTANCE_IN;
    const REDUCEDLENGTH: bool = C::REDUCEDLENGTH;
    const GEODESICSCALE: bool = C::GEODESICSCALE;
    const AREA: bool = C::AREA;
}

pub struct DistanceIn<C: Caps> {
    _data: std::marker::PhantomData<C>,
}
impl<C: Caps> Caps for DistanceIn<C> {
    const LATITUDE: bool = C::LATITUDE;
    const LONGITUDE: bool = C::LONGITUDE;
    const AZIMUTH: bool = C::AZIMUTH;
    const DISTANCE: bool = C::DISTANCE;
    const DISTANCE_IN: bool = true | C::DISTANCE_IN;
    const REDUCEDLENGTH: bool = C::REDUCEDLENGTH;
    const GEODESICSCALE: bool = C::GEODESICSCALE;
    const AREA: bool = C::AREA;
}

pub struct ReducedLength<C: Caps> {
    _data: std::marker::PhantomData<C>,
}
impl<C: Caps> Caps for ReducedLength<C> {
    const LATITUDE: bool = C::LATITUDE;
    const LONGITUDE: bool = C::LONGITUDE;
    const AZIMUTH: bool = C::AZIMUTH;
    const DISTANCE: bool = C::DISTANCE;
    const DISTANCE_IN: bool = C::DISTANCE_IN;
    const REDUCEDLENGTH: bool = true | C::REDUCEDLENGTH;
    const GEODESICSCALE: bool = C::GEODESICSCALE;
    const AREA: bool = C::AREA;
}

pub struct GeodesicScale<C: Caps> {
    _data: std::marker::PhantomData<C>,
}
impl<C: Caps> Caps for GeodesicScale<C> {
    const LATITUDE: bool = C::LATITUDE;
    const LONGITUDE: bool = C::LONGITUDE;
    const AZIMUTH: bool = C::AZIMUTH;
    const DISTANCE: bool = C::DISTANCE;
    const DISTANCE_IN: bool = C::DISTANCE_IN;
    const REDUCEDLENGTH: bool = C::REDUCEDLENGTH;
    const GEODESICSCALE: bool = true | C::GEODESICSCALE;
    const AREA: bool = C::AREA;
}

pub struct Area<C: Caps> {
    _data: std::marker::PhantomData<C>,
}
impl<C: Caps> Caps for Area<C> {
    const LATITUDE: bool = C::LATITUDE;
    const LONGITUDE: bool = C::LONGITUDE;
    const AZIMUTH: bool = C::AZIMUTH;
    const DISTANCE: bool = C::DISTANCE;
    const DISTANCE_IN: bool = C::DISTANCE_IN;
    const REDUCEDLENGTH: bool = C::REDUCEDLENGTH;
    const GEODESICSCALE: bool = C::GEODESICSCALE;
    const AREA: bool = true | C::AREA;
}

pub struct CheckN<C: Caps> {
    _data: std::marker::PhantomData<C>,
}
impl<C: Caps> Caps for CheckN<C> {
    const LATITUDE: bool = C::LATITUDE;
    const LONGITUDE: bool = C::LONGITUDE;
    const AZIMUTH: bool = C::AZIMUTH;
    const DISTANCE: bool = C::DISTANCE;
    const DISTANCE_IN: bool = C::DISTANCE_IN;
    const REDUCEDLENGTH: bool = C::REDUCEDLENGTH;
    const GEODESICSCALE: bool = C::GEODESICSCALE;
    const AREA: bool = C::AREA;
    const CHECK_THIRD_FLATTENING: bool = true;
}

pub type Standard = Latitude<Longitude<Azimuth<Distance<Empty>>>>;
pub type PolygonAreaCap = Latitude<Longitude<Distance<Area<Empty>>>>;

#[test]
fn test_mask_standard() {
    use crate::geodesic_capability::{self as caps};

    // test standard
    assert_eq!((caps::STANDARD & caps::CAP_C1) != 0, <Standard as Caps>::C1);
    assert_eq!((caps::STANDARD & caps::CAP_C1p) != 0, <Standard as Caps>::C1P);
    assert_eq!((caps::STANDARD & caps::CAP_C2) != 0, <Standard as Caps>::C2);
    assert_eq!((caps::STANDARD & caps::CAP_C3) != 0, <Standard as Caps>::C3);
    assert_eq!((caps::STANDARD & caps::CAP_C4) != 0, <Standard as Caps>::C4);
    assert!(<Standard as Caps>::LATITUDE);
    assert!(<Standard as Caps>::LONGITUDE);
    assert!(<Standard as Caps>::AZIMUTH);
    assert!(<Standard as Caps>::DISTANCE);
}

#[test]
fn test_mask_area() {
    use crate::geodesic_capability::{self as caps};
    // test POLYGONAREA
    const POLYGONAREA_MASK: u64 = caps::LATITUDE | caps::LONGITUDE | caps::DISTANCE | caps::AREA | caps::LONG_UNROLL;

    assert_eq!((POLYGONAREA_MASK & caps::CAP_C1) != 0, <PolygonAreaCap as Caps>::C1);
    assert_eq!((POLYGONAREA_MASK & caps::CAP_C1p) != 0, <PolygonAreaCap as Caps>::C1P);
    assert_eq!((POLYGONAREA_MASK & caps::CAP_C2) != 0, <PolygonAreaCap as Caps>::C2);
    assert_eq!((POLYGONAREA_MASK & caps::CAP_C3) != 0, <PolygonAreaCap as Caps>::C3);
    assert_eq!((POLYGONAREA_MASK & caps::CAP_C4) != 0, <PolygonAreaCap as Caps>::C4);
    assert!(<PolygonAreaCap as Caps>::LATITUDE);
    assert!(<PolygonAreaCap as Caps>::LONGITUDE);
    assert!(<PolygonAreaCap as Caps>::DISTANCE);
    assert!(<PolygonAreaCap as Caps>::AREA);
}



pub struct All;
impl Caps for All {
    const LATITUDE: bool = true;
    const LONGITUDE: bool = true;
    const AZIMUTH: bool = true;
    const DISTANCE: bool = true;
    const DISTANCE_IN: bool = true;
    const REDUCEDLENGTH: bool = true;
    const GEODESICSCALE: bool = true;
    const AREA: bool = true;
    const CHECK_THIRD_FLATTENING: bool = false;
}

#[test]
fn test_mask_all() {
    use crate::geodesic_capability::{self as caps};

    assert_eq!((caps::ALL & caps::CAP_C1) != 0, <All as Caps>::C1);
    assert_eq!((caps::ALL & caps::CAP_C1p) != 0, <All as Caps>::C1P);
    assert_eq!((caps::ALL & caps::CAP_C2) != 0, <All as Caps>::C2);
    assert_eq!((caps::ALL & caps::CAP_C3) != 0, <All as Caps>::C3);
    assert_eq!((caps::ALL & caps::CAP_C4) != 0, <All as Caps>::C4);
    assert_eq!((caps::ALL & caps::LATITUDE) != 0, <All as Caps>::LATITUDE);
    assert_eq!((caps::ALL & caps::LONGITUDE) != 0, <All as Caps>::LONGITUDE);
    assert_eq!((caps::ALL & caps::AZIMUTH) != 0, <All as Caps>::AZIMUTH);
    assert_eq!((caps::ALL & caps::DISTANCE) != 0, <All as Caps>::DISTANCE);
    assert_eq!((caps::ALL & caps::DISTANCE_IN) != 0, <All as Caps>::DISTANCE_IN);
    assert_eq!((caps::ALL & caps::GEODESICSCALE) != 0, <All as Caps>::GEODESICSCALE);
    assert_eq!((caps::ALL & caps::AREA) != 0, <All as Caps>::AREA);
}
