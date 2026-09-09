use crate::ops::Inv;
use crate::zn::ZnRing;

/// # Safety
/// Implementors must be fieldless `#[repr(u8)]` enums whose discriminants
/// are `0..N`, in the same order as `ALL`.
pub unsafe trait Piece<const N: usize>: Copy + Eq {
    const ALL: [Self; N];

    fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }
}

#[must_use]
pub const fn index<P, const N: usize>(piece: P) -> usize
where
    P: Piece<N>,
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
    P: Piece<N>,
{
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl<P, const N: usize, const O: usize> Inv for PieceConfiguration<P, N, O>
where
    P: Piece<N>,
{
    fn inverse(&self) -> Self {
        self.const_inverse()
    }
}

impl<P, const N: usize, const O: usize> PieceConfiguration<P, N, O>
where
    P: Piece<N>,
{
    /// The solved state for a given piece type
    pub const IDENTITY: Self = Self {
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
        let mut resp = Self::IDENTITY;
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
        let mut inv = Self::IDENTITY;
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
            assert_eq!(SingleCorner::from_index(i), Some(*c));
        }
        for (i, e) in SingleEdge::ALL.iter().enumerate() {
            assert_eq!(*e as usize, i);
            assert_eq!(SingleEdge::from_index(i), Some(*e));
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
