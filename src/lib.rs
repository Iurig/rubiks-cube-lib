#![doc = include_str!("../README.md")]
#![feature(const_trait_impl)]
mod method;
mod ops;
mod piece;
mod puzzle;
pub mod zn;

pub use method::Method;
pub use method::{Step, StepByPiece, SteppedMethod, do_all};
pub use ops::{Inv, Pow};
pub use piece::{Piece, PieceConfiguration};
pub use puzzle::Puzzle;
pub use puzzle::cube3by3::Cube3By3;
pub use puzzle::cube3by3::moves::{ParseMoveError, ParseSequenceError};
pub use puzzle::cube3by3::pieces::{Center, Corner, Edge};
