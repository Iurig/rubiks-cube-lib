use std::{fmt::Debug, ops::Mul};

pub mod cube3by3;
pub trait Puzzle: Default + Mul + Debug {
    type Pieces: Copy + Eq + Debug + 'static;

    const ALL_PIECES: &'static [Self::Pieces];

    #[must_use]
    fn random_state_with_seed(rng: &mut fastrand::Rng) -> Self;

    #[must_use]
    fn random_state() -> Self {
        let mut rng = fastrand::Rng::new();
        Self::random_state_with_seed(&mut rng)
    }
}
