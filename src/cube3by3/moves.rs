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
                Self::Wide(x) => return write!(f, "{}w", Self::Face(*x)),
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
    /// One token of move notation: a part, then a modifier.
    ///
    /// The part is an uppercase face letter (`R`), a face letter followed by
    /// `w` for the wide move (`Rw`), a lowercase face letter meaning the same
    /// wide move (`r`), a slice (`M E S`), or a rotation (`x y z`). Whatever
    /// follows the part must be a modifier: nothing, `'`, `2`, or `2'`.
    fn try_from(token: &str) -> Result<Self, Self::Error> {
        let mut chars = token.chars();
        let first = chars
            .next()
            .ok_or("only non empty strings can be turned into a move")?;
        let after_first = chars.as_str();
        let face = |letter: char| match letter {
            'R' => Some(Faces::R),
            'L' => Some(Faces::L),
            'U' => Some(Faces::U),
            'D' => Some(Faces::D),
            'F' => Some(Faces::F),
            'B' => Some(Faces::B),
            _ => None,
        };
        let (part, modifier_text) = match first {
            'x' => (MovablePart::Rotation(Rotations::x), after_first),
            'y' => (MovablePart::Rotation(Rotations::y), after_first),
            'z' => (MovablePart::Rotation(Rotations::z), after_first),
            'M' => (MovablePart::Slice(Slices::M), after_first),
            'E' => (MovablePart::Slice(Slices::E), after_first),
            'S' => (MovablePart::Slice(Slices::S), after_first),
            _ => match (face(first), face(first.to_ascii_uppercase())) {
                (Some(f), _) => after_first
                    .strip_prefix('w')
                    .map_or((MovablePart::Face(f), after_first), |after_w| {
                        (MovablePart::Wide(f), after_w)
                    }),
                (None, Some(f)) => (MovablePart::Wide(f), after_first),
                (None, None) => {
                    return Err(format!(
                        "{token} is not a valid face, rotation, slice or wide move"
                    ));
                }
            },
        };
        let modifier = match modifier_text {
            "" => MoveModifier::Clockwise,
            "'" => MoveModifier::CounterClockwise,
            "2" => MoveModifier::Double,
            "2'" | "'2" => MoveModifier::CounterDouble,
            other => {
                return Err(format!("{other:?} in {token} isn't a valid move modifier"));
            }
        };
        Ok(Self::new(part, modifier))
    }
}

impl Move {
    /// Every move in a move sequence, in order.
    ///
    /// `//` starts a comment that runs to the end of the line, whitespace
    /// separates moves, and each token is parsed with [`TryFrom<&str>`]. A
    /// sequence with no moves in it, such as an empty string or a comment on
    /// its own, yields nothing.
    pub fn sequence(text: &str) -> impl Iterator<Item = Result<Self, String>> {
        text.lines()
            .flat_map(|line| {
                line.split_once("//")
                    .map_or(line, |(moves, _)| moves)
                    .split_whitespace()
            })
            .map(Self::try_from)
    }
}

impl From<Move> for Cube3By3 {
    fn from(m: Move) -> Self {
        cube_state(m.part, m.modifier)
    }
}

#[cfg(test)]
#[allow(clippy::panic_in_result_fn)]
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

    fn moves_of(text: &str) -> Result<Vec<Move>, String> {
        Move::sequence(text).collect()
    }

    #[test]
    fn comments_run_to_end_of_line() -> Result<(), String> {
        assert_eq!(moves_of("R U //this is a comment")?, moves_of("R U")?);
        assert_eq!(
            moves_of(
                "y2 F' M F' R U' R U' Fw z' // FB
                 U R U r M' U' R U2' R' // SS
                 U R' U' R U' R' U' r // SP (CMLL skip)
                 U M' U' M U' U' M' U M // EOLR
                 U' U' M2' U' M U' U' M' U' U' M2' // EP"
            )?,
            moves_of(
                "y2 F' M F' R U' R U' Fw z'
                 U R U r M' U' R U2' R'
                 U R' U' R U' R' U' r
                 U M' U' M U' U' M' U M
                 U' U' M2' U' M U' U' M' U' U' M2'"
            )?
        );
        Ok(())
    }

    #[test]
    fn empty_and_comment_only_sequences_have_no_moves() -> Result<(), String> {
        assert!(moves_of("")?.is_empty());
        assert!(moves_of("// just a comment")?.is_empty());
        assert!(moves_of("  \n\t// one\n// two\n")?.is_empty());
        Ok(())
    }

    #[test]
    fn lowercase_face_is_the_wide_move() -> Result<(), String> {
        for face in Faces::ALL {
            let wide = MovablePart::Wide(face).to_string();
            let lower = MovablePart::Face(face).to_string().to_lowercase();
            for modifier in ["", "'", "2", "2'"] {
                assert_eq!(
                    Move::try_from(format!("{lower}{modifier}").as_str())?,
                    Move::try_from(format!("{wide}{modifier}").as_str())?,
                    "{lower}{modifier} should be {wide}{modifier}"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn a_bad_token_is_an_error_that_names_it() {
        for bad in ["rw", "Rww", "Q", "R3", "w", "xw", "Mw", "R''"] {
            let err = Move::try_from(bad).unwrap_err();
            assert!(err.contains(bad), "{err:?} should mention {bad:?}");
        }
        assert!(moves_of("R Q U").is_err());
    }
}
