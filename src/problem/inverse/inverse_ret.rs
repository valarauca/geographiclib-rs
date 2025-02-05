

pub (in crate) trait InverseProblemReturnValue: Default {
    type Features: Caps;

    #[inline(always)]
    fn set_a12(&mut self, _a12: f64) { }
    #[inline(always)]
    fn set_s12(&mut self, _s12: f64) { }
    #[inline(always)]
    fn set_salp1(&mut self, _salp1: f64) { }
    #[inline(always)]
    fn set_calp1(&mut self, _calp1: f64) { }
    #[inline(always)]
    fn set_salp2(&mut self, _salp1: f64) { }
    #[inline(always)]
    fn set_calp2(&mut self, _calp1: f64) { }
    #[inline(always)]
    fn set_m12(&mut self, _m12: f64) { }
    #[inline(always)]
    fn set_M12(&mut self, _M12: f64) { }
    #[inline(always)]
    fn set_M21(&mut self, _M21: f64) { }


    #[inline(always)]
    fn get_a12(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_s12(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_salp1(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_calp1(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_salp2(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_calp2(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_m12(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_M12(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_M21(&self) -> f64 { f64::NAN }
}
