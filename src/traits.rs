
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

type Standard = Latitude<Longitude<Azimuth<Distance<Empty>>>>;

#[test]
fn test_mask_generation() {
    use crate::geodesic_capability::{self as caps};

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
