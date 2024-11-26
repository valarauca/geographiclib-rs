
/// This trait exists because Rust/LLVM isn't always so smart about
/// ignoring return values in a function if they aren't used
/// even when they aren't used.
///
/// So in cases where only 1 value for `_Lengths` is returned,
/// it will still reserve stack space for all 5.
///
/// This is problematic.
pub (in crate) trait LengthsReturnValue: Default {
    #[inline(always)]
    fn set_s12b(&mut self, _s12b: f64) { }
    #[inline(always)]
    fn set_m12b(&mut self, _m12b: f64) { }
    #[inline(always)]
    fn set_m0(&mut self, _m0: f64) { }
    #[inline(always)]
    fn set_m12(&mut self, _m12: f64) { }
    #[inline(always)]
    fn set_m21(&mut self, _m21: f64) { }

    #[inline(always)]
    fn get_s12b(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_m12b(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_m0(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_m12(&self) -> f64 { f64::NAN }
    #[inline(always)]
    fn get_m21(&self) -> f64 { f64::NAN }
}

// this type is used in testing.
#[allow(dead_code)]
pub (in crate) struct S12bAlone {
    s12b: f64,
}
impl Default for S12bAlone {
    #[inline(always)]
    fn default() -> Self { Self { s12b: f64::NAN } }
}
impl LengthsReturnValue for S12bAlone {
    #[inline(always)]
    fn set_s12b(&mut self, s12b: f64) { self.s12b = s12b; }
    #[inline(always)]
    fn get_s12b(&self) -> f64 { self.s12b }
}
pub (in crate) struct M12b {
    m12b: f64,
}
impl Default for M12b {
    #[inline(always)]
    fn default() -> Self { Self { m12b: f64::NAN } }
}
impl LengthsReturnValue for M12b {
    #[inline(always)]
    fn set_m12b(&mut self, m12b: f64) { self.m12b = m12b }
    #[inline(always)]
    fn get_m12b(&self) -> f64 { self.m12b }
}

/*
 * A lot of boilerplate for all the other arguments
 *
 */

pub (in crate) struct S12b<L: LengthsReturnValue> {
    s12b: f64,
    other: L,
}
impl<L: LengthsReturnValue> Default for S12b<L> {
    #[inline(always)]
    fn default() -> Self { 
        Self {
            s12b: f64::NAN,
            other: L::default(),
        }
    }
}
impl<L: LengthsReturnValue> LengthsReturnValue for S12b<L> {
    #[inline(always)]
    fn set_s12b(&mut self, s12b: f64) {
        self.s12b = s12b;
    }
    #[inline(always)]
    fn set_m12b(&mut self, m12b: f64) {
        self.other.set_m12b(m12b);
    }
    #[inline(always)]
    fn set_m0(&mut self, m0: f64) {
        self.other.set_m0(m0);
    }
    #[inline(always)]
    fn set_m12(&mut self, m12: f64) {
        self.other.set_m12(m12);
    }
    #[inline(always)]
    fn set_m21(&mut self, m21: f64) {
        self.other.set_m21(m21);
    }

    #[inline(always)]
    fn get_s12b(&self) -> f64 { self.s12b }
    #[inline(always)]
    fn get_m12b(&self) -> f64 { self.other.get_m12b() }
    #[inline(always)]
    fn get_m0(&self) -> f64 { self.other.get_m0() }
    #[inline(always)]
    fn get_m12(&self) -> f64 { self.other.get_m12() }
    #[inline(always)]
    fn get_m21(&self) -> f64 { self.other.get_m21() }
}

pub (in crate) struct M0<L: LengthsReturnValue> {
    m0: f64,
    other: L,
}
impl<L: LengthsReturnValue> Default for M0<L> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            m0: f64::NAN,
            other: L::default(),
        }
    }
}
impl<L: LengthsReturnValue> LengthsReturnValue for M0<L> {
    #[inline(always)]
    fn set_s12b(&mut self, s12b: f64) {
        self.other.set_m12b(s12b);
    }
    #[inline(always)]
    fn set_m12b(&mut self, m12b: f64) {
        self.other.set_m12b(m12b);
    }
    #[inline(always)]
    fn set_m0(&mut self, m0: f64) {
        self.m0 = m0;
    }
    #[inline(always)]
    fn set_m12(&mut self, m12: f64) {
        self.other.set_m12(m12);
    }
    #[inline(always)]
    fn set_m21(&mut self, m21: f64) {
        self.other.set_m21(m21);
    }

    #[inline(always)]
    fn get_s12b(&self) -> f64 { self.other.get_s12b() }
    #[inline(always)]
    fn get_m12b(&self) -> f64 { self.other.get_m12b() }
    #[inline(always)]
    fn get_m0(&self) -> f64 { self.m0 }
    #[inline(always)]
    fn get_m12(&self) -> f64 { self.other.get_m12() }
    #[inline(always)]
    fn get_m21(&self) -> f64 { self.other.get_m21() }
}

pub (in crate) struct M21M12<L: LengthsReturnValue> {
    m21: f64,
    m12: f64,
    other: L,
}
impl<L: LengthsReturnValue> Default for M21M12<L> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            m21: f64::NAN,
            m12: f64::NAN,
            other: L::default(),
        }
    }
}
impl<L: LengthsReturnValue> LengthsReturnValue for M21M12<L> {
    #[inline(always)]
    fn set_s12b(&mut self, s12b: f64) {
        self.other.set_m12b(s12b);
    }
    #[inline(always)]
    fn set_m12b(&mut self, m12b: f64) {
        self.other.set_m12b(m12b);
    }
    #[inline(always)]
    fn set_m0(&mut self, m0: f64) {
        self.other.set_m0(m0);
    }
    #[inline(always)]
    fn set_m12(&mut self, m12: f64) {
        self.m12 = m12;
    }
    #[inline(always)]
    fn set_m21(&mut self, m21: f64) {
        self.m21 = m21;
    }

    #[inline(always)]
    fn get_s12b(&self) -> f64 { self.other.get_s12b() }
    #[inline(always)]
    fn get_m12b(&self) -> f64 { self.other.get_m12b() }
    #[inline(always)]
    fn get_m0(&self) -> f64 { self.other.get_m0() }
    #[inline(always)]
    fn get_m12(&self) -> f64 { self.m12 }
    #[inline(always)]
    fn get_m21(&self) -> f64 { self.m21 }
}
