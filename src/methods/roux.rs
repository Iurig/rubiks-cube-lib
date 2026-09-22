use std::sync::LazyLock;

use crate::{
    SimpleMethod3x3, SimpleStep3x3,
    puzzles::cube3by3::{
        moves::MovablePart,
        pieces::{Faces, Pieces3x3, Slices},
    },
};

const FB_PIECES: [Pieces3x3; 6] = [
    Pieces3x3::Center(crate::Center::L),
    Pieces3x3::Corner(crate::Corner::Dfl),
    Pieces3x3::Corner(crate::Corner::Dbl),
    Pieces3x3::Edge(crate::Edge::Fl),
    Pieces3x3::Edge(crate::Edge::Dl),
    Pieces3x3::Edge(crate::Edge::Bl),
];

const SB_EDGE_PIECES: [Pieces3x3; 2] = [
    Pieces3x3::Center(crate::Center::R),
    Pieces3x3::Edge(crate::Edge::Dr),
];

const SB_SQUARE_PIECES: [Pieces3x3; 2] = [
    Pieces3x3::Corner(crate::Corner::Dbr),
    Pieces3x3::Edge(crate::Edge::Br),
];

const SB_PIECES: [Pieces3x3; 6] = [
    Pieces3x3::Center(crate::Center::R),
    Pieces3x3::Edge(crate::Edge::Dr),
    Pieces3x3::Corner(crate::Corner::Dbr),
    Pieces3x3::Edge(crate::Edge::Br),
    Pieces3x3::Edge(crate::Edge::Fr),
    Pieces3x3::Corner(crate::Corner::Dfr),
];

const CMLL_PIECES: [Pieces3x3; 4] = [
    Pieces3x3::Corner(crate::Corner::Ufr),
    Pieces3x3::Corner(crate::Corner::Ufl),
    Pieces3x3::Corner(crate::Corner::Ubr),
    Pieces3x3::Corner(crate::Corner::Ubl),
];

const LSE_PIECES: [Pieces3x3; 10] = [
    Pieces3x3::Edge(crate::Edge::Ub),
    Pieces3x3::Edge(crate::Edge::Ur),
    Pieces3x3::Edge(crate::Edge::Ul),
    Pieces3x3::Edge(crate::Edge::Uf),
    Pieces3x3::Edge(crate::Edge::Df),
    Pieces3x3::Edge(crate::Edge::Db),
    Pieces3x3::Center(crate::Center::U),
    Pieces3x3::Center(crate::Center::F),
    Pieces3x3::Center(crate::Center::D),
    Pieces3x3::Center(crate::Center::B),
];

pub static FB: LazyLock<SimpleStep3x3> = LazyLock::new(|| SimpleStep3x3 {
    name: "FB".to_string(),
    before: Box::new([]),
    after: Box::new(FB_PIECES),
    allowed: |_| true,
});

pub static SB_EDGE: LazyLock<SimpleStep3x3> = LazyLock::new(|| SimpleStep3x3 {
    name: "SB Edge".to_string(),
    before: FB.after.clone(),
    after: FB
        .after
        .iter()
        .chain(SB_EDGE_PIECES.iter())
        .copied()
        .collect::<Vec<Pieces3x3>>()
        .into_boxed_slice(),
    allowed: |&m| {
        [
            MovablePart::Face(Faces::U),
            MovablePart::Face(Faces::R),
            MovablePart::Slice(Slices::M),
            MovablePart::Wide(Faces::R),
        ]
        .contains(&m.part)
    },
});

pub static SB_SQUARE: LazyLock<SimpleStep3x3> = LazyLock::new(|| SimpleStep3x3 {
    name: "SB Square".to_string(),
    before: SB_EDGE.after.clone(),
    after: FB
        .after
        .iter()
        .chain(SB_SQUARE_PIECES.iter())
        .copied()
        .collect::<Vec<Pieces3x3>>()
        .into_boxed_slice(),
    allowed: |&m| {
        [
            MovablePart::Face(Faces::U),
            MovablePart::Face(Faces::R),
            MovablePart::Slice(Slices::M),
            MovablePart::Wide(Faces::R),
        ]
        .contains(&m.part)
    },
});

pub static SB: LazyLock<SimpleStep3x3> = LazyLock::new(|| SimpleStep3x3 {
    name: "SB Pair".to_string(),
    before: SB_SQUARE.after.clone(),
    after: FB
        .after
        .iter()
        .chain(SB_PIECES.iter())
        .copied()
        .collect::<Vec<Pieces3x3>>()
        .into_boxed_slice(),
    allowed: |&m| {
        [
            MovablePart::Face(Faces::U),
            MovablePart::Face(Faces::R),
            MovablePart::Slice(Slices::M),
            MovablePart::Wide(Faces::R),
        ]
        .contains(&m.part)
    },
});

pub static CMLL: LazyLock<SimpleStep3x3> = LazyLock::new(|| SimpleStep3x3 {
    name: "CMLL".to_string(),
    before: SB.after.clone(),
    after: SB
        .after
        .iter()
        .chain(CMLL_PIECES.iter())
        .copied()
        .collect::<Vec<Pieces3x3>>()
        .into_boxed_slice(),
    allowed: |_| true,
});

pub static LSE: LazyLock<SimpleStep3x3> = LazyLock::new(|| SimpleStep3x3 {
    name: "LSE".to_string(),
    before: CMLL.after.clone(),
    after: CMLL
        .after
        .iter()
        .chain(LSE_PIECES.iter())
        .copied()
        .collect::<Vec<Pieces3x3>>()
        .into_boxed_slice(),
    allowed: |&m| [MovablePart::Face(Faces::U), MovablePart::Slice(Slices::M)].contains(&m.part),
});

pub static ROUX: LazyLock<SimpleMethod3x3> = LazyLock::new(|| {
    SimpleMethod3x3(vec![
        FB.clone(),
        SB_EDGE.clone(),
        SB_SQUARE.clone(),
        SB.clone(),
        CMLL.clone(),
        LSE.clone(),
    ])
});

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;
    use crate::{Cube3x3, Puzzle};

    #[test]
    fn all_step_pieces_are_all_cube_pieces() {
        assert_eq!(
            FB_PIECES
                .iter()
                .chain(SB_EDGE_PIECES.iter())
                .chain(SB_SQUARE_PIECES.iter())
                .chain(SB_PIECES.iter())
                .chain(CMLL_PIECES.iter())
                .chain(LSE_PIECES.iter())
                .copied()
                .collect::<HashSet<Pieces3x3>>(),
            HashSet::from_iter(Cube3x3::ALL_PIECES.iter().copied())
        );
    }

    #[test]
    fn step_requirements_meet_bound_conditions() {
        for (i, step) in ROUX.0.iter().enumerate() {
            match i {
                0 => {
                    assert_eq!(*step.before, []);
                    assert_eq!(*step.after, *ROUX.0[i + 1].before);
                }
                3 => {
                    assert_eq!(
                        step.after.iter().copied().collect::<HashSet<Pieces3x3>>(),
                        HashSet::from_iter(Cube3x3::ALL_PIECES.iter().copied())
                    );
                    assert_eq!(*step.before, *ROUX.0[i - 1].after);
                }
                _ => {
                    assert_eq!(*step.before, *ROUX.0[i - 1].after);
                    assert_eq!(*step.after, *ROUX.0[i + 1].before);
                }
            }
        }
    }
}
