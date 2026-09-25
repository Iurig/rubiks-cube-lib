use std::sync::LazyLock;

use crate::{
    Cube3x3, Mask, Puzzle, SimpleStep, SolveMethod, SolveStep,
    puzzles::cube3by3::{
        moves::{MovablePart, Move3x3, MoveModifier},
        pieces::{Faces, Pieces3x3, Slices},
    },
};

#[derive(Clone, Default, Debug)]
pub struct Roux(RouxOptions);

impl Roux {
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
}

/// One-look CMLL algorithms, one per line.
const CMLL_ONE_LOOK_ALGS: &str = include_str!("cmll/one_look.txt");
/// Algorithms that orient the U-layer corners, one per line.
const CMLL_ORIENTATION_ALGS: &str = include_str!("cmll/co.txt");
/// Algorithms that permute oriented U-layer corners, one per line.
const CMLL_PERMUTATION_ALGS: &str = include_str!("cmll/cp.txt");

/// The Roux steps in solving order, built once on first use.
static STEPS: LazyLock<Vec<SimpleStep<Cube3x3, Roux>>> = LazyLock::new(roux_steps);

/// Builds the step chain: each step's `before` is the previous step's `after`.
#[expect(
    clippy::too_many_lines,
    reason = "one literal per step, in solving order"
)]
fn roux_steps() -> Vec<SimpleStep<Cube3x3, Roux>> {
    let mut all_parts: Vec<MovablePart> = Cube3x3::ALL_MOVES.iter().map(|m| m.part).collect();
    all_parts.dedup();

    let fb_front_square = SimpleStep::new(
        "FB Square",
        Mask::<Cube3x3>::default(),
        Mask::<Cube3x3>::new_from_pieces(Roux::FB_FRONT_SQUARE_PIECES),
        Roux::moves_of(&all_parts),
        |m: &Roux| !m.0.fb_as_one_step,
    );

    let fb_back_square = SimpleStep::new(
        "FB Square",
        Mask::<Cube3x3>::default(),
        Mask::<Cube3x3>::new_from_pieces(Roux::FB_BACK_SQUARE_PIECES),
        Roux::moves_of(&all_parts),
        |m: &Roux| !m.0.fb_as_one_step,
    );

    let fb_front_pair = SimpleStep::new(
        "FB Pair",
        Mask::<Cube3x3>::new_from_pieces(Roux::FB_BACK_SQUARE_PIECES),
        Mask::<Cube3x3>::new_from_pieces(Roux::FB_PIECES),
        Roux::moves_of(&all_parts),
        |m: &Roux| !m.0.fb_as_one_step,
    );

    let fb_back_pair = SimpleStep::new(
        "FB Pair",
        Mask::<Cube3x3>::new_from_pieces(Roux::FB_FRONT_SQUARE_PIECES),
        Mask::<Cube3x3>::new_from_pieces(Roux::FB_PIECES),
        Roux::moves_of(&all_parts),
        |m: &Roux| !m.0.fb_as_one_step,
    );

    let fb = SimpleStep::new(
        "FB",
        Mask::<Cube3x3>::new_from_pieces([]),
        Mask::<Cube3x3>::new_from_pieces(Roux::FB_PIECES),
        Roux::moves_of(&all_parts),
        |_| true,
    );

    let second_block_moves = [
        MovablePart::Face(Faces::U),
        MovablePart::Face(Faces::R),
        MovablePart::Slice(Slices::M),
        MovablePart::Wide(Faces::R),
    ];

    let sb_square = SimpleStep::new(
        "SB Square",
        fb.solved_pieces(),
        Mask::<Cube3x3>::new_from_pieces(
            Roux::FB_PIECES
                .iter()
                .chain(Roux::SB_SQUARE_PIECES.iter())
                .copied(),
        ),
        Roux::moves_of(&second_block_moves),
        |m: &Roux| !m.0.sb_square_as_one_step,
    );

    let sb_pair = SimpleStep::new(
        "SB Pair",
        sb_square.solved_pieces(),
        Mask::<Cube3x3>::new_from_pieces(
            Roux::FB_PIECES
                .iter()
                .chain(Roux::SB_SQUARE_PIECES.iter())
                .chain(Roux::SB_PIECES.iter())
                .copied(),
        ),
        Roux::moves_of(&second_block_moves),
        |m: &Roux| !m.0.sb_square_as_one_step,
    );

    let sb = SimpleStep::new(
        "SB",
        fb.solved_pieces(),
        Mask::<Cube3x3>::new_from_pieces(
            Roux::FB_PIECES
                .iter()
                .chain(Roux::SB_PIECES.iter())
                .copied(),
        ),
        Roux::moves_of(&second_block_moves),
        |m: &Roux| m.0.sb_square_as_one_step,
    );

    let cmll_after = Mask::<Cube3x3>::new_from_pieces(
        Roux::FB_PIECES
            .iter()
            .chain(Roux::SB_PIECES.iter())
            .chain(Roux::CMLL_PIECES.iter())
            .copied(),
    );
    let cmll = SimpleStep::new(
        "CMLL",
        sb.solved_pieces(),
        cmll_after.clone(),
        Roux::algorithms_with_free_auf(CMLL_ONE_LOOK_ALGS),
        |m: &Roux| m.0.one_look_cmll,
    );

    let cmll_orientation_after = Mask::<Cube3x3>::new(
        Roux::FB_PIECES
            .iter()
            .chain(Roux::SB_PIECES.iter())
            .copied(),
        Roux::FB_PIECES
            .iter()
            .chain(Roux::SB_PIECES.iter())
            .chain(Roux::CMLL_PIECES.iter())
            .copied(),
    );
    let cmll_orientation = SimpleStep::new(
        "CO",
        sb.solved_pieces(),
        cmll_orientation_after.clone(),
        Roux::algorithms_with_free_auf(CMLL_ORIENTATION_ALGS),
        |m: &Roux| !m.0.one_look_cmll,
    );

    let cmll_permutation = SimpleStep::new(
        "CP",
        cmll_orientation_after,
        cmll_after,
        Roux::algorithms_with_free_auf(CMLL_PERMUTATION_ALGS),
        |m: &Roux| !m.0.one_look_cmll,
    );

    let lse = SimpleStep::new(
        "LSE",
        cmll.solved_pieces(),
        Mask::<Cube3x3>::new_from_pieces(
            Roux::FB_PIECES
                .iter()
                .chain(Roux::SB_PIECES.iter())
                .chain(Roux::CMLL_PIECES.iter())
                .chain(Roux::LSE_PIECES.iter())
                .copied(),
        ),
        Roux::moves_of(&[MovablePart::Face(Faces::U), MovablePart::Slice(Slices::M)]),
        |_| true,
    );

    vec![
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
}

#[derive(Clone, Debug)]
pub struct RouxOptions {
    pub sb_square_as_one_step: bool,
    pub fb_as_one_step: bool,
    pub one_look_cmll: bool,
}

impl Default for RouxOptions {
    fn default() -> Self {
        Self {
            sb_square_as_one_step: true,
            fb_as_one_step: true,
            one_look_cmll: true,
        }
    }
}

impl SolveMethod<Cube3x3> for Roux {
    type MethodOptions = RouxOptions;

    fn name(&self) -> String {
        "Roux".to_string()
    }

    fn from_options(options: Self::MethodOptions) -> Self {
        Self(options)
    }

    fn to_options(&self) -> &Self::MethodOptions {
        &self.0
    }

    fn steps(&self) -> Vec<&impl SolveStep<Cube3x3, Self>> {
        STEPS.iter().collect()
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
            Roux::FB_PIECES
                .iter()
                .chain(Roux::SB_SQUARE_PIECES.iter())
                .chain(Roux::SB_PIECES.iter())
                .chain(Roux::CMLL_PIECES.iter())
                .chain(Roux::LSE_PIECES.iter())
                .copied()
                .collect::<HashSet<Pieces3x3>>(),
            HashSet::from_iter(Cube3x3::ALL_PIECES.iter().copied())
        );
    }

    #[test]
    fn cmll_step_leaves_the_cube_with_cmll_solved() {
        let cmll = STEPS.iter().find(|s| s.name() == "CMLL").unwrap();
        let sune = Cube3x3::from_solved("R U R' U R U2 R'").unwrap();
        for scramble in [Cube3x3::from_solved("U2").unwrap(), sune.inverse()] {
            let mut cube = scramble;
            cmll.solve(&mut cube).unwrap();
            assert!(cmll.step_is_solved(&cube), "not solved:\n{cube}");
        }
    }

    #[test]
    fn step_requirements_meet_bound_conditions() {
        for (i, step) in STEPS.iter().enumerate() {
            let needs = step.needs_solved();
            assert!(
                needs == Mask::default()
                    || STEPS
                        .iter()
                        .take(i)
                        .any(|earlier| earlier.solved_pieces() == needs),
                "step {i} ({}) needs pieces that no earlier step solves",
                step.name()
            );
        }
    }
}
