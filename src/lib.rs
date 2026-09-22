#![doc = include_str!("../README.md")]
mod methods;
mod ops;
mod piece;
mod puzzles;
pub mod zn;

pub use methods::{
    Solution, SolveMethod, SolveStep,
    roux::{CMLL, FB, LSE, ROUX, SB},
    simple_methods::{NoOptions, SimpleMethod3x3, SimpleStep3x3},
};
pub use ops::{Inv, Pow};
pub use piece::{Piece, PieceConfiguration};
pub use puzzles::Puzzle;
pub use puzzles::cube3by3::{
    Cube3x3,
    moves::{ParseMoveError, ParseSequenceError},
    pieces::{Center, Corner, Edge},
};
