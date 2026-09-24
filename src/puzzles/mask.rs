use std::{collections::HashSet, iter::IntoIterator};

use crate::Puzzle;

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct Mask<P: Puzzle> {
    pub(crate) permutation: Box<[Option<P::Piece>]>,
    pub(crate) orientation: Box<[Option<usize>]>,
}

impl<P: Puzzle> Mask<P> {
    pub fn new_empty() -> Self {
        Self {
            permutation: (0..P::ALL_PIECES.len()).map(|_| None).collect(),
            orientation: (0..P::ALL_PIECES.len()).map(|_| None).collect(),
        }
    }

    pub fn new<I1, I2>(permutations: I1, orientations: I2) -> Self
    where
        I1: IntoIterator<Item = P::Piece>,
        I2: IntoIterator<Item = P::Piece>,
    {
        let perm_iter: HashSet<<P as Puzzle>::Piece> =
            HashSet::<<P as Puzzle>::Piece>::from_iter(permutations);
        let orient_iter: HashSet<<P as Puzzle>::Piece> =
            HashSet::<<P as Puzzle>::Piece>::from_iter(orientations);
        Self {
            permutation: P::ALL_PIECES
                .iter()
                .map(|piece| perm_iter.contains(piece).then_some(*piece))
                .collect(),
            orientation: P::ALL_PIECES
                .iter()
                .map(|piece| orient_iter.contains(piece).then_some(0))
                .collect(),
        }
    }

    #[expect(
        clippy::indexing_slicing,
        reason = "cannot panic if fields of Mask always are of the same size as P::ALL_PIECES, which should be maintained invariantly in all functions"
    )]
    pub fn applies_to(&self, puzzle: &P) -> bool {
        self.permutation
            .iter()
            .enumerate()
            .all(|(i, &some_p)| some_p.is_none_or(|p| puzzle.piece_at(&P::ALL_PIECES[i]) == p))
            && self.orientation.iter().enumerate().all(|(i, &some_p)| {
                some_p.is_none_or(|p| puzzle.orientation_at(&P::ALL_PIECES[i]) == p)
            })
    }
}
