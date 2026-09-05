use crate::ops::Inv;
use crate::zn::ZnRing;

/// # Safety
/// Implementors must be fieldless `#[repr(u8)]` enums whose discriminants
/// are `0..N`, in the same order as `ALL`.
pub unsafe trait SinglePiece<const N: usize>: Copy + Eq {
    const ALL: [Self; N];
}
#[must_use]
pub fn from_index<P, const N: usize>(index: usize) -> P
where
    P: SinglePiece<N>,
{
    P::ALL[index]
}
#[must_use]
pub const fn index<P, const N: usize>(piece: P) -> usize
where
    P: SinglePiece<N>,
{
    unsafe { (&raw const piece).cast::<u8>().read() as usize }
}
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct PieceConfiguration<P, const N: usize, const O: usize> {
    pub(crate) permutation: [P; N],
    pub(crate) orientation: [ZnRing<O>; N],
}
impl<P, const N: usize, const O: usize> Default for PieceConfiguration<P, N, O>
where
    P: SinglePiece<N>,
{
    fn default() -> Self {
        Self::IDENTITY
    }
}
impl<P, const N: usize, const O: usize> Inv for PieceConfiguration<P, N, O>
where
    P: SinglePiece<N>,
{
    fn inverse(&self) -> Self {
        let mut inv = PieceConfiguration::IDENTITY;
        for i in 0..N {
            inv.permutation[index(self.permutation[i])] = P::ALL[i];
            inv.orientation[index(self.permutation[i])] = -self.orientation[i];
        }
        inv
    }
}
impl<P, const N: usize, const O: usize> PieceConfiguration<P, N, O>
where
    P: SinglePiece<N>,
{
    /// The solved state for a given piece type
    pub const IDENTITY: Self = PieceConfiguration {
        permutation: P::ALL,
        orientation: [ZnRing::ZERO; N],
    };

    /// Compose permutations done by `self` with `other`
    #[must_use]
    pub const fn then(&self, other: &Self) -> Self {
        let mut composed = Self::IDENTITY;
        let mut i = 0;
        while i < N {
            composed.permutation[i] = self.permutation[index(other.permutation[i])];
            composed.orientation[i] =
                self.orientation[index(other.permutation[i])].const_add(other.orientation[i]);
            i += 1;
        }
        composed
    }

    pub const fn cycle<const CYCLE_SIZE: usize, const CYCLE_AMOUNT: usize>(
        to_cycle: [[P; CYCLE_SIZE]; CYCLE_AMOUNT],
    ) -> Self {
        let mut resp = PieceConfiguration::IDENTITY;
        let mut i = 0;
        while i < CYCLE_AMOUNT {
            let mut j = 0;
            while j < CYCLE_SIZE {
                resp.permutation[index(to_cycle[i][(j + 1) % CYCLE_SIZE])] = to_cycle[i][j];
                j += 1;
            }
            i += 1;
        }
        resp
    }
    #[must_use = "the inverse is returned"]
    pub const fn const_inverse(&self) -> Self {
        let mut inv = PieceConfiguration::IDENTITY;
        let mut i = 0;
        while i < N {
            inv.permutation[index(self.permutation[i])] = P::ALL[i];
            inv.orientation[index(self.permutation[i])] = self.orientation[i].const_neg();
            i += 1;
        }
        inv
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cube3by3::pieces::*;
    #[test]
    fn corner_and_edge_all_match_discriminants() {
        for (i, c) in SingleCorner::ALL.iter().enumerate() {
            assert_eq!(*c as usize, i);
            assert_eq!(from_index::<SingleCorner, _>(i), *c);
        }
        for (i, e) in SingleEdge::ALL.iter().enumerate() {
            assert_eq!(*e as usize, i);
            assert_eq!(from_index::<SingleEdge, _>(i), *e);
        }
    }

    #[test]
    fn cycle_is_a_permutation_and_moves_pieces_forward_and_leaves_rest() {
        use SingleCorner::{Dbr, Dfr, Ubr, Ufr};
        let mut perm = Corners::cycle([[Ufr, Ubr, Dbr, Dfr]]).permutation;
        assert_eq!(perm[index(Ubr)], Ufr);
        assert_eq!(perm[index(Dbr)], Ubr);
        assert_eq!(perm[index(Dfr)], Dbr);
        assert_eq!(perm[index(Ufr)], Dfr);
        for c in [
            SingleCorner::Ubl,
            SingleCorner::Ufl,
            SingleCorner::Dfl,
            SingleCorner::Dbl,
        ] {
            assert_eq!(perm[c as usize], c);
        }
        perm.sort();
        assert_eq!(perm, SingleCorner::ALL);
    }

    #[test]
    fn cycle_with_disjoint_cycles_is_a_permutation() {
        use SingleEdge::{Dl, Dr, Ub, Uf};
        let mut perm = Edges::cycle::<2, 2>([[Ub, Uf], [Dl, Dr]]).permutation;
        assert_eq!(perm[Uf as usize], Ub);
        assert_eq!(perm[Ub as usize], Uf);
        perm.sort();
        assert_eq!(perm, SingleEdge::ALL);
    }
}
