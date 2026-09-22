use std::{fmt::Debug, ops::Mul};

pub mod cube3by3;
pub trait Puzzle: Default + Mul + Debug {
    type Pieces: Copy + Eq + Debug + 'static;
    type Moves: 'static;

    const ALL_PIECES: &'static [Self::Pieces];
    const ALL_MOVES: &'static [Self::Moves];

    fn piece_location(&self, piece: &Self::Pieces) -> Self::Pieces;

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
