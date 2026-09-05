#[allow(clippy::wildcard_imports)]
use crate::{
    cube3by3::{Cube3By3, pieces::*},
    ops,
    zn::ZnRing,
};

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum MovablePart {
    Face(Faces),
    Slice(Slices),
    Rotation(Rotations),
}
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
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
    is_slice: bool,
    is_rotation: bool,
    part: MovablePart,
    modifier: MoveModifier,
}
impl ops::Inv for Move {
    fn inverse(&self) -> Self {
        self.const_inverse()
    }
}
impl Move {
    const fn const_inverse(&self) -> Self {
        Move {
            cube_representation: self.cube_representation.const_inverse(),
            is_slice: self.is_slice,
            is_rotation: self.is_rotation,
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
    const fn const_double(&self) -> Move {
        Move {
            cube_representation: self.cube_representation.const_mul(self.cube_representation),
            is_slice: self.is_slice,
            is_rotation: self.is_rotation,
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

impl From<&str> for Move {
    fn from(s: &str) -> Self {
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
        let modif = {
            if s.len() == 1 {
                MoveModifier::Clockwise
            } else {
                match &s[1..] {
                    "'" => MoveModifier::CounterClockwise,
                    "2" => MoveModifier::Double,
                    "2'" | "'2" => MoveModifier::CounterDouble,
                    _ => panic!("invalid move"),
                }
            }
        };
        *ALL_MOVES
            .iter()
            .find(|&m| m.part == part && m.modifier == modif)
            .unwrap()
    }
}

impl From<Move> for Cube3By3 {
    fn from(m: Move) -> Self {
        m.cube_representation
    }
}

const CLOCKWISE_MOVE_COUNT: usize = 6;
const ALL_CLOCKWISE_MOVES: [Move; CLOCKWISE_MOVE_COUNT] = [
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
                SingleEdge::Fr,
                SingleEdge::Ur,
                SingleEdge::Br,
                SingleEdge::Dr,
            ]]),
        },
        is_slice: false,
        is_rotation: false,
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
        part: MovablePart::Face(Faces::D),
        modifier: MoveModifier::Clockwise,
    },
    Move {
        cube_representation: Cube3By3 {
            center_configuration: Centers::cycle([[
                SingleCenter::F,
                SingleCenter::L,
                SingleCenter::B,
                SingleCenter::R,
            ]]),
            corner_configuration: Corners::IDENTITY,
            edge_configuration: Edges::cycle([[
                SingleEdge::Fr,
                SingleEdge::Fl,
                SingleEdge::Bl,
                SingleEdge::Br,
            ]]),
        },
        is_slice: true,
        is_rotation: false,
        part: MovablePart::Slice(Slices::E),
        modifier: MoveModifier::Clockwise,
    },
    Move {
        cube_representation: Cube3By3 {
            center_configuration: Centers::cycle([[
                SingleCenter::F,
                SingleCenter::L,
                SingleCenter::B,
                SingleCenter::R,
            ]]),
            corner_configuration: {
                let mut corners = Corners::cycle([
                    [
                        SingleCorner::Ufr,
                        SingleCorner::Ufl,
                        SingleCorner::Ubl,
                        SingleCorner::Ubr,
                    ],
                    [
                        SingleCorner::Dfr,
                        SingleCorner::Dfl,
                        SingleCorner::Dbl,
                        SingleCorner::Dbr,
                    ],
                ]);
                corners.orientation = [ZnRing::ZERO; CORNERS_COUNT];
                corners
            },
            edge_configuration: Edges::cycle([
                [
                    SingleEdge::Uf,
                    SingleEdge::Ul,
                    SingleEdge::Ub,
                    SingleEdge::Ur,
                ],
                [
                    SingleEdge::Df,
                    SingleEdge::Dl,
                    SingleEdge::Db,
                    SingleEdge::Dr,
                ],
                [
                    SingleEdge::Fr,
                    SingleEdge::Fl,
                    SingleEdge::Bl,
                    SingleEdge::Br,
                ],
            ]),
        },
        is_slice: false,
        is_rotation: true,
        part: MovablePart::Rotation(Rotations::y),
        modifier: MoveModifier::Clockwise,
    },
];

pub const ALL_MOVES: [Move; 3 * CLOCKWISE_MOVE_COUNT] = {
    let mut all_moves = [Move {
        cube_representation: Cube3By3::IDENTITY,
        is_slice: false,
        is_rotation: false,
        part: MovablePart::Face(Faces::R),
        modifier: MoveModifier::Clockwise,
    }; 3 * CLOCKWISE_MOVE_COUNT];
    let mut i: usize = 0;
    while i < CLOCKWISE_MOVE_COUNT {
        all_moves[3 * i] = ALL_CLOCKWISE_MOVES[i];
        all_moves[3 * i + 1] = ALL_CLOCKWISE_MOVES[i].const_inverse();
        all_moves[3 * i + 2] = ALL_CLOCKWISE_MOVES[i].const_double();
        i += 1;
    }
    all_moves
};
