//! A model of the 3×3×3 Rubik's Cube, and a solver that solves it the way a person would, one
//! named step at a time.
//!
//! A cube state is a [`Cube3x3`]. Moves are cube states too, and `a * b` applies `a` and then
//! `b`, in the same order as notation reads:
//!
//! ```
//! use rubiks_cube_lib::{Cube3x3, Inv, Pow, Puzzle};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let scramble = Cube3x3::from_solved("R' U' F D2 L2 F R2 U2 R2 B D2 L B2 D' B2 L' R' B D2 B U2 L U2 R' U' F")?;
//! let solution = Cube3x3::from_solved("D2 F' D2 U2 F' L2 D R2 D B2 F L2 R' F' D U'")?;
//! assert!((scramble * solution).is_solved());
//!
//! assert_eq!(Cube3x3::from_solved("R")?.inverse(), Cube3x3::from_solved("R'")?);
//! assert!(Cube3x3::from_solved("R U R' U'")?.pow(6).is_solved());
//! # Ok(())
//! # }
//! ```
//!
//! Move strings use standard notation: the face turns `R L U D F B`, the slices `M E S`, the
//! rotations `x y z`, and the wide moves `Rw` or `r`, each with no modifier, `'`, `2`, or `2'`.
//! Whitespace separates moves, and `//` starts a comment that runs to the end of the line. A
//! string that is not valid notation returns a [`ParseSequenceError`] naming the line and
//! position of the bad move.
//!
//! # Solving
//!
//! A [`Method`] is an ordered list of [`Step`]s. [`Method::roux`] builds the Roux method, and
//! [`RouxOptions`] chooses between its step variants. Solving returns a [`Solution`], which
//! prints as notation with one line per step:
//!
//! ```no_run
//! use rubiks_cube_lib::{Cube3x3, Method, Puzzle, RouxOptions};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut cube = Cube3x3::from_solved("D2 F2 R2 U L2 D R2 U' B2 L2 B L2 F' L D2 U R' B D2")?;
//! let solution = Method::roux(RouxOptions::default()).solve(&mut cube)?;
//! assert!(cube.is_solved());
//! print!("{solution}");
//! # Ok(())
//! # }
//! ```
//!
//! ```text
//! F' Uw2 Rw Fw M' E' F2    //FB
//! U Rw2 U M' U2 Rw' U Rw2 U R    //SB
//! Rw' D' Rw U Rw' D Rw U' Rw U Rw' U'    //CMLL
//! M U M2 U2 M U' M2 U' M U' M'    //LSE
//! ```
//!
//! To build other methods, combine [`SearchStep`]s, which search for a goal given as a
//! [`Mask`], and [`Choose`], which keeps the shortest of several alternatives. Any type that
//! implements [`Step`] can join them. [`Method::solve_steps`] runs a method one step per
//! iteration, for timing or progress reports.
//!
//! # Errors
//!
//! A solve that fails returns a [`SolveError`] naming the step it stopped at. It wraps the
//! step's [`StepError`], or says the step returned without meeting its goal.
//!
//! # Lower levels
//!
//! [`Cube3x3`] is built from three [`PieceConfiguration`]s (centers, corners, and edges), each a
//! permutation of [`Piece`]s with an orientation per piece in [`zn::Zn`]. The [`Puzzle`] trait
//! is what the solver needs from a puzzle, and [`Facelets`] shows a cube as a sticker net.
mod methods;
mod ops;
mod piece;
mod puzzles;
pub mod zn;

pub use methods::{
    Method, Solution, SolveError, Step, StepError, choose::Choose, cube3x3::roux::RouxOptions,
    search_step::SearchStep,
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

/// Runs the Rust examples in `README.md` as doc tests, so they cannot drift from the code.
/// This item exists only while doc tests run and never appears in the docs.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
