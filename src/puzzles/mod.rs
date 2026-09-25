use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::Mul,
};

pub mod cube3by3;
pub mod mask;

/// A twisty puzzle the solver can work on. [`Cube3x3`](crate::Cube3x3) is the only one so far.
///
/// Import this trait to call its methods on a cube, such as [`is_solved`](Puzzle::is_solved).
/// A puzzle state is a value; applying a move with `*` returns the new state.
pub trait Puzzle:
    Default + Mul<Self::Moves, Output = Self> + Debug + Clone + Eq + Hash + Send + Sync + 'static
{
    /// One piece of the puzzle. A piece also names its home slot.
    type Piece: Copy + Eq + Debug + Hash + 'static + Send + Sync;
    /// One move of the puzzle, such as `R'` on the cube.
    type Moves: crate::Inv + Eq + Copy + 'static + Display + Debug + Send + Sync;

    /// Every piece of the puzzle, each once.
    const ALL_PIECES: &'static [Self::Piece];
    /// The moves a step may use when it allows any move. For the cube this is every face,
    /// slice, and wide move with each modifier, and no rotations.
    const ALL_MOVES: &'static [Self::Moves];

    /// The slot where `piece` sits now.
    fn piece_location(&self, piece: &Self::Piece) -> Self::Piece;

    /// Whether the puzzle is solved, ignoring how the whole puzzle is rotated.
    #[must_use]
    fn is_solved(&self) -> bool;

    /// The piece sitting in `slot` now. `piece_at(slot) == slot` when that piece is home.
    fn piece_at(&self, slot: &Self::Piece) -> Self::Piece;

    /// The orientation of the piece in `slot`: 0 when it is oriented, or how far it is twisted
    /// or flipped.
    fn orientation_at(&self, slot: &Self::Piece) -> usize;

    /// A random reachable state. The same seed always gives the same state.
    #[must_use]
    fn random_state_with_seed(seed: u64) -> Self;

    /// A random reachable state from a random seed.
    #[must_use]
    fn random_state() -> Self {
        Self::random_state_with_seed(fastrand::u64(..))
    }

    #[expect(
        clippy::panic,
        reason = "`ALL_PIECES` lists every piece, so the loop always returns"
    )]
    /// The position of `piece` in [`ALL_PIECES`](Self::ALL_PIECES).
    ///
    /// # Panics
    /// If `ALL_PIECES` does not list `piece`, which breaks the constant's contract.
    #[must_use]
    fn index(piece: Self::Piece) -> usize {
        for (i, &p) in Self::ALL_PIECES.iter().enumerate() {
            if p == piece {
                return i;
            }
        }
        panic!("ALL_PIECES doesn't contain all values of type Pieces");
    }
}
