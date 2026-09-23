use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    hash::Hash,
    iter::IntoIterator,
    ops::Mul,
};

pub mod cube3by3;
pub trait Puzzle: Default + Mul<Self::Moves, Output = Self> + Debug + Clone + Eq + 'static {
    /// The type that represents the puzzle's pieces
    type Piece: Copy + Eq + Debug + Hash + 'static;
    /// The type that represents a move sequence: usually implemented as a `&'static [Move]` for a type `Move` that represents a move for the puzzle
    type Moves: crate::Inv + Copy + 'static + Display;

    /// A slice refference to all pieces in the puzzle, must contain all values `type Piece` can assume
    const ALL_PIECES: &'static [Self::Piece];
    const ALL_MOVES: &'static [Self::Moves];

    fn piece_location(&self, piece: &Self::Piece) -> Self::Piece;

    #[must_use]
    fn is_solved(&self) -> bool;

    fn piece_at(&self, slot: &Self::Piece) -> Self::Piece;

    fn orientation_at(&self, slot: &Self::Piece) -> usize;

    #[must_use]
    fn random_state_with_seed(rng: &mut fastrand::Rng) -> Self;

    #[must_use]
    fn random_state() -> Self {
        let mut rng = fastrand::Rng::new();
        Self::random_state_with_seed(&mut rng)
    }

    #[allow(clippy::panic)]
    #[must_use]
    fn index(piece: Self::Piece) -> usize {
        for (i, &p) in Self::ALL_PIECES.iter().enumerate() {
            if p == piece {
                return i;
            }
        }
        panic!("ALL_PIECES doesn't contain all values of type Pieces");
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Mask<P: Puzzle> {
    permutation: Box<[Option<P::Piece>]>,
    orientation: Box<[Option<usize>]>,
}

impl<P: Puzzle> Mask<P> {
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
