mod table;

use crate::cube3by3::moves::MoveModifier::{CounterDouble, Double};
#[allow(clippy::wildcard_imports)]
use crate::{
    cube3by3::{Cube3By3, pieces::*},
    ops,
};

pub use table::ALL_MOVES;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum MovablePart {
    Face(Faces),
    Slice(Slices),
    Rotation(Rotations),
}
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum MoveModifier {
    Clockwise,
    CounterClockwise,
    Double,
    CounterDouble,
    Nothing,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Move {
    cube_representation: Cube3By3,
    part: MovablePart,
    modifier: MoveModifier,
    is_wide: bool,
    is_slice: bool,
    is_rotation: bool,
}

impl Move {
    const IDENTITY: Self = Self {
        cube_representation: Cube3By3::IDENTITY,
        is_slice: false,
        is_rotation: false,
        is_wide: false,
        part: MovablePart::Face(Faces::R),
        modifier: MoveModifier::Nothing,
    };
    const fn const_inverse(&self) -> Self {
        Self {
            cube_representation: self.cube_representation.const_inverse(),
            is_slice: self.is_slice,
            is_rotation: self.is_rotation,
            is_wide: self.is_wide,
            part: self.part,
            modifier: {
                match self.modifier {
                    MoveModifier::Clockwise => MoveModifier::CounterClockwise,
                    MoveModifier::CounterClockwise => MoveModifier::Clockwise,
                    MoveModifier::Double => MoveModifier::CounterDouble,
                    MoveModifier::CounterDouble => MoveModifier::Double,
                    MoveModifier::Nothing => MoveModifier::Nothing,
                }
            },
        }
    }
    const fn const_double(&self) -> Self {
        Self {
            cube_representation: self.cube_representation.const_mul(self.cube_representation),
            is_slice: self.is_slice,
            is_rotation: self.is_rotation,
            is_wide: self.is_wide,
            part: self.part,
            modifier: {
                match self.modifier {
                    MoveModifier::Clockwise | MoveModifier::CounterClockwise => {
                        MoveModifier::Double
                    }
                    _ => MoveModifier::Nothing,
                }
            },
        }
    }
}

impl Default for Move {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl ops::Inv for Move {
    fn inverse(&self) -> Self {
        self.const_inverse()
    }
}

impl TryFrom<&str> for Move {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let part = match s.chars().next() {
            Some('R') => MovablePart::Face(Faces::R),
            Some('L') => MovablePart::Face(Faces::L),
            Some('U') => MovablePart::Face(Faces::U),
            Some('D') => MovablePart::Face(Faces::D),
            Some('F') => MovablePart::Face(Faces::F),
            Some('B') => MovablePart::Face(Faces::B),
            Some('y') => MovablePart::Rotation(Rotations::y),
            Some('z') => MovablePart::Rotation(Rotations::z),
            Some('x') => MovablePart::Rotation(Rotations::x),
            Some('M') => MovablePart::Slice(Slices::M),
            Some('E') => MovablePart::Slice(Slices::E),
            Some('S') => MovablePart::Slice(Slices::S),
            _ => panic!("{s} is not a valid face, rotation, or slice"),
        };
        let wide = {
            match s.chars().nth(1) {
                Some('w') => true,
                Some(_) | None => false,
            }
        };
        let modif = {
            match &s[(1 + usize::from(wide))..] {
                "" => MoveModifier::Clockwise,
                "'" => MoveModifier::CounterClockwise,
                "2" => MoveModifier::Double,
                "2'" | "'2" => MoveModifier::CounterDouble,
                _ => panic!(
                    "{:?} isn't a valid move modifier",
                    &s[(1 + usize::from(wide))..]
                ),
            }
        };
        ALL_MOVES
            .iter()
            .find(|&m| m.part == part && (m.modifier == modif || (m.modifier == Double && modif == CounterDouble )) && m.is_wide == wide)
            .ok_or_else(|| format!("there is no implemented move for {s}, corresponding to {part:?}, {modif:?} and is_wide = {wide}")).copied()
    }
}

impl From<Move> for Cube3By3 {
    fn from(m: Move) -> Self {
        m.cube_representation
    }
}
