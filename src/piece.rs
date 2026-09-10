use crate::ops::Inv;
use crate::zn::ZnRing;
pub mod private {
    pub trait Sealed {}
}

pub trait Piece<const N: usize>: Copy + Eq + private::Sealed {
    const ALL: [Self; N];

    fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }
}

#[must_use]
#[allow(clippy::redundant_pub_crate)]
pub(crate) const fn index<P, const N: usize>(piece: P) -> usize
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

    /// The piece now sitting in `slot`.
    ///
    /// Slots and pieces share a type: a piece is named by its home slot. So
    /// `piece_at(Ubr) == Ufr` reads "the UFR piece sits in the UBR slot", and
    /// the returned piece can be fed back in as the next slot when tracing a
    /// cycle, the way blind memorization does.
    #[must_use]
    pub const fn piece_at(&self, slot: P) -> P {
        self.permutation[index(slot)]
    }

    /// The orientation held at `slot`: the twist or flip of the piece sitting
    /// there, relative to the slot.
    ///
    /// Centers have no orientation, so on a center configuration this always
    /// returns zero.
    #[must_use]
    pub const fn orientation_at(&self, slot: P) -> ZnRing<O> {
        self.orientation[index(slot)]
    }

    /// # Panics
    ///
    /// Panics if `self.permutation` is not a valid permutation
    #[must_use]
    pub(crate) fn parity(&self) -> ZnRing<2> {
        let mut visited = Vec::new();
        let mut par = ZnRing::new(0);
        for p in self.permutation {
            if !visited.contains(&p) {
                visited.push(p);
                let mut travel = self
                    .permutation
                    .get(index(p))
                    .expect("self.permutation must be a valid permutation");
                let mut cycle_size = 1;
                while travel != &p {
                    visited.push(*travel);
                    cycle_size += 1;
                    travel = self
                        .permutation
                        .get(index(*travel))
                        .expect("self.permutation must be a valid permutation");
                }
                par = par + ZnRing::new(cycle_size - 1);
            }
        }
        par
    }

    #[must_use]
    pub(crate) fn orientation_sum(&self) -> ZnRing<O> {
        self.orientation
            .iter()
            .fold(ZnRing::new(0), |prev, &next| prev + next)
    }

    /// Compose permutations done by `self` with `other`
    #[must_use]
    pub(crate) const fn then(&self, other: &Self) -> Self {
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

    pub(crate) const fn cycle<const CYCLE_SIZE: usize, const CYCLE_AMOUNT: usize>(
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
    pub(crate) const fn const_inverse(&self) -> Self {
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
        for (i, c) in Corner::ALL.iter().enumerate() {
            assert_eq!(*c as usize, i);
            assert_eq!(Corner::from_index(i), Some(*c));
        }
        for (i, e) in Edge::ALL.iter().enumerate() {
            assert_eq!(*e as usize, i);
            assert_eq!(Edge::from_index(i), Some(*e));
        }
    }

    #[test]
    fn cycle_is_a_permutation_and_moves_pieces_forward_and_leaves_rest() {
        use Corner::{Dbr, Dfr, Ubr, Ufr};
        let mut perm = CornerConfiguration::cycle([[Ufr, Ubr, Dbr, Dfr]]).permutation;
        assert_eq!(perm[index(Ubr)], Ufr);
        assert_eq!(perm[index(Dbr)], Ubr);
        assert_eq!(perm[index(Dfr)], Dbr);
        assert_eq!(perm[index(Ufr)], Dfr);
        for c in [Corner::Ubl, Corner::Ufl, Corner::Dfl, Corner::Dbl] {
            assert_eq!(perm[c as usize], c);
        }
        perm.sort();
        assert_eq!(perm, Corner::ALL);
    }

    #[test]
    fn cycle_with_disjoint_cycles_is_a_permutation() {
        use Edge::{Dl, Dr, Ub, Uf};
        let mut perm = EdgeConfiguration::cycle::<2, 2>([[Ub, Uf], [Dl, Dr]]).permutation;
        assert_eq!(perm[Uf as usize], Ub);
        assert_eq!(perm[Ub as usize], Uf);
        perm.sort();
        assert_eq!(perm, Edge::ALL);
    }
}
