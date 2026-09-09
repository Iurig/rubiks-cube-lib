//! The table of every implemented move, built at compile time.
//!
//! The nine face and slice moves are written out by hand; the three rotations
//! are derived from them, and every move's inverse and double are then generated.

use crate::Piece;
#[allow(clippy::wildcard_imports)]
use crate::{
    cube3by3::{Cube3By3, pieces::*},
    zn::ZnRing,
};

#[allow(clippy::enum_glob_use)]
use super::{MovablePart, MovablePart::*, MoveModifier, MoveModifier::*};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct MoveInformation {
    cube_state: Cube3By3,
    part: MovablePart,
    modifier: MoveModifier,
}
/// Position of this part in the clockwise move list.
const fn table_index_clockwise(part: MovablePart) -> usize {
    match part {
        Face(m) => m as usize,
        Slice(m) => Faces::ALL.len() + m as usize,
        Rotation(m) => Faces::ALL.len() + Slices::ALL.len() + m as usize,
        Wide(x) => {
            table_index_clockwise(Face(x))
                + Faces::ALL.len()
                + Slices::ALL.len()
                + Rotations::ALL.len()
        }
    }
}

const _: () = const {
    let mut i = 0;
    while i < ALL_MOVES.len() {
        assert!(i == table_index(ALL_MOVES[i].part, ALL_MOVES[i].modifier));
        i += 1;
    }
};

const fn table_index(part: MovablePart, modifier: MoveModifier) -> usize {
    3 * table_index_clockwise(part)
        + match modifier {
            Clockwise => 0,
            CounterClockwise => 1,
            CounterDouble | Double => 2,
        }
}
pub const fn cube_state(part: MovablePart, modifier: MoveModifier) -> Cube3By3 {
    ALL_MOVES[table_index(part, modifier)].cube_state
}

impl MoveInformation {
    const IDENTITY: Self = Self {
        cube_state: Cube3By3::IDENTITY,
        part: Face(Faces::R),
        modifier: Clockwise,
    };

    const fn const_inverse(&self) -> Self {
        Self {
            cube_state: self.cube_state.const_inverse(),

            part: self.part,
            modifier: self.modifier.inverse(),
        }
    }

    #[allow(clippy::panic)] // only used privately at compile time
    const fn const_double(&self) -> Self {
        Self {
            cube_state: self.cube_state.const_mul(self.cube_state),
            part: self.part,
            modifier: {
                match self.modifier {
                    Clockwise | CounterClockwise => Double,
                    _ => panic!(),
                }
            },
        }
    }
}

impl Slices {
    const fn follows(self) -> Faces {
        match self {
            Self::M => Faces::L,
            Self::E => Faces::D,
            Self::S => Faces::F,
        }
    }
}
impl Rotations {
    const fn follows(self) -> Faces {
        match self {
            Self::x => Faces::R,
            Self::y => Faces::U,
            Self::z => Faces::F,
        }
    }
}
impl Faces {
    const fn opposite(self) -> Self {
        match self {
            Self::R => Self::L,
            Self::L => Self::R,
            Self::U => Self::D,
            Self::D => Self::U,
            Self::F => Self::B,
            Self::B => Self::F,
        }
    }
}

const FACE_AND_SLICES_CLOCKWISE_MOVE_COUNT: usize = Faces::ALL.len() + Slices::ALL.len();
const ALL_FACE_AND_SLICES_CLOCKWISE_MOVES: [MoveInformation; FACE_AND_SLICES_CLOCKWISE_MOVE_COUNT] = [
    MoveInformation {
        cube_state: Cube3By3 {
            center_configuration: Centers::IDENTITY,
            corner_configuration: {
                let mut corners = Corners::cycle([[
                    SingleCorner::Ufr,
                    SingleCorner::Ubr,
                    SingleCorner::Dbr,
                    SingleCorner::Dfr,
                ]]);
                corners.orientation = ZnRing::array([0, 1, 2, 0, 0, 1, 2, 0]);
                corners
            },
            edge_configuration: Edges::cycle([[
                SingleEdge::Fr,
                SingleEdge::Ur,
                SingleEdge::Br,
                SingleEdge::Dr,
            ]]),
        },
        part: Face(Faces::R),
        modifier: Clockwise,
    },
    MoveInformation {
        cube_state: Cube3By3 {
            center_configuration: Centers::IDENTITY,
            corner_configuration: {
                let mut corners = Corners::cycle([[
                    SingleCorner::Ubl,
                    SingleCorner::Ufl,
                    SingleCorner::Dfl,
                    SingleCorner::Dbl,
                ]]);
                corners.orientation = ZnRing::array([2, 0, 0, 1, 2, 0, 0, 1]);
                corners
            },
            edge_configuration: Edges::cycle([[
                SingleEdge::Fl,
                SingleEdge::Dl,
                SingleEdge::Bl,
                SingleEdge::Ul,
            ]]),
        },
        part: Face(Faces::L),
        modifier: Clockwise,
    },
    MoveInformation {
        cube_state: Cube3By3 {
            center_configuration: Centers::IDENTITY,
            corner_configuration: {
                let mut corners = Corners::cycle([[
                    SingleCorner::Ufr,
                    SingleCorner::Ufl,
                    SingleCorner::Ubl,
                    SingleCorner::Ubr,
                ]]);
                corners.orientation = ZnRing::array([0, 0, 0, 0, 0, 0, 0, 0]);
                corners
            },
            edge_configuration: Edges::cycle([[
                SingleEdge::Uf,
                SingleEdge::Ul,
                SingleEdge::Ub,
                SingleEdge::Ur,
            ]]),
        },
        part: Face(Faces::U),
        modifier: Clockwise,
    },
    MoveInformation {
        cube_state: Cube3By3 {
            center_configuration: Centers::IDENTITY,
            corner_configuration: {
                let mut corners = Corners::cycle([[
                    SingleCorner::Dfr,
                    SingleCorner::Dbr,
                    SingleCorner::Dbl,
                    SingleCorner::Dfl,
                ]]);
                corners.orientation = ZnRing::array([0, 0, 0, 0, 0, 0, 0, 0]);
                corners
            },
            edge_configuration: Edges::cycle([[
                SingleEdge::Df,
                SingleEdge::Dr,
                SingleEdge::Db,
                SingleEdge::Dl,
            ]]),
        },
        part: Face(Faces::D),
        modifier: Clockwise,
    },
    MoveInformation {
        cube_state: Cube3By3 {
            center_configuration: Centers::IDENTITY,
            corner_configuration: {
                let mut corners = Corners::cycle([[
                    SingleCorner::Dfr,
                    SingleCorner::Dfl,
                    SingleCorner::Ufl,
                    SingleCorner::Ufr,
                ]]);
                corners.orientation = ZnRing::array([0, 0, 1, 2, 1, 2, 0, 0]);
                corners
            },
            edge_configuration: Edges {
                permutation: Edges::cycle([[
                    SingleEdge::Df,
                    SingleEdge::Fl,
                    SingleEdge::Uf,
                    SingleEdge::Fr,
                ]])
                .permutation,
                orientation: ZnRing::array([0, 0, 1, 0, 1, 1, 0, 0, 1, 0, 0, 0]),
            },
        },
        part: Face(Faces::F),
        modifier: Clockwise,
    },
    MoveInformation {
        cube_state: Cube3By3 {
            center_configuration: Centers::IDENTITY,
            corner_configuration: {
                let mut corners = Corners::cycle([[
                    SingleCorner::Ubr,
                    SingleCorner::Ubl,
                    SingleCorner::Dbl,
                    SingleCorner::Dbr,
                ]]);
                corners.orientation = ZnRing::array([1, 2, 0, 0, 0, 0, 1, 2]);
                corners
            },
            edge_configuration: Edges {
                permutation: Edges::cycle([[
                    SingleEdge::Br,
                    SingleEdge::Ub,
                    SingleEdge::Bl,
                    SingleEdge::Db,
                ]])
                .permutation,
                orientation: ZnRing::array([1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0]),
            },
        },
        part: Face(Faces::B),
        modifier: Clockwise,
    },
    MoveInformation {
        cube_state: Cube3By3 {
            center_configuration: Centers::cycle([[
                SingleCenter::F,
                SingleCenter::R,
                SingleCenter::B,
                SingleCenter::L,
            ]]),
            corner_configuration: Corners::IDENTITY,
            edge_configuration: Edges {
                permutation: Edges::cycle([[
                    SingleEdge::Fr,
                    SingleEdge::Br,
                    SingleEdge::Bl,
                    SingleEdge::Fl,
                ]])
                .permutation,
                orientation: ZnRing::array([0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0]),
            },
        },
        part: Slice(Slices::E),
        modifier: Clockwise,
    },
    MoveInformation {
        cube_state: Cube3By3 {
            center_configuration: Centers::cycle([[
                SingleCenter::F,
                SingleCenter::D,
                SingleCenter::B,
                SingleCenter::U,
            ]]),
            corner_configuration: Corners::IDENTITY,
            edge_configuration: Edges {
                permutation: Edges::cycle([[
                    SingleEdge::Uf,
                    SingleEdge::Df,
                    SingleEdge::Db,
                    SingleEdge::Ub,
                ]])
                .permutation,
                orientation: ZnRing::array([1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0]),
            },
        },
        part: Slice(Slices::M),
        modifier: Clockwise,
    },
    MoveInformation {
        cube_state: Cube3By3 {
            center_configuration: Centers::cycle([[
                SingleCenter::U,
                SingleCenter::R,
                SingleCenter::D,
                SingleCenter::L,
            ]]),
            corner_configuration: Corners::IDENTITY,
            edge_configuration: Edges {
                permutation: Edges::cycle([[
                    SingleEdge::Ul,
                    SingleEdge::Ur,
                    SingleEdge::Dr,
                    SingleEdge::Dl,
                ]])
                .permutation,
                orientation: ZnRing::array([0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1]),
            },
        },
        part: Slice(Slices::S),
        modifier: Clockwise,
    },
];

#[allow(clippy::panic)] // only run at compile time
const fn slice_along(face: Faces, placed: &[MoveInformation; CLOCKWISE_MOVE_COUNT]) -> Cube3By3 {
    let mut i = 0;
    while i < Slices::ALL.len() {
        if Slices::ALL[i].follows() as u8 == face as u8 {
            return placed[table_index_clockwise(Slice(Slices::ALL[i]))].cube_state;
        } else if Slices::ALL[i].follows().opposite() as u8 == face as u8 {
            return placed[table_index_clockwise(Slice(Slices::ALL[i]))]
                .cube_state
                .const_inverse();
        }
        i += 1;
    }
    panic!("all faces must have a slice_along")
}

const CLOCKWISE_MOVE_COUNT: usize =
    FACE_AND_SLICES_CLOCKWISE_MOVE_COUNT + Faces::ALL.len() + Rotations::ALL.len();
const ALL_CLOCKWISE_MOVES: [MoveInformation; CLOCKWISE_MOVE_COUNT] = {
    let mut all_clockwise_moves = [MoveInformation::IDENTITY; CLOCKWISE_MOVE_COUNT];
    let mut i = 0;
    while i < FACE_AND_SLICES_CLOCKWISE_MOVE_COUNT {
        all_clockwise_moves[table_index_clockwise(ALL_FACE_AND_SLICES_CLOCKWISE_MOVES[i].part)] =
            ALL_FACE_AND_SLICES_CLOCKWISE_MOVES[i];
        i += 1;
    }

    let mut i = 0;
    while i < Rotations::ALL.len() {
        all_clockwise_moves[table_index_clockwise(Rotation(Rotations::ALL[i]))] = MoveInformation {
            cube_state: all_clockwise_moves
                [table_index_clockwise(Face(Rotations::ALL[i].follows()))]
            .cube_state
            .const_mul(
                all_clockwise_moves
                    [table_index_clockwise(Face(Rotations::ALL[i].follows().opposite()))]
                .cube_state
                .const_inverse(),
            )
            .const_mul(slice_along(
                Rotations::ALL[i].follows(),
                &all_clockwise_moves,
            )),
            part: Rotation(Rotations::ALL[i]),
            modifier: Clockwise,
        };
        i += 1;
    }
    let mut i = 0;
    while i < Faces::ALL.len() {
        let face = Faces::ALL[i];
        all_clockwise_moves[table_index_clockwise(Wide(face))] = MoveInformation {
            cube_state: all_clockwise_moves[table_index_clockwise(Face(face))]
                .cube_state
                .const_mul(slice_along(face, &all_clockwise_moves)),
            part: Wide(face),
            modifier: Clockwise,
        };
        i += 1;
    }

    all_clockwise_moves
};

const ALL_MOVES: [MoveInformation; 3 * CLOCKWISE_MOVE_COUNT] = {
    let mut all_moves = [MoveInformation::IDENTITY; 3 * CLOCKWISE_MOVE_COUNT];
    let mut i = 0;
    while i < CLOCKWISE_MOVE_COUNT {
        all_moves[3 * i] = ALL_CLOCKWISE_MOVES[i];
        all_moves[3 * i + 1] = ALL_CLOCKWISE_MOVES[i].const_inverse();
        all_moves[3 * i + 2] = ALL_CLOCKWISE_MOVES[i].const_double();
        i += 1;
    }
    all_moves
};

#[cfg(test)]
mod tests {
    //! Handedness pins: one fact from the physical cube per base move, stated as
    //! "after the move, the piece now at slot X came from Y". These are the only
    //! tests that do not depend on the derivation, so they are the ones that
    //! catch a mirrored base move.
    use super::*;
    use crate::index;

    fn corner_at(cube: Cube3By3, slot: SingleCorner) -> SingleCorner {
        cube.corner_configuration.permutation[index(slot)]
    }
    fn center_at(cube: Cube3By3, slot: SingleCenter) -> SingleCenter {
        cube.center_configuration.permutation[index(slot)]
    }
    fn clockwise(part: MovablePart) -> Cube3By3 {
        cube_state(part, Clockwise)
    }

    // Faces: clockwise when looking at that face.

    #[test]
    fn r_takes_front_top_corner_to_back_top() {
        assert_eq!(
            corner_at(clockwise(Face(Faces::R)), SingleCorner::Ubr),
            SingleCorner::Ufr
        );
    }
    #[test]
    fn l_takes_back_top_corner_to_front_top() {
        assert_eq!(
            corner_at(clockwise(Face(Faces::L)), SingleCorner::Ufl),
            SingleCorner::Ubl
        );
    }
    #[test]
    fn u_takes_front_right_corner_to_front_left() {
        assert_eq!(
            corner_at(clockwise(Face(Faces::U)), SingleCorner::Ufl),
            SingleCorner::Ufr
        );
    }
    #[test]
    fn d_takes_front_right_corner_to_back_right() {
        assert_eq!(
            corner_at(clockwise(Face(Faces::D)), SingleCorner::Dbr),
            SingleCorner::Dfr
        );
    }
    #[test]
    fn f_takes_top_right_corner_to_bottom_right() {
        assert_eq!(
            corner_at(clockwise(Face(Faces::F)), SingleCorner::Dfr),
            SingleCorner::Ufr
        );
    }
    #[test]
    fn b_takes_top_right_corner_to_top_left() {
        assert_eq!(
            corner_at(clockwise(Face(Faces::B)), SingleCorner::Ubl),
            SingleCorner::Ubr
        );
    }

    // Slices: each follows its reference face, so it moves centers the way that face moves corners.

    #[test]
    fn e_follows_d_and_takes_front_center_to_right() {
        assert_eq!(
            center_at(clockwise(Slice(Slices::E)), SingleCenter::R),
            SingleCenter::F
        );
    }
    #[test]
    fn m_follows_l_and_takes_top_center_to_front() {
        assert_eq!(
            center_at(clockwise(Slice(Slices::M)), SingleCenter::F),
            SingleCenter::U
        );
    }
    #[test]
    fn s_follows_f_and_takes_top_center_to_right() {
        assert_eq!(
            center_at(clockwise(Slice(Slices::S)), SingleCenter::R),
            SingleCenter::U
        );
    }

    // Derivation identities: these pin the derivation, not the base moves.
    // A mirrored slice mirrors its rotation with it and still passes here.

    fn seq(parts: &[(MovablePart, MoveModifier)]) -> Cube3By3 {
        parts.iter().fold(Cube3By3::IDENTITY, |cube, &(p, m)| {
            cube.const_mul(cube_state(p, m))
        })
    }

    #[test]
    fn rotations_are_face_then_slice_then_opposite_face_undone() {
        assert_eq!(
            clockwise(Rotation(Rotations::x)),
            seq(&[
                (Face(Faces::R), Clockwise),
                (Slice(Slices::M), CounterClockwise),
                (Face(Faces::L), CounterClockwise),
            ])
        );
        assert_eq!(
            clockwise(Rotation(Rotations::y)),
            seq(&[
                (Face(Faces::U), Clockwise),
                (Slice(Slices::E), CounterClockwise),
                (Face(Faces::D), CounterClockwise),
            ])
        );
        assert_eq!(
            clockwise(Rotation(Rotations::z)),
            seq(&[
                (Face(Faces::F), Clockwise),
                (Slice(Slices::S), Clockwise),
                (Face(Faces::B), CounterClockwise),
            ])
        );
    }

    #[test]
    fn wide_moves_equal_opposite_face_plus_rotation() {
        // Rw = L x, Uw = D y, Fw = B z: reaches each wide move by a different
        // route than the table's own derivation (face then parallel slice).
        for (wide, opposite, rotation) in [
            (Faces::R, Faces::L, Rotations::x),
            (Faces::U, Faces::D, Rotations::y),
            (Faces::F, Faces::B, Rotations::z),
        ] {
            assert_eq!(
                clockwise(Wide(wide)),
                seq(&[(Face(opposite), Clockwise), (Rotation(rotation), Clockwise)]),
                "{wide:?}w should equal {opposite:?} {rotation:?}"
            );
        }
    }

    // Modifiers: every clockwise entry generates its own inverse and double.

    #[test]
    fn every_part_has_consistent_modifiers() {
        for entry in ALL_CLOCKWISE_MOVES {
            let part = entry.part;
            let cw = cube_state(part, Clockwise);
            let ccw = cube_state(part, CounterClockwise);
            assert_eq!(
                cw.const_mul(ccw),
                Cube3By3::IDENTITY,
                "{part:?}' must undo {part:?}"
            );
            assert_eq!(
                cube_state(part, Double),
                cw.const_mul(cw),
                "{part:?}2 must be {part:?} twice"
            );
            assert_eq!(
                cube_state(part, Double),
                cube_state(part, CounterDouble),
                "{part:?}2 and {part:?}2' must share a cube state"
            );
        }
    }
}
