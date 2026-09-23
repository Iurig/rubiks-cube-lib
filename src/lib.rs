#![doc = include_str!("../README.md")]
mod methods;
mod ops;
mod piece;
mod puzzles;
pub mod zn;

pub use methods::{
    Solution, SolveMethod, SolveStep,
    roux::{Roux, RouxOptions},
    simple_methods::{NamedMoveSequences, NoOptions, SimpleStep},
};
pub use ops::{Inv, Pow};
pub use piece::{Piece, PieceConfiguration};
pub use puzzles::cube3by3::{
    Cube3x3,
    moves::{ParseMoveError, ParseSequenceError},
    pieces::{Center, Corner, Edge, Pieces3x3},
};
pub use puzzles::{Mask, Puzzle};
