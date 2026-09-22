pub mod roux;
pub mod simple_methods;

use crate::puzzles::Puzzle;

pub trait Solution {
    type ReconOptions: Default;
    fn new() -> Self;
    fn to_recon(&self, options: Self::ReconOptions) -> String;
    #[must_use]
    fn then(&self, next: Self) -> Self;
}

pub trait SolveStep<P: Puzzle, S: Solution, M: SolveMethod<P, S>> {
    fn options_allow(&self, options: &M::MethodOptions) -> bool;
    fn step_is_solved(&self, p: &P) -> bool;
    fn can_apply(&self, p: &P) -> bool;
    fn solve(&self, p: &mut P) -> S;
}

pub trait SolveMethod<P: Puzzle, S: Solution>: std::marker::Sized {
    type MethodOptions: Default;

    fn steps(&self) -> Vec<impl SolveStep<P, S, Self>>;

    fn solve(&self, puzzle: &mut P) -> S {
        self.solve_with_options(puzzle, Self::MethodOptions::default())
    }

    fn solve_with_options(&self, puzzle: &mut P, options: Self::MethodOptions) -> S {
        let mut sol = S::new();
        for step in self.steps() {
            if step.can_apply(puzzle) && step.options_allow(&options) {
                sol = sol.then(step.solve(puzzle));
                println!("yo");
            }
        }
        sol
    }
}
