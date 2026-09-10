use std::ops::Neg;

/// Integers mod `N`, stored as the representative in `0..N`.
///
/// The representative fits a byte: a twist is at most 2 and a flip at most
/// 1, and `N` is capped at 256 so any modulus this crate could want still
/// fits. The public face works in `usize`; the byte never leaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ZnRing<const N: usize>(u8);

impl<const N: usize> From<usize> for ZnRing<N> {
    fn from(integer: usize) -> Self {
        Self::new(integer)
    }
}
impl<const N: usize> std::ops::Add for ZnRing<N> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.const_add(rhs)
    }
}
impl<const N: usize> Neg for ZnRing<N> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        self.const_neg()
    }
}

impl<const N: usize> ZnRing<N> {
    const CHECK: () = assert!(N >= 1 && N <= 256, "N must be in 1..=256");
    pub const ZERO: Self = Self(0);

    #[must_use = "the new value is returned"]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "value % N < N <= 256, so the representative fits a byte"
    )]
    pub const fn new(value: usize) -> Self {
        () = Self::CHECK;
        Self((value % N) as u8)
    }

    #[must_use]
    pub const fn value(&self) -> usize {
        self.0 as usize
    }

    #[must_use = "the array is returned"]
    pub const fn array<const L: usize>(values: [usize; L]) -> [Self; L] {
        let mut final_array = [Self(0); L];
        let mut i = 0;
        while i < L {
            final_array[i] = Self::new(values[i]);
            i += 1;
        }
        final_array
    }

    #[must_use = "the addition is returned"]
    pub const fn const_add(self, rhs: Self) -> Self {
        Self::new(self.value() + rhs.value())
    }

    #[must_use = "the negation is returned"]
    pub const fn const_neg(&self) -> Self {
        Self::new(N - self.value())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zn_ring_public_arithmetic() {
        assert_eq!(ZnRing::<5>::from(7), ZnRing::from(2));
        assert_eq!(ZnRing::<5>::from(3) + ZnRing::from(4), ZnRing::from(2));
        assert_eq!(-ZnRing::<5>::from(2), ZnRing::from(3));
        assert_eq!(-ZnRing::<5>::from(0), ZnRing::from(0));
    }

    #[test]
    fn value_is_the_representative_in_range() {
        assert_eq!(ZnRing::<3>::ZERO.value(), 0);
        assert_eq!(ZnRing::<3>::new(7).value(), 1);
        for x in 0..3 {
            assert_eq!(ZnRing::<3>::new(x).value(), x);
        }
        assert_eq!((ZnRing::<3>::new(2) + ZnRing::new(2)).value(), 1);
    }

    #[test]
    fn ring_is_one_byte_and_the_largest_modulus_still_adds_and_negates() {
        assert_eq!(std::mem::size_of::<ZnRing<3>>(), 1);
        let top = ZnRing::<256>::new(255);
        assert_eq!((top + top).value(), 254);
        assert_eq!((-top).value(), 1);
        assert_eq!((-ZnRing::<256>::ZERO).value(), 0);
    }

    #[test]
    fn zn_ring_reduces_wraps_and_negates() {
        assert_eq!(ZnRing::<3>::from(7), ZnRing::new(1));
        assert_eq!(ZnRing::<3>::new(2) + ZnRing::new(2), ZnRing::new(1));
        for x in 0..3 {
            let v = ZnRing::<3>::from(x);
            assert_eq!(v + (-v), ZnRing::new(0));
        }
    }
}
