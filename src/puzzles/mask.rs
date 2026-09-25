use std::{collections::HashSet, iter::IntoIterator};

use crate::Puzzle;

/// A condition on a puzzle's pieces: which must sit in their home slots, and which slots must
/// hold an oriented piece.
///
/// Steps use masks for where they start and what they solve. A mask ignores every piece it does
/// not name.
///
/// ```
/// use rubiks_cube_lib::{Cube3x3, Edge, Mask, Pieces3x3};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let uf_solved = Mask::<Cube3x3>::new_from_pieces([Pieces3x3::Edge(Edge::Uf)]);
/// assert!(uf_solved.applies_to(&Cube3x3::from_solved("R")?));
/// assert!(!uf_solved.applies_to(&Cube3x3::from_solved("U")?));
/// # Ok(())
/// # }
/// ```
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct Mask<P: Puzzle> {
    pub(crate) permutation: Box<[Option<P::Piece>]>,
    pub(crate) orientation: Box<[Option<usize>]>,
}

impl<P: Puzzle> Default for Mask<P> {
    fn default() -> Self {
        Self::new_empty()
    }
}

impl<P: Puzzle> Mask<P> {
    /// A mask that names no piece, so every puzzle meets it. The same as `Mask::default()`.
    #[must_use]
    pub fn new_empty() -> Self {
        Self {
            permutation: (0..P::ALL_PIECES.len()).map(|_| None).collect(),
            orientation: (0..P::ALL_PIECES.len()).map(|_| None).collect(),
        }
    }

    /// A mask where each piece in `permutations` must sit in its home slot, and the home slot of
    /// each piece in `orientations` must hold an oriented piece.
    ///
    /// The two lists are independent. A piece only in `orientations` asks that whatever piece
    /// sits in its slot be oriented, which is how Roux's corner orientation step (`CO`) says
    /// "orient the last-layer corners, in any order".
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

    /// A mask where each piece in `pieces` must be solved: home and oriented.
    pub fn new_from_pieces<I>(pieces: I) -> Self
    where
        I: IntoIterator<Item = P::Piece> + Clone,
    {
        Self::new(pieces.clone(), pieces)
    }

    #[expect(
        clippy::indexing_slicing,
        reason = "cannot panic if fields of Mask always are of the same size as P::ALL_PIECES, which should be maintained invariantly in all functions"
    )]
    /// Whether `puzzle` meets every condition in this mask.
    #[must_use]
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
