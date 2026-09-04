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
        ZnRing((self.0 + rhs.0) % N)
    }
}
impl<const N: usize> Neg for ZnRing<N> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::from(N - self.0)
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
}
