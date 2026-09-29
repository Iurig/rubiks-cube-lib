use std::{collections::HashSet, iter::IntoIterator};

use crate::Puzzle;

/// A partly defined puzzle state, usually read as a condition: which pieces must sit in their
/// home slots, and which slots must hold an oriented piece.
///
/// [`SearchStep`](crate::SearchStep)s use masks for where they start and what they solve. A mask
/// ignores every piece it does not name.
///
/// ```
/// use rubiks_cube::{Cube3x3, Edge, Mask, Pieces3x3};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let uf_solved = Mask::<Cube3x3>::new_from_pieces([Pieces3x3::Edge(Edge::Uf)]);
/// assert!(uf_solved.applies_to(&Cube3x3::from_solved("R")?));
/// assert!(!uf_solved.applies_to(&Cube3x3::from_solved("U")?));
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Mask<P: Puzzle>(pub(crate) Box<[SlotCondition<P>]>);

/// What a [`Mask`] asks of one slot: which piece must sit there, and which orientation the piece
/// there must have. `None` asks nothing.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SlotCondition<P: Puzzle> {
    pub(crate) piece: Option<P::Piece>,
    pub(crate) orient: Option<P::Orientation>,
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
        Self(
            (0..P::ALL_PIECES.len())
                .map(|_| SlotCondition {
                    piece: None,
                    orient: None,
                })
                .collect(),
        )
    }

    /// A mask where each piece in `pieces` must be solved: home and oriented. Same as
    /// <code>[new](Self::new)(pieces.clone(), pieces)</code>.
    pub fn new_from_pieces<I>(pieces: I) -> Self
    where
        I: IntoIterator<Item = P::Piece> + Clone,
    {
        Self::new(pieces.clone(), pieces)
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
        Self(
            P::ALL_PIECES
                .iter()
                .map(|slot| SlotCondition {
                    piece: perm_iter.contains(slot).then_some(*slot),
                    orient: orient_iter
                        .contains(slot)
                        .then_some(P::default().orientation_at(slot)),
                })
                .collect(),
        )
    }

    /// Whether `puzzle` meets every condition in this mask.
    #[must_use]
    pub fn applies_to(&self, puzzle: &P) -> bool {
        self.0.iter().zip(P::ALL_PIECES).all(|(condition, slot)| {
            condition
                .piece
                .is_none_or(|piece| puzzle.piece_at(slot) == piece)
                && condition
                    .orient
                    .is_none_or(|orient| puzzle.orientation_at(slot) == orient)
        })
    }
}
