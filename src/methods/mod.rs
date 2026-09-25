pub mod cube3x3;
pub mod search;
pub mod simple_methods;

use std::{
    collections::{HashMap, VecDeque},
    fmt::{Debug, Display},
};

use crate::{Mask, ops::Inv, puzzles::Puzzle};

pub trait SolveStep<P: Puzzle, M: SolveMethod<P>> {
    fn options_allow(&self, method: &M) -> bool;
    fn step_is_solved(&self, p: &P) -> bool;
    fn can_apply(&self, p: &P) -> bool;
    fn name(&self) -> String;
    fn allowed_move_sequences(&self) -> Vec<Vec<P::Moves>>;

    fn needs_solved(&self) -> Mask<P>;
    fn solved_pieces(&self) -> Mask<P>;
    fn mask(&self, puzzle: &P) -> Mask<P>;

    /// # Errors
    /// Errors if the moves allowed by the step can't finish the step
    fn solve(&self, p: &mut P) -> Result<Solution<P>, Box<dyn std::error::Error>>;
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

    fn from_iter<I, D>(iter: I) -> Self
    where
        I: IntoIterator<Item = (Vec<P::Moves>, D)>,
        D: Display,
    {
        Self {
            step_solutions: Vec::from_iter(iter.into_iter().map(|(v, d)| (v, d.to_string()))),
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

pub trait SolveMethod<P: Puzzle>: std::marker::Sized + Default {
    type MethodOptions: Default;

    fn name(&self) -> String;

    fn steps(&self) -> Vec<&impl SolveStep<P, Self>>;

    fn from_options(options: Self::MethodOptions) -> Self;

    fn to_options(&self) -> &Self::MethodOptions;

    #[must_use]
    fn new() -> Self {
        Self::default()
    }

    /// # Errors
    /// Errors if any step of the method errors or if the steps can't be combined to solve a cube,
    /// determined if 100 steps are applied and the cube is still unsolved, or if one of its steps
    /// isn't completable
    fn solve(&self, puzzle: &mut P) -> Result<Solution<P>, Box<dyn std::error::Error>> {
        let mut sol = Solution::new();
        let mut solved_pieces = Mask::<P>::default();
        let mut counter = 0;

        while !puzzle.is_solved() {
            if counter >= 100 {
                return Err(Box::new(MethodNotCompletable { name: self.name() }));
            }

            for step in self.steps().iter().filter(|&s| s.options_allow(self)) {
                if step.can_apply(puzzle) && solved_pieces == step.needs_solved() {
                    log::debug!("Starting step: {}", step.name());
                    let step_solution = step.solve(puzzle)?;
                    log::debug!("Finished step: {}: {}", step.name(), step_solution);
                    solved_pieces = step.solved_pieces();
                    sol.extend(step_solution);
                }
            }
            counter += 1;
        }
        Ok(sol)
    }
}

#[derive(Debug)]
pub struct StepNotCompletable {
    name: String,
}
#[derive(Debug)]
pub struct MethodNotCompletable {
    name: String,
}

impl std::error::Error for StepNotCompletable {}
impl std::error::Error for MethodNotCompletable {}

impl Display for StepNotCompletable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "step {} cannot be completed with its allowed moves",
            self.name
        )
    }
}
impl Display for MethodNotCompletable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "method {} did not solve the cube after 100 passes over its steps",
            self.name
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
