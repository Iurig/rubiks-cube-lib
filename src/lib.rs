#![doc = include_str!("../README.md")]
mod methods;
mod ops;
mod piece;
mod puzzles;
pub mod zn;

pub use methods::{
    MethodNotCompletable, Solution, SolveMethod, SolveStep, StepNotCompletable,
    cube3x3::roux::{Roux, RouxOptions},
    simple_methods::{NamedMoveSequences, NoOptions, SimpleStep},
};
pub use ops::{Inv, Pow};
pub use piece::{Piece, PieceConfiguration};
pub use puzzles::cube3by3::{
    Cube3x3,
    facelets::Facelets,
    moves::{ParseMoveError, ParseSequenceError},
    pieces::{Center, Corner, Edge, Pieces3x3},
};
pub use puzzles::{Puzzle, mask::Mask};
