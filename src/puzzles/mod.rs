use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::Mul,
};

pub mod cube3by3;
pub trait Puzzle: Default + Mul<Self::Moves, Output = Self> + Debug + Clone + 'static {
    /// The type that represents the puzzle's pieces
    type Pieces: Copy + Eq + Debug + Hash + 'static;
    /// The type that represents a move sequence: usually implemented as a `&'static [Move]` for a type `Move` that represents a move for the puzzle
    type Moves: crate::Inv + Copy + 'static + Display;

    const ALL_PIECES: &'static [Self::Pieces];
    const ALL_MOVES: &'static [Self::Moves];

    fn piece_location(&self, piece: &Self::Pieces) -> Self::Pieces;

    #[must_use]
    fn is_solved(&self) -> bool;

    fn piece_at(&self, slot: &Self::Pieces) -> Self::Pieces;

    fn orientation_at(&self, slot: &Self::Pieces) -> usize;

    #[must_use]
    fn random_state_with_seed(rng: &mut fastrand::Rng) -> Self;

    #[must_use]
    fn random_state() -> Self {
        let mut rng = fastrand::Rng::new();
        Self::random_state_with_seed(&mut rng)
    }
}
