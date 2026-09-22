pub mod roux;
pub mod simple_methods;

use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
};

use crate::{ops::Inv, puzzles::Puzzle};

pub trait Solution {
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
    fn options_allow(&self, options: &M::MethodOptions) -> bool;
    fn step_is_solved(&self, p: &P) -> bool;
    fn can_apply(&self, p: &P) -> bool;
    fn step_name(&self) -> String;
    fn allowed_move_sequences(&self) -> Vec<Vec<P::Moves>>;

    type PartialCube: Eq + Hash;
    fn mask(&self, puzzle: &P) -> Self::PartialCube;

    fn solve(&self, p: &mut P) -> S;

    #[expect(clippy::panic)]
    fn solve_bfs(&self, p: &mut P) -> Vec<P::Moves> {
        let mut to_investigate = VecDeque::from([(p.clone(), None, 0)]);
        let mut investigated: HashMap<Self::PartialCube, Option<Vec<P::Moves>>> = HashMap::new();
        let mut prev_depth = 0;

        while let Some((current_cube, current_move_sequence, depth)) = to_investigate.pop_front() {
            if investigated.contains_key(&self.mask(&current_cube)) {
                continue;
            }
            investigated.insert(self.mask(&current_cube), current_move_sequence);
            for sequence in self.allowed_move_sequences() {
                let moved_cube = sequence.iter().fold(current_cube.clone(), |c, m| c * *m);
                if self.step_is_solved(&moved_cube) {
                    *p = moved_cube;
                    let mut solution = VecDeque::from([sequence]);
                    let mut cube = current_cube;
                    while let Some(backtracking_move) = investigated
                        .get(&self.mask(&cube))
                        .expect("previously investigated cube was not found")
                    {
                        solution.push_front(backtracking_move.clone());
                        cube = backtracking_move
                            .iter()
                            .rev()
                            .fold(cube, |c, m| c * m.inverse());
                    }
                    return solution.iter().flatten().copied().collect();
                }
                to_investigate.push_back((moved_cube, Some(sequence), depth + 1));
            }
            if depth != prev_depth {
                println!("Step: {}\t Depth:{depth}", self.step_name());
                prev_depth = depth;
            }
        }
        panic!("Step is unsolvable");
    }
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
                println!("Starting step: {}", step.step_name());
                let step_solution = step.solve(puzzle);
                println!("Finished step: {}", step.step_name());
                println!("{}", step_solution.to_recon());
                sol = sol.then(step_solution);
            }
        }
        sol
    }
}
