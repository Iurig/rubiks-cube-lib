pub(crate) mod cube3by3;

pub struct Move<P: Puzzle> {
    part: P::MovablePart,
    modifier: P::MoveModifier,
}

impl<P: Puzzle> std::fmt::Display for Move<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let part_str = self.part.to_string();
        let modifier_str = self.modifier.to_string();
        write!(f, "{}{}", part_str, modifier_str)
    }
}

pub trait Puzzle:
    std::ops::Mul<Self, Output = Self> + std::ops::Mul<Move<Self>, Output = Self> + Clone
{
    type MovablePart: std::fmt::Display;
    type MoveModifier: std::fmt::Display;
    type PuzzlePieces: crate::piece::Piece;
    const IDENTITY: Self;
}
