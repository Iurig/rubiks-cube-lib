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

impl std::fmt::Display for MovablePart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Face(Faces::R) => String::from("R"),
                Self::Face(Faces::F) => String::from("F"),
                Self::Face(Faces::U) => String::from("U"),
                Self::Face(Faces::L) => String::from("L"),
                Self::Face(Faces::D) => String::from("D"),
                Self::Face(Faces::B) => String::from("B"),
                Self::Rotation(Rotations::x) => String::from("x"),
                Self::Rotation(Rotations::y) => String::from("y"),
                Self::Rotation(Rotations::z) => String::from("z"),
                Self::Wide(x) => Self::Face(*x).to_string() + "w",
                Self::Slice(Slices::E) => String::from("E"),
                Self::Slice(Slices::M) => String::from("M"),
                Self::Slice(Slices::S) => String::from("S"),
            }
        )
    }
}

impl std::fmt::Display for MoveModifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Clockwise => "",
                Self::CounterClockwise => "'",
                Self::CounterDouble => "2'",
                Self::Double => "2",
            }
        )
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.part, self.modifier)
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

#[cfg(test)]
mod tests {
    //! Enumerations of every part, modifier, and move. Only tests need them
    //! today; move them out of this module when a production caller appears.
    use super::*;
    use crate::Piece;

    impl MovablePart {
        const ALL: [Self; 2 * Faces::ALL.len() + Slices::ALL.len() + Rotations::ALL.len()] = {
            let mut all = [Self::Face(Faces::R);
                2 * Faces::ALL.len() + Slices::ALL.len() + Rotations::ALL.len()];
            let mut next = 0;

            let mut i = 0;
            while i < Faces::ALL.len() {
                all[next] = Self::Face(Faces::ALL[i]);
                next += 1;
                i += 1;
            }
            let mut i = 0;
            while i < Slices::ALL.len() {
                all[next] = Self::Slice(Slices::ALL[i]);
                next += 1;
                i += 1;
            }
            let mut i = 0;
            while i < Rotations::ALL.len() {
                all[next] = Self::Rotation(Rotations::ALL[i]);
                next += 1;
                i += 1;
            }
            let mut i = 0;
            while i < Faces::ALL.len() {
                all[next] = Self::Wide(Faces::ALL[i]);
                next += 1;
                i += 1;
            }
            assert!(next == all.len());
            all
        };
    }

    impl MoveModifier {
        const ALL: [Self; 4] = [
            Self::Clockwise,
            Self::CounterClockwise,
            Self::Double,
            Self::CounterDouble,
        ];
    }

    impl Move {
        const ALL: [Self; MovablePart::ALL.len() * MoveModifier::ALL.len()] = {
            let mut all: [Self; MovablePart::ALL.len() * MoveModifier::ALL.len()] = [Self {
                part: MovablePart::Face(Faces::R),
                modifier: MoveModifier::Clockwise,
            };
                MovablePart::ALL.len() * MoveModifier::ALL.len()];
            let mut i = 0;
            while i < MovablePart::ALL.len() {
                let mut j = 0;
                while j < MoveModifier::ALL.len() {
                    all[i * MoveModifier::ALL.len() + j] = Self {
                        part: MovablePart::ALL[i],
                        modifier: MoveModifier::ALL[j],
                    };
                    j += 1;
                }
                i += 1;
            }
            all
        };
    }

    #[test]
    fn move_display_round_trip() {
        for m in Move::ALL {
            assert_eq!(m, Move::try_from(m.to_string().as_str()).unwrap());
        }
    }
}
