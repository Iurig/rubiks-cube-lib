#![doc = include_str!("../README.md")]
mod method;
mod ops;
mod piece;
mod puzzles;
pub mod zn;

pub use method::SolveMethod;
pub use ops::{Inv, Pow};
pub use piece::{Piece, PieceConfiguration};
pub use puzzles::Puzzle;
pub use puzzles::cube3by3::{
    Cube3By3,
    moves::{ParseMoveError, ParseSequenceError},
    pieces::{Center, Corner, Edge},
};
