use std::sync::{Arc, LazyLock};

use crate::{
    Cube3x3, Mask, Method, Puzzle, SearchStep, Step,
    puzzles::cube3by3::{
        moves::{MovablePart, Move3x3, MoveModifier},
        pieces::{Faces, Pieces3x3, Slices},
    },
};

const FB_FRONT_SQUARE_PIECES: [Pieces3x3; 4] = [
    Pieces3x3::Center(crate::Center::L),
    Pieces3x3::Corner(crate::Corner::Dfl),
    Pieces3x3::Edge(crate::Edge::Fl),
    Pieces3x3::Edge(crate::Edge::Dl),
];

const FB_BACK_SQUARE_PIECES: [Pieces3x3; 4] = [
    Pieces3x3::Center(crate::Center::L),
    Pieces3x3::Corner(crate::Corner::Dbl),
    Pieces3x3::Edge(crate::Edge::Bl),
    Pieces3x3::Edge(crate::Edge::Dl),
];

const FB_PIECES: [Pieces3x3; 6] = [
    Pieces3x3::Center(crate::Center::L),
    Pieces3x3::Corner(crate::Corner::Dfl),
    Pieces3x3::Corner(crate::Corner::Dbl),
    Pieces3x3::Edge(crate::Edge::Fl),
    Pieces3x3::Edge(crate::Edge::Dl),
    Pieces3x3::Edge(crate::Edge::Bl),
];

const SB_SQUARE_PIECES: [Pieces3x3; 4] = [
    Pieces3x3::Center(crate::Center::R),
    Pieces3x3::Edge(crate::Edge::Dr),
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

fn moves_of(parts: &[MovablePart]) -> Vec<(Vec<Move3x3>, bool)> {
    parts.iter().fold(Vec::new(), |mut v, &part| {
        v.push((vec![Move3x3::new(part, MoveModifier::Clockwise)], true));
        v.push((
            vec![Move3x3::new(part, MoveModifier::CounterClockwise)],
            true,
        ));
        v.push((vec![Move3x3::new(part, MoveModifier::Double)], true));
        v
    })
}

fn algorithms_with_free_auf(algset: &str) -> Vec<(Vec<Move3x3>, bool)> {
    [("U", false), ("U2", false), ("U'", false)]
        .iter()
        .copied()
        .chain(algset.lines().map(|line| (line, true)))
        .map(|(r, has_cost)| {
            (
                Move3x3::sequence(r)
                    .map(|p| p.expect("manually curated sequences should always parse"))
                    .collect(),
                has_cost,
            )
        })
        .collect()
}

/// One-look CMLL algorithms, one per line.
const CMLL_ONE_LOOK_ALGS: &str = include_str!("cmll/one_look.txt");
/// Algorithms that orient the U-layer corners, one per line.
const CMLL_ORIENTATION_ALGS: &str = include_str!("cmll/co.txt");
/// Algorithms that permute oriented U-layer corners, one per line.
const CMLL_PERMUTATION_ALGS: &str = include_str!("cmll/cp.txt");

/// The Roux steps in solving order, built once on first use.
static ALL_ROUX_STEPS: LazyLock<Vec<Arc<dyn Step<Cube3x3>>>> = LazyLock::new(|| {
    let mut all_parts: Vec<MovablePart> = Cube3x3::ALL_MOVES.iter().map(|m| m.part).collect();
    all_parts.dedup();

    let fb_front_square = SearchStep::new(
        "FB Square",
        Mask::<Cube3x3>::default(),
        Mask::<Cube3x3>::new_from_pieces(FB_FRONT_SQUARE_PIECES),
        moves_of(&all_parts),
    );

    let fb_back_square = SearchStep::new(
        "FB Square",
        Mask::<Cube3x3>::default(),
        Mask::<Cube3x3>::new_from_pieces(FB_BACK_SQUARE_PIECES),
        moves_of(&all_parts),
    );

    let fb_front_pair = SearchStep::new(
        "FB Pair",
        Mask::<Cube3x3>::new_from_pieces(FB_BACK_SQUARE_PIECES),
        Mask::<Cube3x3>::new_from_pieces(FB_PIECES),
        moves_of(&all_parts),
    );

    let fb_back_pair = SearchStep::new(
        "FB Pair",
        Mask::<Cube3x3>::new_from_pieces(FB_FRONT_SQUARE_PIECES),
        Mask::<Cube3x3>::new_from_pieces(FB_PIECES),
        moves_of(&all_parts),
    );

    let fb = SearchStep::new(
        "FB",
        Mask::<Cube3x3>::new_from_pieces([]),
        Mask::<Cube3x3>::new_from_pieces(FB_PIECES),
        moves_of(&all_parts),
    );

    let second_block_moves = [
        MovablePart::Face(Faces::U),
        MovablePart::Face(Faces::R),
        MovablePart::Slice(Slices::M),
        MovablePart::Wide(Faces::R),
    ];

    let sb_square = SearchStep::new(
        "SB Square",
        fb.after(),
        Mask::<Cube3x3>::new_from_pieces(FB_PIECES.iter().chain(SB_SQUARE_PIECES.iter()).copied()),
        moves_of(&second_block_moves),
    );

    let sb_pair = SearchStep::new(
        "SB Pair",
        sb_square.after(),
        Mask::<Cube3x3>::new_from_pieces(
            FB_PIECES
                .iter()
                .chain(SB_SQUARE_PIECES.iter())
                .chain(SB_PIECES.iter())
                .copied(),
        ),
        moves_of(&second_block_moves),
    );

    let sb = SearchStep::new(
        "SB",
        fb.after(),
        Mask::<Cube3x3>::new_from_pieces(FB_PIECES.iter().chain(SB_PIECES.iter()).copied()),
        moves_of(&second_block_moves),
    );

    let cmll_after = Mask::<Cube3x3>::new_from_pieces(
        FB_PIECES
            .iter()
            .chain(SB_PIECES.iter())
            .chain(CMLL_PIECES.iter())
            .copied(),
    );
    let cmll = SearchStep::new(
        "CMLL",
        sb.after(),
        cmll_after.clone(),
        algorithms_with_free_auf(CMLL_ONE_LOOK_ALGS),
    );

    let cmll_orientation_after = Mask::<Cube3x3>::new(
        FB_PIECES.iter().chain(SB_PIECES.iter()).copied(),
        FB_PIECES
            .iter()
            .chain(SB_PIECES.iter())
            .chain(CMLL_PIECES.iter())
            .copied(),
    );
    let cmll_orientation = SearchStep::new(
        "CO",
        sb.after(),
        cmll_orientation_after.clone(),
        algorithms_with_free_auf(CMLL_ORIENTATION_ALGS),
    );

    let cmll_permutation = SearchStep::new(
        "CP",
        cmll_orientation_after,
        cmll_after,
        algorithms_with_free_auf(CMLL_PERMUTATION_ALGS),
    );

    let lse = SearchStep::new(
        "LSE",
        cmll.after(),
        Mask::<Cube3x3>::new_from_pieces(
            FB_PIECES
                .iter()
                .chain(SB_PIECES.iter())
                .chain(CMLL_PIECES.iter())
                .chain(LSE_PIECES.iter())
                .copied(),
        ),
        moves_of(&[MovablePart::Face(Faces::U), MovablePart::Slice(Slices::M)]),
    );

    [
        fb_front_square,
        fb_back_square,
        fb_front_pair,
        fb_back_pair,
        fb,
        sb_square,
        sb_pair,
        sb,
        cmll,
        cmll_orientation,
        cmll_permutation,
        lse,
    ]
    .into_iter()
    .map(|s| Arc::new(s) as Arc<dyn Step<Cube3x3>>)
    .collect::<Vec<Arc<dyn Step<Cube3x3>>>>()
});

#[derive(Clone, Debug, Copy)]
pub struct RouxOptions {
    pub sb_as_one_step: bool,
    pub fb_as_one_step: bool,
    pub one_look_cmll: bool,
}

impl Default for RouxOptions {
    fn default() -> Self {
        Self {
            sb_as_one_step: true,
            fb_as_one_step: true,
            one_look_cmll: true,
        }
    }
}

impl Method<Cube3x3> {
    pub fn roux(options: RouxOptions) -> Self {
        let mut steps = ALL_ROUX_STEPS.clone();

        if options.sb_as_one_step {
            steps.retain(|s| s.name() != "SB Square" && s.name() != "SB Pair");
        } else {
            steps.retain(|s| s.name() != "SB");
        }

        if options.fb_as_one_step {
            steps.retain(|s| s.name() != "FB Square" && s.name() != "FB Pair");
        } else {
            steps.retain(|s| s.name() != "FB");
        }

        if options.one_look_cmll {
            steps.retain(|s| s.name() != "CO" && s.name() != "CP");
        } else {
            steps.retain(|s| s.name() != "CMLL");
        }

        Self::new("Roux", steps)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;
    use crate::{Cube3x3, Inv, Puzzle};

    #[test]
    fn all_step_pieces_are_all_cube_pieces() {
        assert_eq!(
            FB_PIECES
                .iter()
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
    fn cmll_step_leaves_the_cube_with_cmll_solved() {
        let cmll = ALL_ROUX_STEPS.iter().find(|s| s.name() == "CMLL").unwrap();
        let sune = Cube3x3::from_solved("R U R' U R U2 R'").unwrap();
        for scramble in [Cube3x3::from_solved("U2").unwrap(), sune.inverse()] {
            let mut cube = scramble;
            cmll.solve(&mut cube).unwrap();
            assert!(cmll.is_done(&cube), "not solved:\n{cube}");
        }
    }

    #[test]
    fn step_requirements_meet_bound_conditions() {
        for (i, step) in ALL_ROUX_STEPS.iter().enumerate() {
            let needs = step.needs_solved();
            assert!(
                needs == Mask::default()
                    || ALL_ROUX_STEPS
                        .iter()
                        .take(i)
                        .any(|earlier| earlier.solved_pieces() == needs),
                "step {i} ({}) needs pieces that no earlier step solves",
                step.name()
            );
        }
    }
}
