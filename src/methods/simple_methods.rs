use std::collections::{HashMap, VecDeque};

use crate::Inv;
#[allow(clippy::wildcard_imports)]
use crate::{
    Cube3x3,
    methods::*,
    puzzles::cube3by3::{moves::Move3x3, pieces::Pieces3x3},
};

#[derive(Clone, Debug)]
pub struct NamedMoveSequence3x3(String, Vec<Move3x3>);
#[derive(Debug)]
pub struct NamedSolution3x3(Vec<NamedMoveSequence3x3>);
#[derive(Clone)]
pub struct SimpleStep3x3 {
    pub name: String,
    pub before: Box<[Pieces3x3]>,
    pub after: Box<[Pieces3x3]>,
    pub allowed: fn(&Move3x3) -> bool,
}
#[derive(Default)]
pub struct NoOptions();
pub struct SimpleMethod3x3(pub Vec<SimpleStep3x3>);

impl Solution for NamedSolution3x3 {
    type ReconOptions = NoOptions;
    fn new() -> Self {
        Self(Vec::new())
    }
    fn to_recon(&self, _: Self::ReconOptions) -> String {
        self.0
            .iter()
            .map(|NamedMoveSequence3x3(name, move_sequence)| {
                move_sequence
                    .iter()
                    .map(Move3x3::to_string)
                    .collect::<Vec<String>>()
                    .join(" ")
                    + "\t//"
                    + name
                    + "\n"
            })
            .collect::<String>()
    }
    fn then(&self, next: Self) -> Self {
        let mut concatenation = self.0.clone();
        concatenation.extend(next.0);
        Self(concatenation)
    }
}

impl SolveStep<Cube3x3, NamedSolution3x3, SimpleMethod3x3> for SimpleStep3x3 {
    fn can_apply(&self, cube: &Cube3x3) -> bool {
        (*self.before)
            .iter()
            .all(|piece| cube.piece_at(piece) == *piece && cube.orientation_at(piece) == 0)
    }
    fn options_allow(
        &self,
        _options: &<SimpleMethod3x3 as SolveMethod<Cube3x3, NamedSolution3x3>>::MethodOptions,
    ) -> bool {
        true
    }
    fn step_is_solved(&self, cube: &Cube3x3) -> bool {
        (*self.after)
            .iter()
            .all(|piece| cube.piece_at(piece) == *piece && cube.orientation_at(piece) == 0)
    }
    #[expect(clippy::panic)]
    fn solve(&self, p: &mut Cube3x3) -> NamedSolution3x3 {
        let mut to_investigate = VecDeque::from([(*p, None, 0)]);
        let mut investigated: HashMap<Cube3x3, Option<Move3x3>> = HashMap::new();
        let mut prev_depth = 0;
        let available_moves: Vec<Move3x3> = Cube3x3::ALL_MOVES
            .iter()
            .filter(|&m| (self.allowed)(m))
            .copied()
            .collect();
        while let Some((current_cube, current_move, depth)) = to_investigate.pop_front() {
            if investigated.contains_key(&current_cube) {
                continue;
            }
            investigated.insert(current_cube, current_move);
            for &m in &available_moves {
                let moved_cube = current_cube * m;
                if self.step_is_solved(&moved_cube) {
                    *p = moved_cube;
                    let mut solution = VecDeque::from([m]);
                    let mut cube = current_cube;
                    while let Some(backtracking_move) = investigated[&cube] {
                        solution.push_front(backtracking_move);
                        cube = cube * backtracking_move.inverse();
                    }
                    return NamedSolution3x3(vec![NamedMoveSequence3x3(
                        self.name.clone(),
                        Vec::from(solution),
                    )]);
                }
                to_investigate.push_back((moved_cube, Some(m), depth + 1));
            }
            if depth != prev_depth {
                println!("{depth}");
                prev_depth = depth;
            }
        }
        panic!("Step is unsolvable");
    }
}

impl SolveMethod<Cube3x3, NamedSolution3x3> for SimpleMethod3x3 {
    type MethodOptions = NoOptions;

    fn steps(&self) -> Vec<impl SolveStep<Cube3x3, NamedSolution3x3, Self>> {
        self.0.clone()
    }
}
