mod table;

#[allow(clippy::wildcard_imports)]
use crate::{
    cube3by3::{Cube3By3, pieces::*},
    ops,
};

use table::cube_state;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum MovablePart {
    Face(Faces),
    Slice(Slices),
    Rotation(Rotations),
    Wide(Faces),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum MoveModifier {
    Clockwise,
    CounterClockwise,
    Double,
    CounterDouble,
}

impl MoveModifier {
    pub const fn inverse(self) -> Self {
        match self {
            Self::Clockwise => Self::CounterClockwise,
            Self::CounterClockwise => Self::Clockwise,
            Self::Double => Self::CounterDouble,
            Self::CounterDouble => Self::Double,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Move {
    part: MovablePart,
    modifier: MoveModifier,
}

impl Move {
    pub const fn new(part: MovablePart, modifier: MoveModifier) -> Self {
        Self { part, modifier }
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let part_string = match self.part {
            MovablePart::Face(Faces::R) => "R",
            MovablePart::Face(Faces::F) => "F",
            MovablePart::Face(Faces::U) => "U",
            MovablePart::Face(Faces::L) => "L",
            MovablePart::Face(Faces::D) => "D",
            MovablePart::Face(Faces::B) => "B",
            MovablePart::Rotation(Rotations::x) => "x",
            MovablePart::Rotation(Rotations::y) => "y",
            MovablePart::Rotation(Rotations::z) => "z",
            MovablePart::Wide(Faces::R) => "Rw",
            MovablePart::Wide(Faces::F) => "Fw",
            MovablePart::Wide(Faces::U) => "Uw",
            MovablePart::Wide(Faces::L) => "Lw",
            MovablePart::Wide(Faces::D) => "Dw",
            MovablePart::Wide(Faces::B) => "Bw",
            MovablePart::Slice(Slices::E) => "E",
            MovablePart::Slice(Slices::M) => "M",
            MovablePart::Slice(Slices::S) => "S",
        };
        let modif_string = match self.modifier {
            MoveModifier::Clockwise => "",
            MoveModifier::CounterClockwise => "'",
            MoveModifier::CounterDouble => "2'",
            MoveModifier::Double => "2",
        };
        write!(f, "{part_string}{modif_string}")
    }
}

impl std::ops::Mul for Move {
    type Output = Cube3By3;
    fn mul(self, rhs: Self) -> Self::Output {
        Cube3By3::from(self) * Cube3By3::from(rhs)
    }
}

impl ops::Inv for Move {
    fn inverse(&self) -> Self {
        Self {
            part: self.part,
            modifier: self.modifier.inverse(),
        }
    }
}

impl TryFrom<&str> for Move {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let wide = {
            match s.chars().nth(1) {
                Some('w') => true,
                Some(_) | None => false,
            }
        };
        let part = match s
            .chars()
            .next()
            .ok_or("only non empty strings can be turned into a move")?
        {
            'R' if !wide => MovablePart::Face(Faces::R),
            'L' if !wide => MovablePart::Face(Faces::L),
            'U' if !wide => MovablePart::Face(Faces::U),
            'D' if !wide => MovablePart::Face(Faces::D),
            'F' if !wide => MovablePart::Face(Faces::F),
            'B' if !wide => MovablePart::Face(Faces::B),
            'R' if wide => MovablePart::Wide(Faces::R),
            'L' if wide => MovablePart::Wide(Faces::L),
            'U' if wide => MovablePart::Wide(Faces::U),
            'D' if wide => MovablePart::Wide(Faces::D),
            'F' if wide => MovablePart::Wide(Faces::F),
            'B' if wide => MovablePart::Wide(Faces::B),
            'y' => MovablePart::Rotation(Rotations::y),
            'z' => MovablePart::Rotation(Rotations::z),
            'x' => MovablePart::Rotation(Rotations::x),
            'M' => MovablePart::Slice(Slices::M),
            'E' => MovablePart::Slice(Slices::E),
            'S' => MovablePart::Slice(Slices::S),
            _ => {
                return Err(format!(
                    "{s} is not a valid face, rotation, slice or wide move"
                ));
            }
        };
        let modif = {
            match s
                .get((1 + usize::from(wide))..)
                .ok_or_else(|| format!("{s} must be a valid UTF-8 &str"))?
            {
                "" => MoveModifier::Clockwise,
                "'" => MoveModifier::CounterClockwise,
                "2" => MoveModifier::Double,
                "2'" | "'2" => MoveModifier::CounterDouble,
                _ => {
                    return Err(format!(
                        "{:?} isn't a valid move modifier",
                        s.get((1 + usize::from(wide))..)
                    ));
                }
            }
        };
        Ok(Self::new(part, modif))
    }
}

impl From<Move> for Cube3By3 {
    fn from(m: Move) -> Self {
        cube_state(m.part, m.modifier)
    }
}
