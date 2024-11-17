pub const CAP_NONE: u64 = 0;
pub const CAP_C1: u64 = 1 << 0;
#[allow(non_upper_case_globals)]
pub const CAP_C1p: u64 = 1 << 1;
pub const CAP_C2: u64 = 1 << 2;
pub const CAP_C3: u64 = 1 << 3;
pub const CAP_C4: u64 = 1 << 4;
pub const CAP_ALL: u64 = 0x1F;
pub const CAP_MASK: u64 = CAP_ALL; //0111_1111_1000_0000
pub const OUT_ALL: u64 = 0x7F80;
// Includes LONG_UNROLL
pub const OUT_MASK: u64 = 0xFF80; // b1111_1111_1000_000
pub const EMPTY: u64 = 0;
pub const LATITUDE: u64 = 1 << 7 | CAP_NONE;        // b000_1000_0000
pub const LONGITUDE: u64 = 1 << 8 | CAP_C3;         // b001_0000_1000
pub const AZIMUTH: u64 = 1 << 9 | CAP_NONE;         // b010_0000_0000
pub const DISTANCE: u64 = 1 << 10 | CAP_C1;         // b100_0000_0001
pub const STANDARD: u64 = LATITUDE | LONGITUDE | AZIMUTH | DISTANCE;
pub const DISTANCE_IN: u64 = 1 << 11 | CAP_C1 | CAP_C1p;
pub const REDUCEDLENGTH: u64 = 1 << 12 | CAP_C1 | CAP_C2;
pub const GEODESICSCALE: u64 = 1 << 13 | CAP_C1 | CAP_C2;
pub const AREA: u64 = 1 << 14 | CAP_C4;
pub const LONG_UNROLL: u64 = 1 << 15;
// Does not include LONG_UNROLL
pub const ALL: u64 = OUT_ALL | CAP_ALL;


