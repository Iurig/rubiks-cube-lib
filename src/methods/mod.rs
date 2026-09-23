pub mod roux;
pub mod search;
pub mod simple_methods;

use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
};

use crate::{ops::Inv, puzzles::Puzzle};

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
    fn needs_solved(&self) -> Vec<P::Pieces>;
    fn solved_pieces(&self) -> Vec<P::Pieces>;
    fn step_name(&self) -> String;
    fn allowed_move_sequences(&self) -> Vec<Vec<P::Moves>>;

    type PartialCube: Eq + Hash;
    fn mask(&self, puzzle: &P) -> Self::PartialCube;

    fn solve(&self, p: &mut P) -> S;
}

pub trait SolveMethod<P: Puzzle, S: Solution>: std::marker::Sized + Default {
    type MethodOptions: Default;

    fn steps(&self) -> Vec<&impl SolveStep<P, S, Self>>;

    fn from_options(options: Self::MethodOptions) -> Self;

    fn to_options(&self) -> &Self::MethodOptions;

    #[must_use]
    fn new() -> Self {
        Self::default()
    }

    fn solve(&self, puzzle: &mut P) -> S {
        let mut sol = S::new();
        let mut solved_pieces = HashSet::new();

        while !puzzle.is_solved() {
            for step in self.steps().iter().filter(|&s| s.options_allow(self)) {
                if step.can_apply(puzzle)
                    && solved_pieces
                        .iter()
                        .all(|p| step.needs_solved().contains(p))
                {
                    println!("Starting step: {}", step.step_name());
                    let step_solution = step.solve(puzzle);
                    println!("Finished step: {}", step.step_name());
                    println!("{}", step_solution.to_recon());
                    solved_pieces.extend(step.solved_pieces());
                    sol = sol.then(step_solution);
                } else {
                    //dbg!(step.step_name(), &solved_pieces, step.needs_solved());
                }
            }
        }
        sol
    }
}
