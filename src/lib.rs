#![doc = include_str!("../README.md")]
#![feature(
    const_trait_impl,
    const_cmp,
    const_iter,
    const_index,
    const_ops,
    derive_const
)]
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
pub use puzzle::cube3by3::{
    Cube3By3,
    methods::roux::{Cmll, Roux},
    moves::{ParseMoveError, ParseSequenceError},
    pieces::{Center, Corner, Edge},
};
