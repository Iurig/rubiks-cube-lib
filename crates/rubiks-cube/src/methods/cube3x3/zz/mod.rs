#[cfg(test)]
use enum_iterator::{Sequence, all};
use std::sync::{Arc, LazyLock};

use crate::methods::cube3x3::helpers::{LastLayer, algs, parts};
use crate::{AlgSet, Choose, Cube3x3, Marked, Method, Step, Technique};

/// The ZZ method for the 3x3 Rubik's Cube: EO line, first two layers, then last layer.
#[derive(Debug, Default)]
#[cfg_attr(test, derive(Sequence))]
pub struct ZZ {
    eoline: EOLineOptions,
    f2l: F2LOptions,
    ll: LLOptions,
}

#[non_exhaustive]
#[derive(Debug, Default, Clone)]
#[cfg_attr(test, derive(Sequence))]
pub enum EOLineOptions {
    #[default]
    Full,
}
#[non_exhaustive]
#[derive(Debug, Default, Clone)]
#[cfg_attr(test, derive(Sequence))]
pub enum F2LOptions {
    #[default]
    Full,
}
#[non_exhaustive]
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Sequence))]
pub enum LLOptions {
    TwoLook {
        orientation: OrientationOptions,
        permutation: PermutationOptions,
    },
    OneLook,
}
#[non_exhaustive]
#[derive(Debug, Default, Clone)]
#[cfg_attr(test, derive(Sequence))]
pub enum OrientationOptions {
    #[default]
    OCLL,
    COLL,
}
#[non_exhaustive]
#[derive(Debug, Default, Clone)]
#[cfg_attr(test, derive(Sequence))]
pub enum PermutationOptions {
    #[default]
    OneLook,
    TwoLook,
}

impl Default for LLOptions {
    fn default() -> Self {
        Self::TwoLook {
            orientation: OrientationOptions::default(),
            permutation: PermutationOptions::default(),
        }
    }
}

impl Method<Cube3x3> for ZZ {
    fn to_technique(&self) -> Technique<Cube3x3> {
        let eoline_tech = match self.eoline {
            EOLineOptions::Full => {
                chain_steps!(moves: &EO_LINE_MOVES, goal: &F2L_MOVES, "EO Line")
            }
        };

        let f2l_tech = match self.f2l {
            F2LOptions::Full => {
                chain_steps!(moves: &F2L_MOVES, goal: &OCLL_ALGS, "F2L")
            }
        };

        let ll_tech = match self.ll {
            LLOptions::TwoLook {
                orientation: ref oll,
                permutation: ref pll,
            } => {
                let oll_tech = match oll {
                    OrientationOptions::OCLL => LastLayer::ocll(OCLL_ALGS.clone()).to_steps(),
                    OrientationOptions::COLL => LastLayer::coll(COLL_ALGS.clone()).to_steps(),
                };
                let pll_tech = match pll {
                    PermutationOptions::OneLook => LastLayer::pll(PLL_ALGS.clone()).to_steps(),
                    PermutationOptions::TwoLook => {
                        let corners = LastLayer::cpll(PLL_CORNER_ALGS.clone()).to_steps();
                        let edges = LastLayer::epll(PLL_EDGE_ALGS.clone()).to_steps();
                        [corners, edges].into_iter().flatten().collect()
                    }
                };
                [oll_tech, pll_tech].into_iter().flatten().collect()
            }
            LLOptions::OneLook => LastLayer::zbll(ZBLL_ALGS.clone()).to_steps(),
        };

        [eoline_tech, f2l_tech, ll_tech]
            .into_iter()
            .flatten()
            .collect()
    }
}

static EO_LINE_MOVES: LazyLock<AlgSet<Cube3x3>> = LazyLock::new(|| parts("F B U R L D "));
static F2L_MOVES: LazyLock<AlgSet<Cube3x3>> = LazyLock::new(|| parts("U R L "));
static OCLL_ALGS: LazyLock<AlgSet<Cube3x3>> =
    LazyLock::new(|| algs(include_str!("oll/ocll.txt")).or_skip());
static COLL_ALGS: LazyLock<AlgSet<Cube3x3>> =
    LazyLock::new(|| algs(include_str!("oll/coll.txt")).or_skip());
static PLL_CORNER_ALGS: LazyLock<AlgSet<Cube3x3>> =
    LazyLock::new(|| algs(include_str!("pll/corner.txt")).or_skip());
static PLL_EDGE_ALGS: LazyLock<AlgSet<Cube3x3>> =
    LazyLock::new(|| algs(include_str!("pll/edge.txt")).or_skip());
static PLL_ALGS: LazyLock<AlgSet<Cube3x3>> =
    LazyLock::new(|| algs(include_str!("pll/one_look.txt")).or_skip());
static ZBLL_ALGS: LazyLock<AlgSet<Cube3x3>> = LazyLock::new(|| {
    algs(concat!(
        include_str!("../algsets/zbll/t.txt"),
        include_str!("../algsets/zbll/u.txt"),
        include_str!("../algsets/zbll/l.txt"),
        include_str!("../algsets/zbll/pi.txt"),
        include_str!("../algsets/zbll/h.txt"),
        include_str!("../algsets/zbll/sune.txt"),
        include_str!("../algsets/zbll/anti_sune.txt"),
    ))
    .or_skip()
});

#[cfg(test)]
mod test {
    #![expect(clippy::panic_in_result_fn, reason = "standard procedure in our tests")]
    use std::error::Error;

    use crate::Puzzle;

    use super::*;

    #[test]
    fn zz_solves_the_cube() -> Result<(), Box<dyn Error>> {
        let scramble = Cube3x3::scramble_with_seed(2).unwrap();
        let mut cube = Cube3x3::default().apply(&scramble);

        println!("{scramble}");
        for partial_solution in ZZ::default().solve_steps(&mut cube) {
            println!("{}", partial_solution?);
        }
        assert!(cube.is_solved());
        Ok(())
    }

    #[test]
    fn zz_works_with_all_options() -> Result<(), Box<dyn Error>> {
        let settings = all::<ZZ>().collect::<Vec<_>>();
        let scrambles = (0..u64::try_from(settings.len())?)
            .map(|seed| Cube3x3::scramble_with_seed(seed).unwrap())
            .collect::<Vec<_>>();
        for (scramble, setting) in scrambles.iter().zip(settings) {
            let mut cube = Cube3x3::default().apply(scramble);

            dbg!(&setting);

            println!("{scramble}\n");
            for partial_solution in setting.solve_steps(&mut cube) {
                print!("{}", partial_solution?);
            }
            assert!(cube.is_solved());
        }
        Ok(())
    }
}
