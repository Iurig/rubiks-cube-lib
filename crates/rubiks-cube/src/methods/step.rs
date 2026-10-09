use crate::{Puzzle, Solution, StepError};

use std::fmt::Debug;

pub mod choose;
pub mod combine_pruned;
pub mod search_step;

/// One stage of a solving method, such as building the first block in Roux.
///
/// A [`Technique`](crate::Technique) runs its steps in order. After each step it calls
/// [`is_done`](Step::is_done), and stops with a [`SolveError`](crate::SolveError) if the step
/// failed or its goal is not met.
///
/// The crate has two step types: [`SearchStep`](crate::SearchStep) searches for its goal, and
/// [`Shortest`](crate::Shortest) runs several steps and keeps the shortest result. Implement this
/// trait to write another kind, such as a step that follows hand-written rules. A step is
/// `Send + Sync` so that one built step can be shared by many methods through an `Arc`.
pub trait Step<P: Puzzle>: Send + Sync + Debug {
    /// The step's name, as it appears in errors and in the [`Solution`] it returns.
    fn name(&self) -> &str;

    /// Whether `puzzle` meets this step's goal.
    fn is_done(&self, puzzle: &P) -> bool;

    /// Moves `puzzle` to a state that meets this step's goal and returns the moves it applied.
    ///
    /// The returned moves, applied to `puzzle` as it was, must give `puzzle` as this function
    /// leaves it. A printed [`Solution`] replays only if every step keeps this promise.
    ///
    /// # Errors
    /// A [`StepError`] when the step cannot start on `puzzle` or cannot reach its goal. The
    /// step may leave `puzzle` changed when it fails.
    fn solve(&self, puzzle: &mut P) -> Result<Solution<P>, StepError>;

    /// Whether `puzzle` meets this step's pre-requisites. Expensive for expensive-to-solve steps -
    /// override recommended for those.
    fn can_solve(&self, puzzle: &P) -> bool {
        let mut puzzle_copy = puzzle.clone();
        match self.solve(&mut puzzle_copy) {
            Ok(_) => self.is_done(&puzzle_copy),
            Err(_) => false,
        }
    }
}
