
use crate::traits::lengths::LengthsReturnValue;

/// Capabilities is an abstract trait that lets us define
/// what calculations should/should not be performed.
pub (in crate) trait Capabilities {
    const LATITUDE: bool = false;
    const LONGITUDE: bool = false;
    const AZIMUTH: bool = false;
    const DISTANCE: bool = false;
    const DISTANCE_IN: bool = false;
    const REDUCED_LENGTH: bool = false;
    const GEODESIC_SCALE: bool = false;
    const AREA: bool = false;

    type LRV: LengthsReturnValue;

    const PANIC: () = {
        if <Self::LRV as LengthsReturnValue>::DISTANCE != Self::DISTANCE {
            panic!()
        }
        if <Self::LRV as LengthsReturnValue>::GEODESIC_SCALE != Self::GEODESIC_SCALE {
            panic!()
        }
        if <Self::LRV as LengthsReturnValue>::REDUCED_LENGTH != Self::REDUCED_LENGTH {
            panic!()
        }
    };
}

/// This 'lowers' the relationship of meaningful user inputs
/// into what functions should/should not be used.
///
/// It is implemented for every type that implements Capabilities.
pub (in crate) trait UsedFourierConstants {
    const C1: bool = false;
    const C1p: bool = false;
    const C2: bool = false;
    const C3: bool = false;
    const C4: bool = false;
}

impl<C: Capabilities> UsedFourierConstants for C {
    const C1: bool = C::GEODESIC_SCALE | C::REDUCED_LENGTH | C::DISTANCE_IN | C::DISTANCE;
    const C1p: bool = C::DISTANCE_IN;
    const C2: bool = C::GEODESIC_SCALE | C::REDUCED_LENGTH;
    const C3: bool = C::LONGITUDE;
    const C4: bool = C::AREA;
}

pub (in crate) trait GenInverseReturnValue: Sized + Default { }

pub (in crate) trait InverseReturnValue: Sized + Default { }
