

pub (in crate) trait LengthsReturnValue: Sized + Default {
    const DISTANCE: bool = false;
    const REDUCED_LENGTH: bool = false;
    const GEODESIC_SCALE: bool = false;

    fn set_s12b(&mut self, _s12b: f64) { }
    fn set_m12b(&mut self, _m12b: f64) { }
    fn set_m0(&mut self, _m0: f64) { }
    fn set_m12(&mut self, _m12: f64) { }
    fn set_m21(&mut self, _m21: f64) { }

    fn get_s12b(&self) -> f64 { f64::NAN }
    fn get_m12b(&self) -> f64 { f64::NAN }
    fn get_m0(&self) -> f64 { f64::NAN }
    fn get_m12(&self) -> f64 { f64::NAN }
    fn get_m21(&self) -> f64 { f64:: NAN }
}
