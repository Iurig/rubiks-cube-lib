//! The table of every implemented move, built at compile time.
//!
//! The nine face and slice moves are written out by hand; the three rotations
//! are derived from them, and every move's inverse and double are then generated.

#[allow(clippy::wildcard_imports)]
use crate::{
    cube3by3::{Cube3By3, pieces::*},
    zn::ZnRing,
};

use super::{MovablePart, MovablePart::Rotation, Move, MoveModifier};

const FACE_AND_SLICES_CLOCKWISE_MOVE_COUNT: usize = 9;
const ALL_FACE_AND_SLICES_CLOCKWISE_MOVES: [Move; FACE_AND_SLICES_CLOCKWISE_MOVE_COUNT] = [
    Move {
        cube_representation: Cube3By3 {
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
        is_slice: false,
        is_rotation: false,
        is_wide: false,
        part: MovablePart::Face(Faces::R),
        modifier: MoveModifier::Clockwise,
    },
    Move {
        cube_representation: Cube3By3 {
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
        is_slice: false,
        is_rotation: false,
        is_wide: false,
        part: MovablePart::Face(Faces::L),
        modifier: MoveModifier::Clockwise,
    },
    Move {
        cube_representation: Cube3By3 {
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
        is_slice: false,
        is_rotation: false,
        is_wide: false,
        part: MovablePart::Face(Faces::U),
        modifier: MoveModifier::Clockwise,
    },
    Move {
        cube_representation: Cube3By3 {
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
        is_slice: false,
        is_rotation: false,
        is_wide: false,
        part: MovablePart::Face(Faces::D),
        modifier: MoveModifier::Clockwise,
    },
    Move {
        cube_representation: Cube3By3 {
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
        is_slice: false,
        is_rotation: false,
        is_wide: false,
        part: MovablePart::Face(Faces::F),
        modifier: MoveModifier::Clockwise,
    },
    Move {
        cube_representation: Cube3By3 {
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
        is_slice: false,
        is_rotation: false,
        is_wide: false,
        part: MovablePart::Face(Faces::B),
        modifier: MoveModifier::Clockwise,
    },
    Move {
        cube_representation: Cube3By3 {
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
        is_slice: true,
        is_rotation: false,
        is_wide: false,
        part: MovablePart::Slice(Slices::E),
        modifier: MoveModifier::Clockwise,
    },
    Move {
        cube_representation: Cube3By3 {
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
        is_slice: true,
        is_rotation: false,
        is_wide: false,
        part: MovablePart::Slice(Slices::M),
        modifier: MoveModifier::Clockwise,
    },
    Move {
        cube_representation: Cube3By3 {
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
        is_slice: true,
        is_rotation: false,
        is_wide: false,
        part: MovablePart::Slice(Slices::S),
        modifier: MoveModifier::Clockwise,
    },
];

const CLOCKWISE_MOVE_COUNT: usize = FACE_AND_SLICES_CLOCKWISE_MOVE_COUNT + 3;
const ALL_CLOCKWISE_MOVES: [Move; CLOCKWISE_MOVE_COUNT] = {
    let mut all_clockwise_moves = [Move::IDENTITY; CLOCKWISE_MOVE_COUNT];
    let mut i = 0;
    while i < FACE_AND_SLICES_CLOCKWISE_MOVE_COUNT {
        all_clockwise_moves[i] = ALL_FACE_AND_SLICES_CLOCKWISE_MOVES[i];
        i += 1;
    }
    all_clockwise_moves[Rotation(Rotations::z).table_index()] = Move {
        cube_representation: ALL_FACE_AND_SLICES_CLOCKWISE_MOVES
            [FACE_AND_SLICES_CLOCKWISE_MOVE_COUNT - 1]
            .cube_representation
            .const_mul(ALL_FACE_AND_SLICES_CLOCKWISE_MOVES[4].cube_representation)
            .const_mul(
                ALL_FACE_AND_SLICES_CLOCKWISE_MOVES[5]
                    .cube_representation
                    .const_inverse(),
            ),
        part: Rotation(Rotations::z),
        modifier: MoveModifier::Clockwise,
        is_rotation: true,
        is_wide: false,
        is_slice: false,
    };
    all_clockwise_moves[Rotation(Rotations::x).table_index()] = Move {
        cube_representation: ALL_FACE_AND_SLICES_CLOCKWISE_MOVES
            [FACE_AND_SLICES_CLOCKWISE_MOVE_COUNT - 2]
            .cube_representation
            .const_inverse()
            .const_mul(ALL_FACE_AND_SLICES_CLOCKWISE_MOVES[0].cube_representation)
            .const_mul(
                ALL_FACE_AND_SLICES_CLOCKWISE_MOVES[1]
                    .cube_representation
                    .const_inverse(),
            ),
        part: Rotation(Rotations::x),
        modifier: MoveModifier::Clockwise,
        is_rotation: true,
        is_wide: false,
        is_slice: false,
    };
    all_clockwise_moves[Rotation(Rotations::y).table_index()] = Move {
        cube_representation: ALL_FACE_AND_SLICES_CLOCKWISE_MOVES
            [FACE_AND_SLICES_CLOCKWISE_MOVE_COUNT - 3]
            .cube_representation
            .const_inverse()
            .const_mul(ALL_FACE_AND_SLICES_CLOCKWISE_MOVES[2].cube_representation)
            .const_mul(
                ALL_FACE_AND_SLICES_CLOCKWISE_MOVES[3]
                    .cube_representation
                    .const_inverse(),
            ),
        part: Rotation(Rotations::y),
        modifier: MoveModifier::Clockwise,
        is_rotation: true,
        is_wide: false,
        is_slice: false,
    };
    all_clockwise_moves
};

pub const ALL_MOVES: [Move; 3 * CLOCKWISE_MOVE_COUNT] = {
    let mut all_moves = [Move::IDENTITY; 3 * CLOCKWISE_MOVE_COUNT];
    let mut i = 0;
    while i < CLOCKWISE_MOVE_COUNT {
        all_moves[3 * i] = ALL_CLOCKWISE_MOVES[i];
        all_moves[3 * i + 1] = ALL_CLOCKWISE_MOVES[i].const_inverse();
        all_moves[3 * i + 2] = ALL_CLOCKWISE_MOVES[i].const_double();
        i += 1;
    }
    all_moves
};
