use std::fmt::Debug;

use crate::ops::Inv;
use crate::zn::Zn;
pub mod private {
    pub trait Sealed {}
}

/// One of the `N` pieces of a kind, named by its home slot.
pub trait Piece<const N: usize>: Copy + Eq + private::Sealed + Debug {
    /// Every piece, in slot order.
    fn all_pieces() -> [Self; N];

    /// The piece at position `index` of [`Self::ALL`], if any.
    #[must_use]
    fn from_index(index: usize) -> Option<Self> {
        Self::all_pieces().get(index).copied()
    }

    #[must_use]
    fn random_permutation() -> [Self; N] {
        let mut rng = fastrand::Rng::new();
        Self::random_permutation_with_seed(&mut rng)
    }

    /// Random permutation of a piece set
    #[must_use]
    fn random_permutation_with_seed(rng: &mut fastrand::Rng) -> [Self; N] {
        let mut permutation = Self::all_pieces();
        for i in 0..N {
            permutation.swap(i, rng.usize(i..N));
        }
        permutation
    }
}

#[must_use]
#[allow(clippy::redundant_pub_crate)]
pub(crate) fn index<P, const N: usize>(piece: P) -> usize
where
    P: Piece<N>,
{
    unsafe { (&raw const piece).cast::<u8>().read() as usize }
}

/// Where each of `N` pieces sits and how it is oriented, mod `O`.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct PieceConfiguration<P, const N: usize, const O: usize> {
    pub(crate) permutation: [P; N],
    pub(crate) orientation: [Zn<O>; N],
}

impl<P, const N: usize, const O: usize> Default for PieceConfiguration<P, N, O>
where
    P: Piece<N>,
{
    fn default() -> Self {
        Self::identity()
    }
}

impl<P, const N: usize, const O: usize> Inv for PieceConfiguration<P, N, O>
where
    P: Piece<N>,
{
    #[expect(
        clippy::indexing_slicing,
        reason = "wrong indexing should panic instead of failing silently"
    )]
    fn inverse(&self) -> Self {
        let mut inv = Self::identity();
        for ((&piece, orientation), home) in self
            .permutation
            .iter()
            .zip(&self.orientation)
            .zip(P::all_pieces())
        {
            inv.permutation[index(piece)] = home;
            inv.orientation[index(piece)] = orientation.const_neg();
        }
        inv
    }
}

impl<P, const N: usize, const O: usize> PieceConfiguration<P, N, O>
where
    P: Piece<N>,
{
    /// The solved state for a given piece type
    #[must_use]
    pub fn identity() -> Self {
        Self {
            permutation: P::all_pieces(),
            orientation: [Zn::ZERO; N],
        }
    }

    /// The piece now sitting in `slot`.
    ///
    /// Slots and pieces share a type: a piece is named by its home slot. So
    /// `piece_at(Ubr) == Ufr` reads "the UFR piece sits in the UBR slot", and
    /// the returned piece can be fed back in as the next slot when tracing a
    /// cycle, the way blind memorization does.
    ///
    /// # Panics
    /// Only panics under internal logic error
    #[must_use]
    pub fn piece_at(&self, slot: P) -> P {
        unsafe { *self.permutation.get_unchecked(index(slot)) }
    }

    /// The orientation held at `slot`: the twist or flip of the piece sitting
    /// there, relative to the slot.
    ///
    /// Centers have no orientation, so on a center configuration this always
    /// returns zero.
    #[must_use]
    pub fn orientation_at(&self, slot: P) -> Zn<O> {
        unsafe { *self.orientation.get_unchecked(index(slot)) }
    }

    /// # Panics
    ///
    /// Panics if `self.permutation` is not a valid permutation
    #[must_use]
    pub(crate) fn parity(&self) -> Zn<2> {
        let mut visited = Vec::new();
        let mut par = Zn::new(0);
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
                par = par + Zn::new(cycle_size - 1);
            }
        }
        par
    }

    #[must_use]
    pub(crate) fn orientation_sum(&self) -> Zn<O> {
        self.orientation
            .iter()
            .fold(Zn::new(0), |prev, &next| prev + next)
    }

    /// Compose permutations done by `self` with `other`
    #[must_use]
    pub(crate) fn then(&self, other: &Self) -> Self {
        let mut composed = Self::identity();
        for i in 0..N {
            composed.permutation[i] = self.permutation[index(other.permutation[i])];
            composed.orientation[i] =
                self.orientation[index(other.permutation[i])].const_add(other.orientation[i]);
        }
        composed
    }

    pub(crate) fn cycle<const CYCLE_SIZE: usize, const CYCLE_AMOUNT: usize>(
        to_cycle: [[P; CYCLE_SIZE]; CYCLE_AMOUNT],
    ) -> Self {
        let mut resp = Self::identity();
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

    pub fn random_state_with_seed(rng: &mut fastrand::Rng) -> Self {
        Self {
            permutation: P::random_permutation_with_seed(rng),
            orientation: {
                let mut or = [Zn::ZERO; N];
                for flip in or.iter_mut().take(N) {
                    *flip = Zn::new(rng.usize(0..O));
                }
                or
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzles::cube3by3::pieces::*;
    #[test]
    fn corner_and_edge_all_match_discriminants() {
        for (i, c) in Corner::all_pieces().iter().enumerate() {
            assert_eq!(*c as usize, i);
            assert_eq!(Corner::from_index(i), Some(*c));
        }
        for (i, e) in Edge::all_pieces().iter().enumerate() {
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
        assert_eq!(perm, Corner::all_pieces());
    }

    #[test]
    fn cycle_with_disjoint_cycles_is_a_permutation() {
        use Edge::{Dl, Dr, Ub, Uf};
        let mut perm = EdgeConfiguration::cycle::<2, 2>([[Ub, Uf], [Dl, Dr]]).permutation;
        assert_eq!(perm[Uf as usize], Ub);
        assert_eq!(perm[Ub as usize], Uf);
        perm.sort();
        assert_eq!(perm, Edge::all_pieces());
    }
}
