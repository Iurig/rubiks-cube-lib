pub mod cube3x3;
pub mod search;
pub mod simple_methods;

use std::{
    collections::{HashMap, VecDeque},
    fmt::{Debug, Display},
};

use crate::{Mask, ops::Inv, puzzles::Puzzle};

pub trait Solution: 'static {
    type ReconOptions: Default;
    fn new() -> Self;
    fn recon_with_options(&self, options: Self::ReconOptions) -> String;
    #[must_use]
    fn then(&self, next: Self) -> Self;

    fn to_recon(&self) -> String {
        self.recon_with_options(Self::ReconOptions::default())
    }
}

pub trait SolveStep<P: Puzzle, S: Solution, M: SolveMethod<P, S>> {
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
    fn solve(&self, p: &mut P) -> Result<S, Box<dyn std::error::Error>>;
}

pub trait SolveMethod<P: Puzzle, S: Solution>: std::marker::Sized + Default {
    type MethodOptions: Default;

    fn name(&self) -> String;

    fn steps(&self) -> Vec<&impl SolveStep<P, S, Self>>;

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
    fn solve(&self, puzzle: &mut P) -> Result<S, Box<dyn std::error::Error>> {
        let mut sol = S::new();
        let mut solved_pieces = Mask::<P>::default();
        let mut counter = 0;

        while !puzzle.is_solved() {
            if counter >= 100 {
                return Err(Box::new(MethodNotCompletable { name: self.name() }));
            }

            for step in self.steps().iter().filter(|&s| s.options_allow(self)) {
                if step.can_apply(puzzle) && solved_pieces == step.needs_solved() {
                    println!("Starting step: {}", step.name());
                    let step_solution = step.solve(puzzle)?;
                    println!("Finished step: {}", step.name());
                    println!("{}", step_solution.to_recon());
                    solved_pieces = step.solved_pieces();
                    sol = sol.then(step_solution);
                } else {
                    //dbg!(step.step_name(), &solved_pieces, step.needs_solved());
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
        write!(f, "{}", self.name)
    }
}
impl Display for MethodNotCompletable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
