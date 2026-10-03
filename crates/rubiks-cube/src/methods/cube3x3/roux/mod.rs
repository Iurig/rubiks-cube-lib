use std::sync::{Arc, LazyLock};

use crate::{AlgSet, Choose, Cube3x3, Marked, Method, Puzzle, SearchStep, Step};

/// One-look CMLL algorithms, one per line.
const CMLL_ONE_LOOK_ALGS: &str = include_str!("cmll/one_look.txt");
/// Algorithms that orient the U-layer corners, one per line.
const CMLL_ORIENTATION_ALGS: &str = include_str!("cmll/co.txt");
/// Algorithms that permute oriented U-layer corners, one per line.
const CMLL_PERMUTATION_ALGS: &str = include_str!("cmll/cp.txt");

/// The Roux steps in solving order, built once on first use.
///
/// Each step's goal is the pieces that every move and algorithm of the later steps leaves
/// solved, so the goals are read from the movesets instead of listed piece by piece.
static ALL_ROUX_STEPS: LazyLock<Vec<Arc<dyn Step<Cube3x3>>>> = LazyLock::new(|| {
    let fb_moves =
        AlgSet::from_parts("F U R L D B M r").expect("hand-written part lists should always parse");

    let sb_moves =
        AlgSet::from_parts("U R M r").expect("hand-written part lists should always parse");

    let free_auf = AlgSet::from_moves("U U2 U'").expect("the AUF always parses");

    let cmll_algs = AlgSet::from_algs_in_str(CMLL_ONE_LOOK_ALGS)
        .expect("manually curated algorithm sets should always parse");

    let cmll_orientation_algs = AlgSet::from_algs_in_str(CMLL_ORIENTATION_ALGS)
        .expect("manually curated algorithm sets should always parse");

    let cmll_permutation_algs = AlgSet::from_algs_in_str(CMLL_PERMUTATION_ALGS)
        .expect("manually curated algorithm sets should always parse");

    let lse_moves = AlgSet::from_parts("U M").expect("hand-written part lists should always parse");

    // `U` moves the corners, so the LSE moveset alone would leave them out of the CMLL goal.
    // These sequences move the same edges and centers as `U M` and bring every corner home.
    let lse_keeping_corners = AlgSet::from_algs_in_str("M\nU M U'")
        .expect("hand-written algorithm sets should always parse");

    let fb_front_square = Arc::new(SearchStep::new(
        "FB Front Square",
        Marked::<Cube3x3>::default(),
        Marked::<Cube3x3>::from_algset(&sb_moves.combined_with(
            &AlgSet::from_moves("B").expect("manually typed pieces should parse correctly"),
        )),
        fb_moves.clone(),
    ));

    let fb_back_square = Arc::new(SearchStep::new(
        "FB Back Square",
        Marked::<Cube3x3>::default(),
        Marked::<Cube3x3>::from_algset(&sb_moves.combined_with(
            &AlgSet::from_moves("F").expect("manually typed pieces should parse correctly"),
        )),
        fb_moves.clone(),
    ));

    let fb_after = Marked::<Cube3x3>::from_algset(&sb_moves);

    // Finishes the block from the back square, so the pair it builds is the front one.
    let fb_front_pair = Arc::new(SearchStep::new(
        "FB Front Pair",
        fb_back_square.after(),
        fb_after.clone(),
        fb_moves.clone(),
    ));

    // Finishes the block from the front square, so the pair it builds is the back one.
    let fb_back_pair = Arc::new(SearchStep::new(
        "FB Back Pair",
        fb_front_square.after(),
        fb_after.clone(),
        fb_moves.clone(),
    ));

    let fb = Arc::new(SearchStep::new(
        "FB",
        Marked::<Cube3x3>::new_from_pieces([]),
        fb_after,
        fb_moves,
    ));

    let after_sb_moves = cmll_algs.combined_with(&lse_moves);

    let sb_after = Marked::<Cube3x3>::from_algset(&after_sb_moves);

    // `R' U R` displaces the back pair and returns the front square home.
    let sb_front_square = Arc::new(SearchStep::new(
        "SB Front Square",
        fb.after(),
        Marked::<Cube3x3>::from_algset(
            &after_sb_moves.combined_with(
                &AlgSet::from_algs_in_str("R' U R")
                    .expect("hand-written algorithm sets should always parse"),
            ),
        ),
        sb_moves.clone(),
    ));

    // `R U R'` displaces the front pair and returns the back square home.
    let sb_back_square = Arc::new(SearchStep::new(
        "SB Back Square",
        fb.after(),
        Marked::<Cube3x3>::from_algset(
            &after_sb_moves.combined_with(
                &AlgSet::from_algs_in_str("R U R'")
                    .expect("hand-written algorithm sets should always parse"),
            ),
        ),
        sb_moves.clone(),
    ));

    // Finishes the block from the back square, so the pair it builds is the front one.
    let sb_front_pair = Arc::new(SearchStep::new(
        "SB Front Pair",
        sb_back_square.after(),
        sb_after.clone(),
        sb_moves.clone(),
    ));

    // Finishes the block from the front square, so the pair it builds is the back one.
    let sb_back_pair = Arc::new(SearchStep::new(
        "SB Back Pair",
        sb_front_square.after(),
        sb_after.clone(),
        sb_moves.clone(),
    ));

    let sb = Arc::new(SearchStep::new("SB", fb.after(), sb_after, sb_moves));

    let cmll_after = Marked::<Cube3x3>::from_algset(&lse_keeping_corners);
    let cmll = Arc::new(SearchStep::new_with_free_algs(
        "CMLL",
        sb.after(),
        cmll_after.clone(),
        cmll_algs,
        free_auf.clone(),
    ));

    // Neither the permutation algorithms nor the AUF twist a corner, so the corners stay in
    // the goal through their orientation only.
    let cmll_orientation_after = Marked::<Cube3x3>::from_algset(
        &cmll_permutation_algs
            .combined_with(&free_auf)
            .combined_with(&lse_keeping_corners),
    );
    let cmll_orientation = Arc::new(SearchStep::new_with_free_algs(
        "CO",
        sb.after(),
        cmll_orientation_after.clone(),
        cmll_orientation_algs,
        free_auf.clone(),
    ));

    let cmll_permutation = Arc::new(SearchStep::new_with_free_algs(
        "CP",
        cmll_orientation_after,
        cmll_after,
        cmll_permutation_algs,
        free_auf,
    ));

    let lse = Arc::new(SearchStep::new(
        "LSE",
        cmll.after(),
        Marked::<Cube3x3>::new_from_pieces(Cube3x3::ALL_PIECES.iter().copied()),
        lse_moves,
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
#[derive(Clone, Copy, Debug)]
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

    use std::error::Error;

    use super::*;
    use crate::{Cube3x3, Inv};

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

        let fb_segment = solution
            .iter()
            .next()
            .ok_or("the solution has no segments")?;
        assert_eq!(fb_segment.name, "FB Back Square", "{solution}");
        assert!(
            fb_segment.moves.is_empty(),
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

        let sb_segment = solution
            .iter()
            .find(|segment| segment.name.starts_with("SB"))
            .ok_or("the solution has no SB segment")?;
        assert_eq!(sb_segment.name, "SB Back Square", "{solution}");
        assert!(
            sb_segment.moves.is_empty(),
            "the back square was already solved:\n{solution}"
        );
        Ok(())
    }

    /// Every combination of the three switches.
    fn every_options() -> impl Iterator<Item = RouxOptions> {
        [false, true].into_iter().flat_map(|fb_as_one_step| {
            [false, true].into_iter().flat_map(move |sb_as_one_step| {
                [false, true]
                    .into_iter()
                    .map(move |one_look_cmll| RouxOptions {
                        sb_as_one_step,
                        fb_as_one_step,
                        one_look_cmll,
                    })
            })
        })
    }

    /// `Method` relies on `is_done` to catch a step that returns without meeting its goal, so
    /// every step must tell its goal apart from a cube that misses it.
    #[test]
    fn every_roux_step_is_done_on_a_solved_cube_and_not_on_a_scrambled_one()
    -> Result<(), Box<dyn Error>> {
        let scrambled = Cube3x3::from_solved("R U' F2 L D' B R2 U F' L2 D B' U2 R'")?;

        for step in ALL_ROUX_STEPS.iter() {
            assert!(
                step.is_done(&Cube3x3::default()),
                "{} is not done on a solved cube",
                step.name()
            );
            assert!(
                !step.is_done(&scrambled),
                "{} is done on a scrambled cube:\n{scrambled}",
                step.name()
            );
        }
        Ok(())
    }

    /// Each switch picks its own steps, independently of the others, and the steps stay in
    /// solving order.
    #[test]
    fn options_pick_the_steps_in_solving_order() {
        for options in every_options() {
            let names: Vec<String> = Method::roux(options)
                .steps()
                .map(|step| step.name().to_string())
                .collect();

            let mut expected: Vec<&str> = Vec::new();
            expected.extend_from_slice(if options.fb_as_one_step {
                &["FB"]
            } else {
                &["FB Square", "FB Pair"]
            });
            expected.extend_from_slice(if options.sb_as_one_step {
                &["SB"]
            } else {
                &["SB Square", "SB Pair"]
            });
            expected.extend_from_slice(if options.one_look_cmll {
                &["CMLL"]
            } else {
                &["CO", "CP"]
            });
            expected.push("LSE");

            assert_eq!(names, expected, "{options:?}");
        }
    }
}
