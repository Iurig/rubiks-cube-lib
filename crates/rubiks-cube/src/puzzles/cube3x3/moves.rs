mod table;

use std::str::FromStr;

#[allow(
    clippy::wildcard_imports,
    reason = "`allow`, not `expect`: the lint is skipped when the library is compiled with `cfg(test)`"
)]
use crate::{
    Indexed, Piece, ops,
    puzzles::cube3x3::{Cube3x3, pieces::*},
};
use crate::{ParseMoveError, ParseSequenceError};

use table::cube_state;

#[expect(
    unnameable_types,
    reason = "reachable through `Move3x3::part`; not exported until ticket 01 derives the parts' \
              `Display` and `TryFrom` from one list and ticket 04 settles the face role"
)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MovablePart {
    Face(Face),
    Slice(Slice),
    Rotation(Rotation),
    Wide(Face),
}

#[expect(
    unnameable_types,
    reason = "reachable through `Move3x3::modifier`; exported together with `Move3x3`"
)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MoveModifier {
    Clockwise,
    Prime,
    Double,
    DoublePrime,
}

impl MoveModifier {
    #[must_use]
    pub const fn inverse(self) -> Self {
        match self {
            Self::Clockwise => Self::Prime,
            Self::Prime => Self::Clockwise,
            Self::Double => Self::DoublePrime,
            Self::DoublePrime => Self::Double,
        }
    }
}

#[expect(
    unnameable_types,
    reason = "reachable as `<Cube3x3 as Puzzle>::Move`, and callers build moves from notation; \
              not exported until `MovablePart` is, since its fields are public"
)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Move3x3 {
    pub part: MovablePart,
    pub modifier: MoveModifier,
}

impl Move3x3 {
    #[must_use]
    pub const fn new(part: MovablePart, modifier: MoveModifier) -> Self {
        Self { part, modifier }
    }
}

impl MovablePart {
    const ALL: [Self; 2 * Face::ALL.len() + Slice::ALL.len() + Rotation::ALL.len()] = {
        let mut all =
            [Self::Face(Face::R); 2 * Face::ALL.len() + Slice::ALL.len() + Rotation::ALL.len()];
        let mut next = 0;

        let mut i = 0;
        while i < Face::ALL.len() {
            all[next] = Self::Face(Face::ALL[i]);
            next += 1;
            i += 1;
        }
        let mut i = 0;
        while i < Slice::ALL.len() {
            all[next] = Self::Slice(Slice::ALL[i]);
            next += 1;
            i += 1;
        }
        let mut i = 0;
        while i < Rotation::ALL.len() {
            all[next] = Self::Rotation(Rotation::ALL[i]);
            next += 1;
            i += 1;
        }
        let mut i = 0;
        while i < Face::ALL.len() {
            all[next] = Self::Wide(Face::ALL[i]);
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
        Self::Prime,
        Self::Double,
        Self::DoublePrime,
    ];
}

impl Indexed for MovablePart {
    const COUNT: usize = Self::ALL.len();
    fn index(self) -> usize {
        match self {
            Self::Face(f) => f as usize,
            Self::Slice(s) => Face::ALL.len() + s as usize,
            Self::Rotation(r) => Face::ALL.len() + Slice::ALL.len() + r as usize,
            Self::Wide(f) => Face::ALL.len() + Slice::ALL.len() + Rotation::ALL.len() + f as usize,
        }
    }
    fn from_index(index: usize) -> Self {
        *Self::ALL.get(index).expect("index in 0..COUNT")
    }
}

impl Indexed for MoveModifier {
    const COUNT: usize = Self::ALL.len();
    fn index(self) -> usize {
        self as usize
    }
    fn from_index(index: usize) -> Self {
        *Self::ALL.get(index).expect("index in 0..COUNT")
    }
}

impl Indexed for Move3x3 {
    const COUNT: usize = <(MovablePart, MoveModifier)>::COUNT;
    fn index(self) -> usize {
        (self.part, self.modifier).index()
    }
    fn from_index(index: usize) -> Self {
        let (part, modifier) = <(MovablePart, MoveModifier)>::from_index(index);
        Self { part, modifier }
    }
}

impl std::fmt::Display for MovablePart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Face(Face::R) => write!(f, "R"),
            Self::Face(Face::F) => write!(f, "F"),
            Self::Face(Face::U) => write!(f, "U"),
            Self::Face(Face::L) => write!(f, "L"),
            Self::Face(Face::D) => write!(f, "D"),
            Self::Face(Face::B) => write!(f, "B"),
            Self::Rotation(Rotation::X) => write!(f, "x"),
            Self::Rotation(Rotation::Y) => write!(f, "y"),
            Self::Rotation(Rotation::Z) => write!(f, "z"),
            Self::Wide(x) => write!(f, "{}w", Self::Face(*x)),
            Self::Slice(Slice::E) => write!(f, "E"),
            Self::Slice(Slice::M) => write!(f, "M"),
            Self::Slice(Slice::S) => write!(f, "S"),
        }
    }
}

impl std::fmt::Display for MoveModifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Clockwise => write!(f, ""),
            Self::Prime => write!(f, "'"),
            Self::DoublePrime => write!(f, "2'"),
            Self::Double => write!(f, "2"),
        }
    }
}

impl std::fmt::Display for Move3x3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.part, self.modifier)
    }
}

impl ops::Inv for Move3x3 {
    fn inverse(&self) -> Self {
        Self {
            part: self.part,
            modifier: self.modifier.inverse(),
        }
    }
}

impl FromStr for Move3x3 {
    type Err = ParseMoveError;
    /// One move in notation: a part, then a modifier.
    ///
    /// The part is an uppercase face letter (`R`), a face letter followed by
    /// `w` for the wide move (`Rw`), a lowercase face letter meaning the same
    /// wide move (`r`), a slice (`M E S`), or a rotation (`x y z`). Whatever
    /// follows the part must be a modifier: nothing, `'`, `2`, or `2'`.
    fn from_str(token: &str) -> Result<Self, Self::Err> {
        let mut chars = token.chars();
        let first = chars.next().ok_or(ParseMoveError::EmptyString)?;
        let after_first = chars.as_str();
        let face = |letter: char| match letter {
            'R' => Some(Face::R),
            'L' => Some(Face::L),
            'U' => Some(Face::U),
            'D' => Some(Face::D),
            'F' => Some(Face::F),
            'B' => Some(Face::B),
            _ => None,
        };
        let (part, modifier_text) = match first {
            'x' => (MovablePart::Rotation(Rotation::X), after_first),
            'y' => (MovablePart::Rotation(Rotation::Y), after_first),
            'z' => (MovablePart::Rotation(Rotation::Z), after_first),
            'M' => (MovablePart::Slice(Slice::M), after_first),
            'E' => (MovablePart::Slice(Slice::E), after_first),
            'S' => (MovablePart::Slice(Slice::S), after_first),
            _ => match (face(first), face(first.to_ascii_uppercase())) {
                (Some(f), _) => after_first
                    .strip_prefix('w')
                    .map_or((MovablePart::Face(f), after_first), |after_w| {
                        (MovablePart::Wide(f), after_w)
                    }),
                (None, Some(f)) => (MovablePart::Wide(f), after_first),
                (None, None) => {
                    return Err(ParseMoveError::BadPart {
                        invalid_move: token.to_string(),
                        part: first.to_string(),
                    });
                }
            },
        };
        let modifier = match modifier_text {
            "" => MoveModifier::Clockwise,
            "'" => MoveModifier::Prime,
            "2" => MoveModifier::Double,
            "2'" | "'2" => MoveModifier::DoublePrime,
            other => {
                return Err(ParseMoveError::BadModifier {
                    invalid_move: token.to_string(),
                    modifier: other.to_string(),
                });
            }
        };
        Ok(Self::new(part, modifier))
    }
}

impl Move3x3 {
    /// Every move in a move sequence, in order.
    ///
    /// `//` starts a comment that runs to the end of the line, whitespace
    /// separates moves, and each token is parsed with [`TryFrom<&str>`]. A
    /// sequence with no moves in it, such as an empty string or a comment on
    /// its own, yields nothing.
    pub fn sequence(text: &str) -> impl Iterator<Item = Result<Self, ParseSequenceError>> {
        text.lines().enumerate().flat_map(|(line_number, line)| {
            line.split_once("//")
                .map_or(line, |(moves, _)| moves)
                .split_whitespace()
                .enumerate()
                .map(move |(move_number, m)| {
                    Self::from_str(m).map_err(|e| ParseSequenceError {
                        source: e,
                        line: line_number + 1,
                        position: move_number + 1,
                    })
                })
        })
    }
}

impl From<Move3x3> for Cube3x3 {
    fn from(m: Move3x3) -> Self {
        cube_state(m.part, m.modifier)
    }
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "tests should panic if failed, and return result for `?` convenience"
)]
mod tests {
    //! `Move3x3::ALL` lists every move independently of `Indexed`, so tests can check one
    //! against the other.
    use super::*;
    use crate::indexed::assert_round_trips;
    use std::error::Error;

    impl Move3x3 {
        const ALL: [Self; MovablePart::ALL.len() * MoveModifier::ALL.len()] = {
            let mut all: [Self; MovablePart::ALL.len() * MoveModifier::ALL.len()] = [Self {
                part: MovablePart::Face(Face::R),
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
    fn every_move_has_its_own_index() {
        assert_round_trips::<MovablePart>();
        assert_round_trips::<MoveModifier>();
        assert_round_trips::<Move3x3>();
        assert_eq!(Move3x3::all().collect::<Vec<_>>(), Move3x3::ALL);
    }

    #[test]
    fn move_display_round_trip() {
        for m in Move3x3::ALL {
            assert_eq!(m, Move3x3::from_str(m.to_string().as_str()).unwrap());
        }
    }

    fn moves_of(text: &str) -> Result<Vec<Move3x3>, ParseSequenceError> {
        Move3x3::sequence(text).collect()
    }

    #[test]
    fn comments_run_to_end_of_line() -> Result<(), Box<dyn Error>> {
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
    #[expect(
        clippy::assert_is_empty,
        reason = "is_empty reads better because of type conversion"
    )]
    fn empty_and_comment_only_sequences_have_no_moves() -> Result<(), Box<dyn Error>> {
        assert!(moves_of("")?.is_empty());
        assert!(moves_of("// just a comment")?.is_empty());
        assert!(moves_of("  \n\t// one\n// two\n")?.is_empty());
        Ok(())
    }

    #[test]
    fn lowercase_face_is_the_wide_move() {
        for face in Face::ALL {
            let wide = MovablePart::Wide(face).to_string();
            let lower = MovablePart::Face(face).to_string().to_lowercase();
            for modifier in ["", "'", "2", "2'"] {
                assert_eq!(
                    Move3x3::from_str(format!("{lower}{modifier}").as_str()),
                    Move3x3::from_str(format!("{wide}{modifier}").as_str()),
                );
            }
        }
    }

    #[test]
    fn a_bad_move_is_the_error_variant_that_carries_it() {
        for (bad, expected_err) in [
            (
                "rw",
                ParseMoveError::BadModifier {
                    invalid_move: "rw".to_string(),
                    modifier: "w".to_string(),
                },
            ),
            (
                "Rww",
                ParseMoveError::BadModifier {
                    invalid_move: "Rww".to_string(),
                    modifier: "w".to_string(),
                },
            ),
            (
                "Q",
                ParseMoveError::BadPart {
                    invalid_move: "Q".to_string(),
                    part: "Q".to_string(),
                },
            ),
            (
                "R3",
                ParseMoveError::BadModifier {
                    invalid_move: "R3".to_string(),
                    modifier: "3".to_string(),
                },
            ),
            (
                "w",
                ParseMoveError::BadPart {
                    invalid_move: "w".to_string(),
                    part: "w".to_string(),
                },
            ),
            (
                "xw",
                ParseMoveError::BadModifier {
                    invalid_move: "xw".to_string(),
                    modifier: "w".to_string(),
                },
            ),
            (
                "Mw",
                ParseMoveError::BadModifier {
                    invalid_move: "Mw".to_string(),
                    modifier: "w".to_string(),
                },
            ),
            (
                "R''",
                ParseMoveError::BadModifier {
                    invalid_move: "R''".to_string(),
                    modifier: "''".to_string(),
                },
            ),
        ] {
            assert_eq!(Move3x3::from_str(bad), Err(expected_err));
        }
        assert_eq!(
            moves_of("R Q U"),
            Err(ParseSequenceError {
                source: ParseMoveError::BadPart {
                    invalid_move: "Q".to_string(),
                    part: "Q".to_string()
                },
                line: 1,
                position: 2
            })
        );
    }
}
