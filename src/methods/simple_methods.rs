use std::{fmt::Display, marker::PhantomData, sync::Mutex};

use crate::methods::search::BFSMemo;
#[allow(clippy::wildcard_imports)]
use crate::{Puzzle, methods::*};

pub struct SimpleStep<P: Puzzle, M: SolveMethod<P, NamedMoveSequences<P>>> {
    pub name: String,
    pub before: Box<[P::Piece]>,
    pub after: Box<[P::Piece]>,
    pub allowed_moves: Vec<Vec<P::Moves>>,
    pub is_allowed: fn(&M) -> bool,
    pub memo: Mutex<BFSMemo<P>>,
    pub phantom: PhantomData<M>,
}

type MoveSequence<P> = Vec<<P as Puzzle>::Moves>;

pub type NamedMoveSequences<P> = Vec<(String, MoveSequence<P>)>;

pub type SimpleMask<P> = Vec<(<P as Puzzle>::Piece, usize)>;

impl<P: Puzzle, M: SolveMethod<P, NamedMoveSequences<P>>> SimpleStep<P, M> {
    #[expect(clippy::panic, clippy::type_complexity)]
    fn solve_bfs(&self, p: &mut P) -> Vec<P::Moves> {
        let mut to_investigate = VecDeque::from([(p.clone(), None, 0)]);
        let mut investigated: HashMap<
            <Self as SolveStep<P, NamedMoveSequences<P>, M>>::PartialCube,
            Option<MoveSequence<P>>,
        > = HashMap::new();
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

#[derive(Default)]
pub struct NoOptions();

impl<M> Solution for Vec<(String, Vec<M>)>
where
    M: crate::Inv + Copy + 'static + Display,
{
    type ReconOptions = NoOptions;
    fn new() -> Self {
        Self::new()
    }
    fn recon_with_options(&self, _: Self::ReconOptions) -> String {
        self.iter()
            .map(|(name, move_sequence)| {
                move_sequence
                    .iter()
                    .map(M::to_string)
                    .collect::<Vec<String>>()
                    .join(" ")
                    + "\t//"
                    + name
                    + "\n"
            })
            .collect::<String>()
    }
    fn then(&self, next: Self) -> Self {
        let mut concatenation = self.clone();
        concatenation.extend(next);
        concatenation
    }
}

impl<P, M> SolveStep<P, Vec<(String, Vec<P::Moves>)>, M> for SimpleStep<P, M>
where
    P: Puzzle,
    M: SolveMethod<P, Vec<(String, Vec<P::Moves>)>>,
{
    fn options_allow(&self, method: &M) -> bool {
        (self.is_allowed)(method)
    }
    fn can_apply(&self, cube: &P) -> bool {
        (*self.before)
            .iter()
            .all(|piece| cube.piece_at(piece) == *piece && cube.orientation_at(piece) == 0)
    }
    fn needs_solved(&self) -> Vec<<P as Puzzle>::Piece> {
        (*self.before).to_vec()
    }
    fn solved_pieces(&self) -> Vec<<P as Puzzle>::Piece> {
        (*self.after).to_vec()
    }
    fn step_is_solved(&self, cube: &P) -> bool {
        (*self.after)
            .iter()
            .all(|piece| cube.piece_at(piece) == *piece && cube.orientation_at(piece) == 0)
    }
    fn step_name(&self) -> String {
        self.name.clone()
    }
    fn allowed_move_sequences(&self) -> Vec<Vec<<P as Puzzle>::Moves>> {
        self.allowed_moves.clone()
    }

    type PartialCube = SimpleMask<P>;
    fn mask(&self, puzzle: &P) -> Self::PartialCube {
        self.after
            .iter()
            .map(|p| {
                (
                    puzzle.piece_location(p),
                    puzzle.orientation_at(&puzzle.piece_location(p)),
                )
            })
            .collect::<Vec<(P::Piece, usize)>>()
    }
    fn solve(&self, p: &mut P) -> Vec<(String, Vec<P::Moves>)> {
        vec![(
            <Self as SolveStep<P, Vec<(String, Vec<P::Moves>)>, M>>::step_name(self),
            Self::solve_bfs(self, p),
        )]
    }
}
