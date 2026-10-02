use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Add, Mul},
};

pub mod algset;
pub mod cube3x3;
pub mod mask;

/// A twisty puzzle the solver can work on. [`Cube3x3`](crate::Cube3x3) is the only one so far.
///
/// Import this trait to call its methods on a cube, such as [`is_solved`](Puzzle::is_solved).
/// A puzzle state is a value; applying a move with `*` returns the new state.
///
/// # Laws
///
/// Generic code, such as a [`Mask`](crate::Mask), relies on every implementation keeping these:
///
/// 1. `Self::default()` is the solved state, and the identity for moves.
/// 2. A move acts the same way on every state. For any state `p`, move `m`, and slot `s`, let
///    `action = Self::default() * m` and `src = action.piece_at(&s)`, the slot `m` moves into `s`.
///    Then `(p * m).piece_at(&s) == p.piece_at(&src)`, and `(p * m).orientation_at(&s) ==
///    p.orientation_at(&src) + action.orientation_at(&s)`.
///
/// So reading `piece_at` and `orientation_at` at every slot of `Self::default() * m` gives
/// everything `m` does to any state.
pub trait Puzzle:
    Default + Mul<Self::Moves, Output = Self> + Debug + Clone + Eq + Hash + Send + Sync + 'static
{
    /// One piece of the puzzle. A piece also names its home slot.
    type Piece: Copy + Eq + Debug + Hash + 'static + Send + Sync;
    /// How a piece is oriented in its slot. One type covers every kind of piece, so a puzzle with
    /// several kinds, such as the cube's corners and edges, uses an enum with one variant per kind.
    type Orientation: Debug
        + Eq
        + Hash
        + Clone
        + Copy
        + Send
        + Sync
        + Add<Output = Self::Orientation>;
    /// One move of the puzzle, such as `R'` on the cube.
    type Moves: crate::Inv + Eq + Copy + 'static + Display + Debug + Send + Sync + Hash;

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

    /// The orientation of the piece in `slot`. It is oriented when this equals the solved
    /// state's orientation at `slot`.
    fn orientation_at(&self, slot: &Self::Piece) -> Self::Orientation;

    /// A random scrambled state. The same seed always gives the same state.
    #[must_use]
    fn scramble_with_seed(seed: u64) -> Self;

    /// A random scrambled state from a random seed.
    #[must_use]
    fn scramble() -> Self {
        Self::scramble_with_seed(fastrand::u64(..))
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

/// A sequence of moves applied in order, such as an algorithm or a step's solution.
pub type Algorithm<P> = Vec<<P as Puzzle>::Moves>;
