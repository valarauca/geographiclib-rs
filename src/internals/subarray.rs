

/// SubArray is a zero-sized type to encode information about array
/// indexing into the type system.
///
/// This is somewhat complicated as it allows for us to remove all
/// bounds checks (without requiring unsafe code) on a stable rust
/// compiler version. 
///
/// The primary use-case is to convert arrays of type
/// `&'a [T;5] -> &'a [T;3]` without any checks.
///
/// While this may seem like a fairly trivial thing and not worthy
/// of such optimization a large amount of time in the karney inverse
/// algorithm is spent summing polynomials with the clenshaw algorithm
/// which requires creating a sub-array window of a larger array. If
/// this sub-array window has any branches it can compliate lowering
/// the resulting code into SIMD.
///
/// As an example consider the c3f function:
///
/// * naive implementation (-Copt-level=3 -Ctarget-cpu=native)
/// * * fully unrolled (with constant values creating windows)
/// * * 7 jump targets
/// * * 121 instructions
/// * * 2 branch instructions
/// * * 2 possible panics
/// * subarray implementation (-Copt-level=3 -Ctarget-cpu=native)
/// * * fully unrolled (with subarray creating windows)
/// * * 0 jump targets
/// * * 36 instructions
/// * * 0 branches
/// * * 0 panics
///
/// It ain't pretty but it is fast.
pub (in crate) struct SubArray<const LEN: usize, const FROM: usize>;

impl<T, const A: usize, const LEN: usize, const FROM: usize> std::ops::Index<SubArray<LEN,FROM>> for [T;A] {
    type Output = [T;LEN];
    #[inline(always)]
    fn index<'a>(&'a self, _index: SubArray<LEN,FROM>) -> &'a Self::Output  {
        use std::convert::TryInto;
        if FROM >= A || LEN >= A || (FROM+LEN) > A {
            panic!("out of bounds index")
        } else {
            self[FROM.. (FROM+LEN)].try_into().unwrap()
        }
    }
}


#[test]
fn ensure_subslicing_works() {
    use crate::internals::constants::{COEFF_SIZE,C1F_COEFF};

    let sub_array = &C1F_COEFF[SubArray::<{COEFF_SIZE - 4},4>];
    let sub_slice = &C1F_COEFF[4..];
    assert_eq!(&sub_array, &sub_slice);

    let sub_array = &C1F_COEFF[SubArray::<{COEFF_SIZE - 8},8>];
    let sub_slice = &C1F_COEFF[8..];
    assert_eq!(&sub_array, &sub_slice);

    let sub_array = &C1F_COEFF[SubArray::<{COEFF_SIZE - 11},11>];
    let sub_slice = &C1F_COEFF[11..];
    assert_eq!(&sub_array, &sub_slice);
}
