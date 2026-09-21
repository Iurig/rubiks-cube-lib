pub mod cube3by3;
pub trait Puzzle {
    fn random_state_with_seed(rng: &mut fastrand::Rng) -> Self;

    #[must_use]
    fn random_state() -> Self
    where
        Self: Sized,
    {
        let mut rng = fastrand::Rng::new();
        Self::random_state_with_seed(&mut rng)
    }
}
