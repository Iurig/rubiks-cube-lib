use std::ops::Neg;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ZnRing<const N: usize>(usize);

impl<const N: usize> From<usize> for ZnRing<N> {
    fn from(integer: usize) -> Self {
        () = Self::CHECK;
        ZnRing(integer % N)
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
    const CHECK: () = assert!(N >= 1);
    pub const ZERO: Self = ZnRing(0);

    #[must_use = "the new value is returned"]
    pub const fn new(value: usize) -> Self {
        () = Self::CHECK;
        ZnRing(value % N)
    }

    #[must_use = "the array is returned"]
    pub const fn array<const L: usize>(values: [usize; L]) -> [Self; L] {
        let mut final_array = [ZnRing(0); L];
        let mut i = 0;
        while i < L {
            final_array[i] = ZnRing(values[i]);
            i += 1;
        }
        final_array
    }

    #[must_use = "the addition is returned"]
    pub const fn const_add(self, rhs: Self) -> Self {
        ZnRing((self.0 + rhs.0) % N)
    }

    #[must_use = "the negation is returned"]
    pub const fn const_neg(&self) -> Self {
        Self((N - self.0) % N)
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
    fn zn_ring_reduces_wraps_and_negates() {
        assert_eq!(ZnRing::<3>::from(7), ZnRing::new(1));
        assert_eq!(ZnRing::<3>::new(2) + ZnRing::new(2), ZnRing::new(1));
        for x in 0..3 {
            let v = ZnRing::<3>::from(x);
            assert_eq!(v + (-v), ZnRing::new(0));
        }
    }
}
