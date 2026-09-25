pub mod cube3x3;
pub mod search;
pub mod simple_methods;

use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    fmt::{Debug, Display},
    sync::Arc,
};

use crate::{ops::Inv, puzzles::Puzzle};

pub trait Step<P: Puzzle>: Send + Sync + Debug {
    fn name(&self) -> &'static str;

    fn is_done(&self, puzzle: &P) -> bool;

    fn allowed_move_sequences(&self) -> Vec<Vec<P::Moves>>;

    /// # Errors
    /// Errors if the moves allowed by the step can't finish the step
    fn solve(&self, puzzle: &mut P) -> Result<Solution<P>, StepError>;
}

#[derive(Debug)]
pub struct Solution<P: Puzzle> {
    step_solutions: Vec<(Vec<P::Moves>, String)>,
}

impl<P: Puzzle> Solution<P> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, (Vec<P::Moves>, String)> {
        self.step_solutions.iter()
    }

    #[must_use]
    pub fn move_count(&self) -> usize {
        self.step_solutions
            .iter()
            .map(|(moves, _)| moves.len())
            .sum()
    }
}

/// Builds a solution from `(moves, step name)` segments, in order.
impl<P: Puzzle, D: Display> FromIterator<(Vec<P::Moves>, D)> for Solution<P> {
    fn from_iter<I: IntoIterator<Item = (Vec<P::Moves>, D)>>(iter: I) -> Self {
        Self {
            step_solutions: iter
                .into_iter()
                .map(|(moves, name)| (moves, name.to_string()))
                .collect(),
        }
    }
}

/// Joins solutions into one, keeping every segment in order.
impl<P: Puzzle> FromIterator<Self> for Solution<P> {
    fn from_iter<I: IntoIterator<Item = Self>>(iter: I) -> Self {
        Self {
            step_solutions: iter.into_iter().flatten().collect(),
        }
    }
}

impl<P: Puzzle> Default for Solution<P> {
    fn default() -> Self {
        Self {
            step_solutions: Vec::<(Vec<P::Moves>, String)>::new(),
        }
    }
}
impl<'a, P: Puzzle> IntoIterator for &'a Solution<P> {
    type Item = &'a (Vec<<P as Puzzle>::Moves>, String);
    type IntoIter = std::slice::Iter<'a, (Vec<<P as Puzzle>::Moves>, String)>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<P: Puzzle> Extend<(Vec<P::Moves>, String)> for Solution<P> {
    fn extend<T: IntoIterator<Item = (Vec<P::Moves>, String)>>(&mut self, iter: T) {
        self.step_solutions.extend(iter);
    }
}
impl<P: Puzzle> IntoIterator for Solution<P> {
    type Item = (Vec<P::Moves>, String);
    type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.step_solutions.into_iter()
    }
}
impl<P: Puzzle> Display for Solution<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.step_solutions
                .iter()
                .map(|(move_sequence, name)| {
                    move_sequence
                        .iter()
                        .map(P::Moves::to_string)
                        .collect::<Vec<String>>()
                        .join(" ")
                        + "\t//"
                        + name
                        + "\n"
                })
                .collect::<String>()
        )
    }
}

#[derive(Debug)]
pub struct Method<P: Puzzle> {
    steps: Vec<Arc<dyn Step<P>>>,
    name: &'static str,
}

impl<P: Puzzle> Method<P> {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub fn steps(&self) -> impl Iterator<Item = Arc<dyn Step<P>>> {
        self.steps.iter().cloned()
    }

    #[must_use]
    fn new(name: &'static str, steps: Vec<Arc<dyn Step<P>>>) -> Self {
        Self { steps, name }
    }

    /// Solves `puzzle` one step at a time. Each call to `next()` runs the next step and yields
    /// its solution.
    pub fn solve_steps(
        &self,
        puzzle: &mut P,
    ) -> impl Iterator<Item = Result<Solution<P>, SolveError>> {
        let mut failed = false;
        self.steps().map_while(move |step| {
            if failed {
                return None;
            }
            let result = match step.solve(puzzle) {
                Err(e) => Err(StepFailures::Internal(e)),
                Ok(_) if !step.is_done(puzzle) => Err(StepFailures::Unsolved),
                Ok(solution) => Ok(solution),
            };
            failed = result.is_err();
            Some(result.map_err(|error| SolveError {
                name: step.name(),
                error,
            }))
        })
    }

    /// Runs every step in order and joins their solutions.
    ///
    /// # Errors
    /// Returns the first step's error, and runs no later steps.
    pub fn solve(&self, puzzle: &mut P) -> Result<Solution<P>, SolveError> {
        self.solve_steps(puzzle).collect()
    }
}

#[derive(Debug)]
pub enum StepError {
    UnrecheableGoal,
    InvalidStartingState,
    /// Another thread panicked while deepening the step's memo, so the memo may be inconsistent.
    MemoPoisoned,
    Custom(Box<dyn Error + Send + Sync>),
}
#[derive(Debug)]
pub struct SolveError {
    name: &'static str,
    error: StepFailures,
}

#[derive(Debug)]
pub enum StepFailures {
    Internal(StepError),
    Unsolved,
}

impl std::error::Error for StepError {}
impl std::error::Error for SolveError {}

impl Display for StepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::UnrecheableGoal =>
                    "goal could not be reached with the given moveset".to_string(),
                Self::InvalidStartingState =>
                    "starting state doesn't fit expected properties".to_string(),
                Self::MemoPoisoned =>
                    "the step's memo is unusable: another solve panicked while deepening it"
                        .to_string(),
                Self::Custom(e) => e.to_string(),
            }
        )
    }
}
impl Display for SolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self.error {
                StepFailures::Unsolved => format!(
                    "step {} returned a solution, but its goal is not met",
                    self.name
                ),
                StepFailures::Internal(e) => format!("step {} could not finish: {}", self.name, e),
            }
        )
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::panic_in_result_fn,
        reason = "`?` reports setup failures; `assert!` reports the property under test failing"
    )]

    use std::error::Error;

    use crate::{Cube3x3, ParseSequenceError, puzzles::cube3by3::moves::Move3x3};

    use super::*;

    #[test]
    fn move_count_counts_correctly() -> Result<(), Box<dyn Error>> {
        let s: Solution<Cube3x3> = Solution::from_iter([(
            Move3x3::sequence("y U2 r M'").collect::<Result<Vec<Move3x3>, ParseSequenceError>>()?,
            "Step 1",
        )]);
        assert_eq!(s.move_count(), 4);
        Ok(())
    }
}
