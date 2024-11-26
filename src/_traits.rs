
#[derive(Default)]
#[repr(transparent)]
pub (in crate) struct M12b {
    m12b: f64,
}
impl LengthsReturnValue for M12bM0 {
    #[inline(always)]
    fn set_m12b(&mut self, m12b: f64) { self.m12b = m12b; }
    fn get_m12b(&mut self) -> f64 { self.m12b }

#[derive(Default)]
pub (in crate) struct M12bM0 {
    m12b: f64,
    m0: f64,
}
impl LengthsReturnValue for M12bM0 {
    #[inline(always)]
    fn set_m12b(&mut self, m12b: f64) { self.m12b = m12b; }
    #[inline(always)]
    fn set_m0(&mut self, m0: f64) { self.m0 = m0; }
    #[inline(always)]
    fn get_m12b(&mut self) -> f64 { self.m12b }
    #[inline(always)]
    fn get_m0(&mut self) -> f64 { self.m0 }
}

pub (in crate) struct s12x{
}

