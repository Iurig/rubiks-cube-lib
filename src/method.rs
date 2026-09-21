use crate::puzzles::Puzzle;

pub trait SolveMethod<P: Puzzle> {
    type Options;
}
