#[cfg(test)]
use enum_iterator::{Sequence, all};
use std::sync::{Arc, LazyLock};

use crate::methods::cube3x3::helpers::{BlockGoal, LastLayer, algs, parts, split_by_blocks};

use crate::{AlgSet, Choose, Cube3x3, Marked, Method, Step, Technique};

/// The ZZ method for the 3x3 Rubik's Cube: EO line, first two layers, then last layer.
#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(test, derive(Sequence))]
pub struct ZZ {
    eoline: EoLineOptions,
    f2l: F2LOptions,
    ll: LLOptions,
}

/// The ZZA variant of the ZZ method. Uses ZBLL on the last layer.
pub const ZZA: ZZ = ZZ {
    eoline: EoLineOptions,
    f2l: F2LOptions,
    ll: LLOptions::OneLook,
};

/// How [`ZZ`] builds its first step, the `EOLine`.
#[non_exhaustive]
#[derive(Debug, Clone, Default, Copy)]
#[cfg_attr(test, derive(Sequence))]
pub struct EoLineOptions;
/// How [`ZZ`] builds its F2L.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(test, derive(Sequence))]
pub struct F2LOptions;
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(Sequence))]
pub enum LLOptions {
    TwoLook {
        orientation: OrientationOptions,
        permutation: PermutationOptions,
    },
    OneLook,
}
/// The first step of a [`two-look`](LLOptions::TwoLook) ZZ last layer.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(test, derive(Sequence))]
pub enum OrientationOptions {
    /// Orientation of the Corners of the Last Layer.
    #[default]
    Ocll,
    /// Orientation and permutation of the Corners of the Last Layer.
    Coll,
}
/// The second step of a [`two-look`](LLOptions::TwoLook) ZZ last layer.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(test, derive(Sequence))]
pub enum PermutationOptions {
    /// Standard PLL
    #[default]
    OneLook,
    /// Two-look PLL
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
            EoLineOptions => chain_steps!(moves: &EO_LINE_MOVES, goal: &F2L_MOVES, "EO Line"),
        };

        let f2l_tech = match self.f2l {
            F2LOptions => vec![Arc::from(split_by_blocks(
                "F2L",
                Marked::from_algset(&F2L_MOVES),
                F2L_BLOCKS.iter(),
                F2L_MOVES.clone(),
            )) as Arc<dyn Step<Cube3x3>>],
        };

        let ll_tech = match self.ll {
            LLOptions::TwoLook {
                orientation: oll,
                permutation: pll,
            } => {
                let oll_tech = match oll {
                    OrientationOptions::Ocll => LastLayer::ocll(OCLL_ALGS.clone()).to_steps(),
                    OrientationOptions::Coll => LastLayer::coll(COLL_ALGS.clone()).to_steps(),
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
static F2L_BLOCKS: LazyLock<[BlockGoal; 2]> = LazyLock::new(|| {
    let f2l = Marked::from_algset(&OCLL_ALGS);
    ["R U", "L U"].map(|other_side| {
        BlockGoal::new(&f2l, &Marked::from_algset(&parts(other_side)), &F2L_MOVES)
    })
});
static OCLL_ALGS: LazyLock<AlgSet<Cube3x3>> =
    LazyLock::new(|| algs(include_str!("algsets/oll/ocll.txt")).or_skip());
static COLL_ALGS: LazyLock<AlgSet<Cube3x3>> =
    LazyLock::new(|| algs(include_str!("algsets/oll/coll.txt")).or_skip());
static PLL_CORNER_ALGS: LazyLock<AlgSet<Cube3x3>> =
    LazyLock::new(|| algs(include_str!("algsets/pll/corner.txt")).or_skip());
static PLL_EDGE_ALGS: LazyLock<AlgSet<Cube3x3>> =
    LazyLock::new(|| algs(include_str!("algsets/pll/edge.txt")).or_skip());
static PLL_ALGS: LazyLock<AlgSet<Cube3x3>> =
    LazyLock::new(|| algs(include_str!("algsets/pll/one_look.txt")).or_skip());
static ZBLL_ALGS: LazyLock<AlgSet<Cube3x3>> = LazyLock::new(|| {
    algs(concat!(
        include_str!("algsets/zbll/t.txt"),
        include_str!("algsets/zbll/u.txt"),
        include_str!("algsets/zbll/l.txt"),
        include_str!("algsets/zbll/pi.txt"),
        include_str!("algsets/zbll/h.txt"),
        include_str!("algsets/zbll/sune.txt"),
        include_str!("algsets/zbll/anti_sune.txt"),
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
        let mut cube = Cube3x3::apply_scramble_with_seed(2);

        for partial_solution in ZZ::default().solve_steps(&mut cube) {
            println!("{}", partial_solution?);
        }
        assert!(cube.is_solved());
        Ok(())
    }

    #[test]
    fn zz_works_with_all_options() -> Result<(), Box<dyn Error>> {
        let settings = all::<ZZ>().collect::<Vec<_>>();
        let scrambled_cubes = (0..u64::try_from(settings.len())?)
            .map(Cube3x3::apply_scramble_with_seed)
            .collect::<Vec<_>>();
        for (&cube, setting) in scrambled_cubes.iter().zip(settings) {
            dbg!(&setting);
            let mut solving_cube = cube;
            println!("{cube}\n");
            for partial_solution in setting.solve_steps(&mut solving_cube) {
                print!("{}", partial_solution?);
            }
            assert!(solving_cube.is_solved());
        }
        Ok(())
    }
}
