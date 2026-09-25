use std::sync::{Arc, LazyLock};

use crate::{
    Choose, Cube3x3, Mask, Method, Puzzle, SearchStep, Step,
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

const SB_FRONT_SQUARE_PIECES: [Pieces3x3; 4] = [
    Pieces3x3::Center(crate::Center::R),
    Pieces3x3::Edge(crate::Edge::Dr),
    Pieces3x3::Corner(crate::Corner::Dfr),
    Pieces3x3::Edge(crate::Edge::Fr),
];

const SB_BACK_SQUARE_PIECES: [Pieces3x3; 4] = [
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

    let fb_front_square = Arc::new(SearchStep::new(
        "FB Front Square",
        Mask::<Cube3x3>::default(),
        Mask::<Cube3x3>::new_from_pieces(FB_FRONT_SQUARE_PIECES),
        moves_of(&all_parts),
    ));

    let fb_back_square = Arc::new(SearchStep::new(
        "FB Back Square",
        Mask::<Cube3x3>::default(),
        Mask::<Cube3x3>::new_from_pieces(FB_BACK_SQUARE_PIECES),
        moves_of(&all_parts),
    ));

    // Finishes the block from the back square, so the pair it builds is the front one.
    let fb_front_pair = Arc::new(SearchStep::new(
        "FB Front Pair",
        Mask::<Cube3x3>::new_from_pieces(FB_BACK_SQUARE_PIECES),
        Mask::<Cube3x3>::new_from_pieces(FB_PIECES),
        moves_of(&all_parts),
    ));

    // Finishes the block from the front square, so the pair it builds is the back one.
    let fb_back_pair = Arc::new(SearchStep::new(
        "FB Back Pair",
        Mask::<Cube3x3>::new_from_pieces(FB_FRONT_SQUARE_PIECES),
        Mask::<Cube3x3>::new_from_pieces(FB_PIECES),
        moves_of(&all_parts),
    ));

    let fb = Arc::new(SearchStep::new(
        "FB",
        Mask::<Cube3x3>::new_from_pieces([]),
        Mask::<Cube3x3>::new_from_pieces(FB_PIECES),
        moves_of(&all_parts),
    ));

    let second_block_moves = [
        MovablePart::Face(Faces::U),
        MovablePart::Face(Faces::R),
        MovablePart::Slice(Slices::M),
        MovablePart::Wide(Faces::R),
    ];

    let sb_after =
        Mask::<Cube3x3>::new_from_pieces(FB_PIECES.iter().chain(SB_PIECES.iter()).copied());

    let sb_front_square = Arc::new(SearchStep::new(
        "SB Front Square",
        fb.after(),
        Mask::<Cube3x3>::new_from_pieces(
            FB_PIECES
                .iter()
                .chain(SB_FRONT_SQUARE_PIECES.iter())
                .copied(),
        ),
        moves_of(&second_block_moves),
    ));

    let sb_back_square = Arc::new(SearchStep::new(
        "SB Back Square",
        fb.after(),
        Mask::<Cube3x3>::new_from_pieces(
            FB_PIECES
                .iter()
                .chain(SB_BACK_SQUARE_PIECES.iter())
                .copied(),
        ),
        moves_of(&second_block_moves),
    ));

    // Finishes the block from the back square, so the pair it builds is the front one.
    let sb_front_pair = Arc::new(SearchStep::new(
        "SB Front Pair",
        sb_back_square.after(),
        sb_after.clone(),
        moves_of(&second_block_moves),
    ));

    // Finishes the block from the front square, so the pair it builds is the back one.
    let sb_back_pair = Arc::new(SearchStep::new(
        "SB Back Pair",
        sb_front_square.after(),
        sb_after.clone(),
        moves_of(&second_block_moves),
    ));

    let sb = Arc::new(SearchStep::new(
        "SB",
        fb.after(),
        sb_after,
        moves_of(&second_block_moves),
    ));

    let cmll_after = Mask::<Cube3x3>::new_from_pieces(
        FB_PIECES
            .iter()
            .chain(SB_PIECES.iter())
            .chain(CMLL_PIECES.iter())
            .copied(),
    );
    let cmll = Arc::new(SearchStep::new(
        "CMLL",
        sb.after(),
        cmll_after.clone(),
        algorithms_with_free_auf(CMLL_ONE_LOOK_ALGS),
    ));

    let cmll_orientation_after = Mask::<Cube3x3>::new(
        FB_PIECES.iter().chain(SB_PIECES.iter()).copied(),
        FB_PIECES
            .iter()
            .chain(SB_PIECES.iter())
            .chain(CMLL_PIECES.iter())
            .copied(),
    );
    let cmll_orientation = Arc::new(SearchStep::new(
        "CO",
        sb.after(),
        cmll_orientation_after.clone(),
        algorithms_with_free_auf(CMLL_ORIENTATION_ALGS),
    ));

    let cmll_permutation = Arc::new(SearchStep::new(
        "CP",
        cmll_orientation_after,
        cmll_after,
        algorithms_with_free_auf(CMLL_PERMUTATION_ALGS),
    ));

    let lse = Arc::new(SearchStep::new(
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
    ));

    let steps: Vec<Arc<dyn Step<Cube3x3>>> = vec![
        Arc::new(Choose::named(
            "FB Square",
            vec![fb_front_square, fb_back_square],
        )),
        Arc::new(Choose::named("FB Pair", vec![fb_front_pair, fb_back_pair])),
        fb,
        Arc::new(Choose::named(
            "SB Square",
            vec![sb_front_square, sb_back_square],
        )),
        Arc::new(Choose::named("SB Pair", vec![sb_front_pair, sb_back_pair])),
        sb,
        cmll,
        cmll_orientation,
        cmll_permutation,
        lse,
    ];
    steps
});

/// Which steps [`Method::roux`] uses. All three switches are on by default.
#[derive(Clone, Debug, Copy)]
pub struct RouxOptions {
    /// Build the second block in one search. When off, build a square, front or back, whichever
    /// takes fewer moves, and then the pair that square leaves.
    pub sb_as_one_step: bool,
    /// Build the first block in one search. When off, build a square and then a pair, as for
    /// the second block.
    pub fb_as_one_step: bool,
    /// Solve the last-layer corners with one algorithm (CMLL). When off, orient them first
    /// (`CO`) and then permute them (`CP`).
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
    /// The Roux method, with the steps `options` selects.
    ///
    /// The first block may use any move. The second block uses `U`, `R`, `M`, and `r`. The
    /// last-layer corners use algorithms from a fixed list, each costing one, and `U` turns,
    /// which cost nothing. The last six edges use `U` and `M`.
    ///
    /// Every method this returns shares the same built steps, so the search memos that one
    /// solve grows speed up every later solve, whatever the options.
    #[must_use]
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
    #![expect(
        clippy::panic_in_result_fn,
        reason = "`?` reports setup failures; `assert!` reports the property under test failing"
    )]

    use std::{collections::HashSet, error::Error};

    use super::*;
    use crate::{Cube3x3, Inv, Puzzle};

    #[test]
    fn all_step_pieces_are_all_cube_pieces() {
        assert_eq!(
            FB_PIECES
                .iter()
                .chain(SB_FRONT_SQUARE_PIECES.iter())
                .chain(SB_BACK_SQUARE_PIECES.iter())
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

    /// A step owns its memo, so a method that holds the same step object as the static shares
    /// the static's memo. Rebuilding steps per method would start every method from empty memos.
    #[test]
    fn every_roux_method_shares_the_static_steps_and_their_memos() {
        for fb_as_one_step in [false, true] {
            for sb_as_one_step in [false, true] {
                for one_look_cmll in [false, true] {
                    let options = RouxOptions {
                        sb_as_one_step,
                        fb_as_one_step,
                        one_look_cmll,
                    };
                    for step in Method::roux(options).steps() {
                        assert!(
                            ALL_ROUX_STEPS
                                .iter()
                                .any(|shared| Arc::ptr_eq(shared, &step)),
                            "{options:?}: step {} is not the shared one",
                            step.name()
                        );
                    }
                }
            }
        }
    }

    /// R, U, and F turns never touch the L center, DBL, BL, or DL, so the back square stays
    /// solved and costs nothing, while F breaks the front square.
    #[test]
    fn split_fb_builds_the_back_square_when_it_is_cheaper() -> Result<(), Box<dyn Error>> {
        let roux = Method::roux(RouxOptions {
            fb_as_one_step: false,
            ..RouxOptions::default()
        });
        let mut cube = Cube3x3::from_solved("F R U")?;

        let solution = roux.solve(&mut cube)?;

        let (moves, name) = solution
            .iter()
            .next()
            .ok_or("the solution has no segments")?;
        assert_eq!(name, "FB Back Square", "{solution}");
        assert!(
            moves.is_empty(),
            "the back square was already solved:\n{solution}"
        );
        Ok(())
    }

    /// `R U R'` displaces the FR edge and DFR corner but returns DR, BR, and DBR home, and
    /// neither R nor U touches the first block. So the second block's back square is solved
    /// and its front square is not.
    #[test]
    fn split_sb_builds_the_back_square_when_it_is_cheaper() -> Result<(), Box<dyn Error>> {
        let roux = Method::roux(RouxOptions {
            sb_as_one_step: false,
            ..RouxOptions::default()
        });
        let mut cube = Cube3x3::from_solved("R U R'")?;

        let solution = roux.solve(&mut cube)?;

        let (moves, name) = solution
            .iter()
            .find(|(_, name)| name.starts_with("SB"))
            .ok_or("the solution has no SB segment")?;
        assert_eq!(name, "SB Back Square", "{solution}");
        assert!(
            moves.is_empty(),
            "the back square was already solved:\n{solution}"
        );
        Ok(())
    }
}
