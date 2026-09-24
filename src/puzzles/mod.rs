use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::Mul,
};

use crate::Mask;

pub mod cube3by3;
pub mod mask;

pub trait Puzzle:
    Default + Mul<Self::Moves, Output = Self> + Debug + Clone + Eq + Hash + 'static
{
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
