//! The table of every implemented move, built at compile time.
//!
//! The nine face and slice moves are written out by hand; the three rotations
//! are derived from them, and every move's inverse and double are then generated.

use std::sync::LazyLock;

use crate::Inv;

// `allow` instead of `expect` because the lint is skipped once the
// library is compiled with `cfg(test)`
#[allow(clippy::wildcard_imports, clippy::enum_glob_use)]
use {
    super::{MovablePart, MovablePart::*, MoveModifier, MoveModifier::*},
    crate::{
        puzzles::cube3by3::{Cube3By3, pieces::*},
        zn::Zn,
    },
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
struct MoveInformation {
    cube_state: Cube3By3,
    part: MovablePart,
    modifier: MoveModifier,
}
/// Position of this part in the clockwise move list.
fn table_index_clockwise(part: MovablePart) -> usize {
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

fn table_index(part: MovablePart, modifier: MoveModifier) -> usize {
    3 * table_index_clockwise(part)
        + match modifier {
            Clockwise => 0,
            CounterClockwise => 1,
            CounterDouble | Double => 2,
        }
}
pub fn cube_state(part: MovablePart, modifier: MoveModifier) -> Cube3By3 {
    ALL_MOVES[table_index(part, modifier)].cube_state
}

impl Inv for MoveInformation {
    fn inverse(&self) -> Self {
        Self {
            cube_state: self.cube_state.inverse(),
            part: self.part,
            modifier: self.modifier.inverse(),
        }
    }
}

impl MoveInformation {
    #[expect(clippy::panic, reason = "only used privately at compile time")]
    fn double(&self) -> Self {
        Self {
            cube_state: self.cube_state * self.cube_state,
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
static ALL_FACE_AND_SLICES_CLOCKWISE_MOVES: LazyLock<
    [MoveInformation; FACE_AND_SLICES_CLOCKWISE_MOVE_COUNT],
> = LazyLock::new(|| {
    [
        MoveInformation {
            cube_state: Cube3By3 {
                center_configuration: CenterConfiguration::identity(),
                corner_configuration: {
                    let mut corners = CornerConfiguration::cycle([[
                        Corner::Ufr,
                        Corner::Ubr,
                        Corner::Dbr,
                        Corner::Dfr,
                    ]]);
                    corners.orientation = Zn::array([0, 1, 2, 0, 0, 1, 2, 0]);
                    corners
                },
                edge_configuration: EdgeConfiguration::cycle([[
                    Edge::Fr,
                    Edge::Ur,
                    Edge::Br,
                    Edge::Dr,
                ]]),
            },
            part: Face(Faces::R),
            modifier: Clockwise,
        },
        MoveInformation {
            cube_state: Cube3By3 {
                center_configuration: CenterConfiguration::identity(),
                corner_configuration: {
                    let mut corners = CornerConfiguration::cycle([[
                        Corner::Ubl,
                        Corner::Ufl,
                        Corner::Dfl,
                        Corner::Dbl,
                    ]]);
                    corners.orientation = Zn::array([2, 0, 0, 1, 2, 0, 0, 1]);
                    corners
                },
                edge_configuration: EdgeConfiguration::cycle([[
                    Edge::Fl,
                    Edge::Dl,
                    Edge::Bl,
                    Edge::Ul,
                ]]),
            },
            part: Face(Faces::L),
            modifier: Clockwise,
        },
        MoveInformation {
            cube_state: Cube3By3 {
                center_configuration: CenterConfiguration::identity(),
                corner_configuration: {
                    let mut corners = CornerConfiguration::cycle([[
                        Corner::Ufr,
                        Corner::Ufl,
                        Corner::Ubl,
                        Corner::Ubr,
                    ]]);
                    corners.orientation = Zn::array([0, 0, 0, 0, 0, 0, 0, 0]);
                    corners
                },
                edge_configuration: EdgeConfiguration::cycle([[
                    Edge::Uf,
                    Edge::Ul,
                    Edge::Ub,
                    Edge::Ur,
                ]]),
            },
            part: Face(Faces::U),
            modifier: Clockwise,
        },
        MoveInformation {
            cube_state: Cube3By3 {
                center_configuration: CenterConfiguration::identity(),
                corner_configuration: {
                    let mut corners = CornerConfiguration::cycle([[
                        Corner::Dfr,
                        Corner::Dbr,
                        Corner::Dbl,
                        Corner::Dfl,
                    ]]);
                    corners.orientation = Zn::array([0, 0, 0, 0, 0, 0, 0, 0]);
                    corners
                },
                edge_configuration: EdgeConfiguration::cycle([[
                    Edge::Df,
                    Edge::Dr,
                    Edge::Db,
                    Edge::Dl,
                ]]),
            },
            part: Face(Faces::D),
            modifier: Clockwise,
        },
        MoveInformation {
            cube_state: Cube3By3 {
                center_configuration: CenterConfiguration::identity(),
                corner_configuration: {
                    let mut corners = CornerConfiguration::cycle([[
                        Corner::Dfr,
                        Corner::Dfl,
                        Corner::Ufl,
                        Corner::Ufr,
                    ]]);
                    corners.orientation = Zn::array([0, 0, 1, 2, 1, 2, 0, 0]);
                    corners
                },
                edge_configuration: EdgeConfiguration {
                    permutation: EdgeConfiguration::cycle([[
                        Edge::Df,
                        Edge::Fl,
                        Edge::Uf,
                        Edge::Fr,
                    ]])
                    .permutation,
                    orientation: Zn::array([0, 0, 1, 0, 1, 1, 0, 0, 1, 0, 0, 0]),
                },
            },
            part: Face(Faces::F),
            modifier: Clockwise,
        },
        MoveInformation {
            cube_state: Cube3By3 {
                center_configuration: CenterConfiguration::identity(),
                corner_configuration: {
                    let mut corners = CornerConfiguration::cycle([[
                        Corner::Ubr,
                        Corner::Ubl,
                        Corner::Dbl,
                        Corner::Dbr,
                    ]]);
                    corners.orientation = Zn::array([1, 2, 0, 0, 0, 0, 1, 2]);
                    corners
                },
                edge_configuration: EdgeConfiguration {
                    permutation: EdgeConfiguration::cycle([[
                        Edge::Br,
                        Edge::Ub,
                        Edge::Bl,
                        Edge::Db,
                    ]])
                    .permutation,
                    orientation: Zn::array([1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0]),
                },
            },
            part: Face(Faces::B),
            modifier: Clockwise,
        },
        MoveInformation {
            cube_state: Cube3By3 {
                center_configuration: CenterConfiguration::cycle([[
                    Center::F,
                    Center::R,
                    Center::B,
                    Center::L,
                ]]),
                corner_configuration: CornerConfiguration::identity(),
                edge_configuration: EdgeConfiguration {
                    permutation: EdgeConfiguration::cycle([[
                        Edge::Fr,
                        Edge::Br,
                        Edge::Bl,
                        Edge::Fl,
                    ]])
                    .permutation,
                    orientation: Zn::array([0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0]),
                },
            },
            part: Slice(Slices::E),
            modifier: Clockwise,
        },
        MoveInformation {
            cube_state: Cube3By3 {
                center_configuration: CenterConfiguration::cycle([[
                    Center::F,
                    Center::D,
                    Center::B,
                    Center::U,
                ]]),
                corner_configuration: CornerConfiguration::identity(),
                edge_configuration: EdgeConfiguration {
                    permutation: EdgeConfiguration::cycle([[
                        Edge::Uf,
                        Edge::Df,
                        Edge::Db,
                        Edge::Ub,
                    ]])
                    .permutation,
                    orientation: Zn::array([1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0]),
                },
            },
            part: Slice(Slices::M),
            modifier: Clockwise,
        },
        MoveInformation {
            cube_state: Cube3By3 {
                center_configuration: CenterConfiguration::cycle([[
                    Center::U,
                    Center::R,
                    Center::D,
                    Center::L,
                ]]),
                corner_configuration: CornerConfiguration::identity(),
                edge_configuration: EdgeConfiguration {
                    permutation: EdgeConfiguration::cycle([[
                        Edge::Ul,
                        Edge::Ur,
                        Edge::Dr,
                        Edge::Dl,
                    ]])
                    .permutation,
                    orientation: Zn::array([0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1]),
                },
            },
            part: Slice(Slices::S),
            modifier: Clockwise,
        },
    ]
});

#[expect(clippy::panic, reason = "TODO")]
fn slice_along(face: Faces, placed: &[MoveInformation; CLOCKWISE_MOVE_COUNT]) -> Cube3By3 {
    for s in Slices::ALL {
        if s.follows() as u8 == face as u8 {
            return placed[table_index_clockwise(Slice(s))].cube_state;
        } else if s.follows().opposite() as u8 == face as u8 {
            return placed[table_index_clockwise(Slice(s))].cube_state.inverse();
        }
    }
    panic!("all faces must have a slice_along")
}

const CLOCKWISE_MOVE_COUNT: usize =
    FACE_AND_SLICES_CLOCKWISE_MOVE_COUNT + Faces::ALL.len() + Rotations::ALL.len();
static ALL_CLOCKWISE_MOVES: LazyLock<[MoveInformation; CLOCKWISE_MOVE_COUNT]> =
    LazyLock::new(|| {
        let mut all_clockwise_moves = [MoveInformation::default(); CLOCKWISE_MOVE_COUNT];
        let mut i = 0;
        while i < FACE_AND_SLICES_CLOCKWISE_MOVE_COUNT {
            all_clockwise_moves
                [table_index_clockwise(ALL_FACE_AND_SLICES_CLOCKWISE_MOVES[i].part)] =
                ALL_FACE_AND_SLICES_CLOCKWISE_MOVES[i];
            i += 1;
        }

        let mut i = 0;
        while i < Rotations::ALL.len() {
            all_clockwise_moves[table_index_clockwise(Rotation(Rotations::ALL[i]))] =
                MoveInformation {
                    cube_state: all_clockwise_moves
                        [table_index_clockwise(Face(Rotations::ALL[i].follows()))]
                    .cube_state
                        * (all_clockwise_moves
                            [table_index_clockwise(Face(Rotations::ALL[i].follows().opposite()))]
                        .cube_state
                        .inverse())
                        * (slice_along(Rotations::ALL[i].follows(), &all_clockwise_moves)),
                    part: Rotation(Rotations::ALL[i]),
                    modifier: Clockwise,
                };
            i += 1;
        }
        let mut i = 0;
        while i < Faces::ALL.len() {
            let face = Faces::ALL[i];
            all_clockwise_moves[table_index_clockwise(Wide(face))] = MoveInformation {
                cube_state: all_clockwise_moves[table_index_clockwise(Face(face))].cube_state
                    * (slice_along(face, &all_clockwise_moves)),
                part: Wide(face),
                modifier: Clockwise,
            };
            i += 1;
        }

        all_clockwise_moves
    });

static ALL_MOVES: LazyLock<[MoveInformation; 3 * CLOCKWISE_MOVE_COUNT]> = LazyLock::new(|| {
    let mut all_moves = [MoveInformation::default(); 3 * CLOCKWISE_MOVE_COUNT];
    let mut i = 0;
    while i < CLOCKWISE_MOVE_COUNT {
        all_moves[3 * i] = ALL_CLOCKWISE_MOVES[i];
        all_moves[3 * i + 1] = ALL_CLOCKWISE_MOVES[i].inverse();
        all_moves[3 * i + 2] = ALL_CLOCKWISE_MOVES[i].double();
        i += 1;
    }
    let mut i = 0;
    while i < all_moves.len() {
        assert_eq!(i, table_index(all_moves[i].part, all_moves[i].modifier));
        i += 1;
    }
    all_moves
});

#[cfg(test)]
mod tests {
    //! Tests of the table's own construction: derivation identities and
    //! modifier consistency. Handedness pins, the facts from the physical cube
    //! that catch a mirrored base move, live in `tests/testing.rs` and go
    //! through the public queries.
    use super::*;

    fn clockwise(part: MovablePart) -> Cube3By3 {
        cube_state(part, Clockwise)
    }

    // Derivation identities: these pin the derivation, not the base moves.
    // A mirrored slice mirrors its rotation with it and still passes here.

    fn seq(parts: &[(MovablePart, MoveModifier)]) -> Cube3By3 {
        parts.iter().fold(Cube3By3::default(), |cube, &(p, m)| {
            cube * (cube_state(p, m))
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
        for entry in *ALL_CLOCKWISE_MOVES {
            let part = entry.part;
            let cw = cube_state(part, Clockwise);
            let ccw = cube_state(part, CounterClockwise);
            assert_eq!(
                cw * ccw,
                Cube3By3::default(),
                "{part:?}' must undo {part:?}"
            );
            assert_eq!(
                cube_state(part, Double),
                cw * cw,
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
