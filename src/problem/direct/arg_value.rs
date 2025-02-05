

pub trait DirectProblemAzimuth: Sized + Clone + Copy {
    const AZI_ONLY: bool;

    #[inline(always)]
    fn get_azimuth(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_salp1(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_calp1(&self) -> f64 { f64::NAN }

    #[inline(always)]
    fn get_angle_info(&self) -> (f64,f64) {
        use crate::geomath::{self};

        if Self::AZI_ONLY {
            let azi1 = geomath::ang_normalize(self.get_azimuth());
            let (salp1, calp1) = geomath::sincosd(geomath::ang_round(azi1));
            (salp1, calp1)
        } else {
            (
                self.get_salp1(),
                self.get_calp1(),
            )
        }
    }
}

#[derive(Copy,Clone)]
pub struct AzimuthOnly {
    azi: f64,
}
impl From<f64> for AzimuthOnly {
    fn from(arg: f64) -> Self {
        Self {
            azi: arg,
        }
    }
}
impl DirectProblemAzimuth for AzimuthOnly {
    const AZI_ONLY: bool = true;
   
    #[inline(always)]
    fn get_azimuth(&self) -> f64 { self.azi }
}

#[derive(Copy,Clone)]
pub struct AzimuthAndAngles {
    azi: f64,
    salp1: f64,
    calp1: f64,
}
impl From<(f64,f64,f64)> for AzimuthAndAngles {
    fn from(arg: (f64,f64,f64)) -> Self {
        Self {
            azi: arg.0,
            salp1: arg.1,
            calp1: arg.2,
        }
    }
}
impl DirectProblemAzimuth for AzimuthAndAngles {
    const AZI_ONLY: bool = false;
   
    #[inline(always)]
    fn get_azimuth(&self) -> f64 { self.azi }
    #[inline(always)]
    fn get_salp1(&self) -> f64 { self.salp1 }
    #[inline(always)]
    fn get_calp1(&self) -> f64 { self.calp1 }
}

/// Is the input the direct problem an Arc (degrees) or Disance (meters)
pub trait DirectProblemDistanceArg: Sized + Clone + Copy {
    const ARC_MODE: bool;

    #[inline(always)]
    fn get_a12(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_s12(&self) -> f64 { f64::NAN }
}

#[derive(Copy,Clone)]
pub struct DistanceIn {
    s12: f64,
}
impl From<f64> for DistanceIn {
    fn from(arg: f64) -> Self {
        Self {
            s12: arg,
        }
    }
}
impl DirectProblemDistanceArg for DistanceIn {
    const ARC_MODE: bool = false;

    #[inline(always)]
    fn get_s12(&self) -> f64 { self.s12 }
}

#[derive(Copy,Clone)]
pub struct ArcIn {
    a12: f64,
}
impl From<f64> for ArcIn {
    fn from(arg: f64) -> Self {
        Self {
            a12: arg,
        }
    }
}
impl DirectProblemDistanceArg for ArcIn {
    const ARC_MODE: bool = true;

    #[inline(always)]
    fn get_a12(&self) -> f64 { self.a12 }
}
