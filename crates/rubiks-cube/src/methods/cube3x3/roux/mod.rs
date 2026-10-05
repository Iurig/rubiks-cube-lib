use std::sync::{Arc, LazyLock};

use crate::{AlgSet, Choose, Cube3x3, Marked, Method, Puzzle, Step, methods::Technique};

use crate::methods::cube3x3::helpers::{algs, parts, search, search_with_free_algs};

const CMLL_ONE_LOOK_ALGS_STR: &str = include_str!("../algsets/cmll/one_look.txt");
const CO_ALGS_TEXT: &str = include_str!("../algsets/cmll/co.txt");
const CP_ALGS_TEXT: &str = include_str!("../algsets/cmll/cp.txt");

/// The Roux method for the 3x3 Rubik's Cube: first block, second block, CMLL, then the last six
/// edges.
///
/// Each stage can be split into smaller steps. Start from `Roux::default()` and pick the
/// split of a stage with its method:
///
/// ```rust
/// use rubiks_cube::{CMLLOptions, FirstBlockOptions, Roux};
///
/// let roux = Roux::default()
///     .first_block(FirstBlockOptions::SquarePair)
///     .cmll(CMLLOptions::TwoLook);
/// ```
#[non_exhaustive]
#[derive(Default, Debug)]
pub struct Roux {
    fb: FirstBlockOptions,
    sb: SecondBlockOptions,
    cmll: CMLLOptions,
    lse: LSEOptions,
}

impl Roux {
    /// Changes the first block options of the current method to [`fb`](FirstBlockOptions).
    #[must_use]
    pub const fn first_block(mut self, fb: FirstBlockOptions) -> Self {
        self.fb = fb;
        self
    }

    /// Changes the second block options of the current method to [`sb`](SecondBlockOptions).
    #[must_use]
    pub const fn second_block(mut self, sb: SecondBlockOptions) -> Self {
        self.sb = sb;
        self
    }

    /// Changes the CMLL options of the current method to [`cmll`](CMLLOptions).
    #[must_use]
    pub const fn cmll(mut self, cmll: CMLLOptions) -> Self {
        self.cmll = cmll;
        self
    }

    /// Changes the LSE options of the current method to [`lse`](LSEOptions).
    #[must_use]
    pub const fn lse(mut self, lse: LSEOptions) -> Self {
        self.lse = lse;
        self
    }
}

/// How [`Roux`] builds the first block, the 1x2x3 block on the left.
#[derive(Clone, Copy, Default, Debug)]
pub enum FirstBlockOptions {
    /// One search for the whole block.
    #[default]
    OneLook,
    /// A 1x2x2 square, front or back, then the pair that finishes the block.
    SquarePair,
    /// The DL edge, then a square, then the pair.
    EdgePairPair,
}

/// How [`Roux`] builds the second block, the 1x2x3 block on the right.
#[derive(Clone, Copy, Default, Debug)]
pub enum SecondBlockOptions {
    /// One search for the whole block.
    OneLook,
    /// A 1x2x2 square, front or back, then the pair that finishes the block.
    SquarePair,
    /// The DR edge, then a square, then the pair.
    #[default]
    EdgePairPair,
}

/// How [`Roux`] solves the U-layer corners.
#[derive(Clone, Copy, Default, Debug)]
pub enum CMLLOptions {
    /// One algorithm from the full CMLL set.
    #[default]
    OneLook,
    /// One algorithm to orient the corners, then one to permute them.
    TwoLook,
}

/// How [`Roux`] solves the last six edges.
#[derive(Clone, Copy, Default, Debug)]
pub enum LSEOptions {
    /// One search for all six edges with `U` and `M`.
    #[default]
    EOLR,
    //EO,
}

static FB_MOVES: LazyLock<AlgSet<Cube3x3>> = LazyLock::new(|| parts("F U R L D B M r"));
static SB_MOVES: LazyLock<AlgSet<Cube3x3>> = LazyLock::new(|| parts("U R M r"));
static FREE_AUF: LazyLock<AlgSet<Cube3x3>> = LazyLock::new(|| parts("U"));
static CMLL_ALGS: LazyLock<AlgSet<Cube3x3>> = LazyLock::new(|| algs(CMLL_ONE_LOOK_ALGS_STR));
static LSE_MOVES: LazyLock<AlgSet<Cube3x3>> = LazyLock::new(|| parts("U M"));
static LSE_KEEPING_CORNERS: LazyLock<AlgSet<Cube3x3>> = LazyLock::new(|| algs("M\nU M U'"));

impl Method<Cube3x3> for Roux {
    fn to_technique(&self) -> Technique<Cube3x3> {
        let fb_tech = match self.fb {
            FirstBlockOptions::OneLook => chain_steps!(
                moves: &FB_MOVES, goal: &SB_MOVES,
                "FB",
            ),
            FirstBlockOptions::SquarePair => chain_steps!(
                moves: &FB_MOVES, goal: &SB_MOVES,
                "FB Square" => (parts("B"), parts("F")),
                "FB Pair",
            ),
            FirstBlockOptions::EdgePairPair => chain_steps!(
                moves: &FB_MOVES, goal: &SB_MOVES,
                "DL" => (parts("F B")),
                "FB Square" => (parts("B"), parts("F")),
                "FB Pair",
            ),
        };

        let sb_tech = match self.sb {
            SecondBlockOptions::OneLook => chain_steps!(
                moves: &SB_MOVES, goal: &CMLL_ALGS,
                "SB"
            ),
            SecondBlockOptions::SquarePair => chain_steps!(
                moves: &SB_MOVES, goal: &CMLL_ALGS,
                "SB Square" => (algs("r U r'"), algs("r' U r")),
                "SB Pair"
            ),
            SecondBlockOptions::EdgePairPair => chain_steps!(
                moves: &SB_MOVES, goal: &CMLL_ALGS,
                "DR" => (algs("r U r'\n r' U r")),
                "SB Square" => (algs("r U r'"), algs("r' U r")),
                "SB Pair",
            ),
        };

        let after_sb = Marked::from_algset(&CMLL_ALGS.combined_with(&LSE_MOVES));
        let after_cmll = Marked::from_algset(&LSE_KEEPING_CORNERS);

        let cmll_tech: Vec<Arc<dyn Step<Cube3x3>>> = match self.cmll {
            CMLLOptions::OneLook => vec![search_with_free_algs(
                "CMLL",
                &after_sb,
                &after_cmll,
                &CMLL_ALGS,
                &FREE_AUF,
            )],
            CMLLOptions::TwoLook => {
                let cp_algs = algs(CP_ALGS_TEXT);
                // Neither the permutation algorithms nor the AUF twist a corner, so the corners
                // stay in the goal through their orientation only.
                let after_co = Marked::from_algset(
                    &cp_algs
                        .combined_with(&FREE_AUF)
                        .combined_with(&LSE_KEEPING_CORNERS),
                );
                vec![
                    search_with_free_algs(
                        "CO",
                        &after_sb,
                        &after_co,
                        &algs(CO_ALGS_TEXT),
                        &FREE_AUF,
                    ),
                    search_with_free_algs("CP", &after_co, &after_cmll, &cp_algs, &FREE_AUF),
                ]
            }
        };

        let lse_tech: Vec<Arc<dyn Step<Cube3x3>>> = match self.lse {
            LSEOptions::EOLR => vec![search(
                "LSE",
                &after_cmll,
                &Marked::new_from_pieces(Cube3x3::ALL_PIECES.iter().copied()),
                &LSE_MOVES,
            )],
            //LSEOptions::EO => (),
        };

        [fb_tech, sb_tech, cmll_tech, lse_tech]
            .into_iter()
            .flatten()
            .collect()
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
        let cmll = Roux::default()
            .to_technique()
            .steps
            .into_iter()
            .find(|s| s.name() == "CMLL")
            .unwrap();
        let sune = Cube3x3::from_solved("R U R' U R U2 R'").unwrap();
        for scramble in [Cube3x3::from_solved("U2").unwrap(), sune.inverse()] {
            let mut cube = scramble;
            cmll.solve(&mut cube).unwrap();
            assert!(cmll.is_done(&cube), "not solved:\n{cube}");
        }
    }

    /// Every `to_technique` call builds new step objects, so memos are shared through
    /// `MEMOS`, keyed by `after ∪ before` and the movesets. A step built again with the same
    /// goals shares its memo, and so does the one-look block with the pair that finishes it,
    /// because the pair's `before` lies inside its `after`.
    #[test]
    fn rebuilt_roux_steps_share_their_memos() {
        let fb_start = Marked::from_algset(&FB_MOVES);
        let fb_goal = Marked::from_algset(&SB_MOVES);
        let back_square = Marked::from_algset(&SB_MOVES.combined_with(&parts("F")));

        let fb = search("FB", &fb_start, &fb_goal, &FB_MOVES);
        let fb_again = search("FB", &fb_start, &fb_goal, &FB_MOVES);
        let fb_pair = search("FB Pair", &back_square, &fb_goal, &FB_MOVES);
        let fb_square = search("FB Square", &fb_start, &back_square, &FB_MOVES);

        assert!(fb.shares_memo_with(&fb_again));
        assert!(fb.shares_memo_with(&fb_pair));
        assert!(!fb.shares_memo_with(&fb_square));
    }

    /// R, U, and F turns never touch the L center, DBL, BL, or DL, so the back square stays
    /// solved and costs nothing, while F breaks the front square.
    #[test]
    fn split_fb_builds_the_back_square_when_it_is_cheaper() -> Result<(), Box<dyn Error>> {
        let roux = Roux {
            fb: FirstBlockOptions::SquarePair,
            ..Roux::default()
        };
        let mut cube = Cube3x3::from_solved("F R U")?;

        let solution = roux.solve(&mut cube)?;

        let fb_segment = solution
            .iter()
            .next()
            .ok_or("the solution has no segments")?;
        assert_eq!(fb_segment.name, "FB Square", "{solution}");
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
        let roux = Roux {
            sb: SecondBlockOptions::SquarePair,
            ..Roux::default()
        };
        let mut cube = Cube3x3::from_solved("R U R'")?;

        let solution = roux.solve(&mut cube)?;

        let sb_segment = solution
            .iter()
            .find(|segment| segment.name.starts_with("SB"))
            .ok_or("the solution has no SB segment")?;
        assert_eq!(sb_segment.name, "SB Square", "{solution}");
        assert!(
            sb_segment.moves.is_empty(),
            "the back square was already solved:\n{solution}"
        );
        Ok(())
    }

    /// Every combination of the options.
    fn every_options() -> impl Iterator<Item = Roux> {
        use CMLLOptions as C;
        use FirstBlockOptions as F;
        use SecondBlockOptions as S;
        [F::OneLook, F::SquarePair, F::EdgePairPair]
            .into_iter()
            .flat_map(|fb| {
                [S::OneLook, S::SquarePair, S::EdgePairPair]
                    .into_iter()
                    .flat_map(move |sb| {
                        [C::OneLook, C::TwoLook].into_iter().map(move |cmll| Roux {
                            fb,
                            sb,
                            cmll,
                            lse: LSEOptions::EOLR,
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

        for step in every_options().flat_map(|options| options.to_technique().steps) {
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
            let names: Vec<String> = options
                .to_technique()
                .steps
                .iter()
                .map(|step| step.name().to_string())
                .collect();

            let mut expected: Vec<&str> = Vec::new();
            expected.extend_from_slice(match options.fb {
                FirstBlockOptions::OneLook => &["FB"][..],
                FirstBlockOptions::SquarePair => &["FB Square", "FB Pair"],
                FirstBlockOptions::EdgePairPair => &["DL", "FB Square", "FB Pair"],
            });
            expected.extend_from_slice(match options.sb {
                SecondBlockOptions::OneLook => &["SB"][..],
                SecondBlockOptions::SquarePair => &["SB Square", "SB Pair"],
                SecondBlockOptions::EdgePairPair => &["DR", "SB Square", "SB Pair"],
            });
            expected.extend_from_slice(match options.cmll {
                CMLLOptions::OneLook => &["CMLL"][..],
                CMLLOptions::TwoLook => &["CO", "CP"],
            });
            expected.push("LSE");

            assert_eq!(names, expected, "{options:?}");
        }
    }
}
