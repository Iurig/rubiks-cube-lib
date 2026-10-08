#[cfg(test)]
use enum_iterator::{Sequence, all};
use std::sync::{Arc, LazyLock};

use crate::methods::cube3x3::helpers::{
    BlockGoal, LastLayer, algs, chain_steps, into_steps, parts, set_options, split_by_blocks,
};

use crate::{AlgSet, Choose, Cube3x3, Method, PieceSet, Step, Technique};

/// The ZZ method for the 3x3 Rubik's Cube: EO line, first two layers, then last layer.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(Sequence))]
pub struct ZZ {
    eoline: EoLine,
    f2l: F2L,
    ll: LL,
}

impl ZZ {
    /// The ZZ-a variant of the ZZ method. Uses ZBLL on the last layer.
    pub const ZZ_A: Self = Self {
        eoline: EoLine,
        f2l: F2L,
        ll: LL::OneLook,
    };
    set_options!("EOLine", eoline, EoLine);
    set_options!("F2L", f2l, F2L);
    set_options!("last layer", ll, LL);
}

/// How [`ZZ`] builds its first step, the `EOLine`.
#[non_exhaustive]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(Sequence))]
pub struct EoLine;
/// How [`ZZ`] builds its F2L.
#[non_exhaustive]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(Sequence))]
pub struct F2L;
/// How [`ZZ`] builds its last layer.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(Sequence))]
pub enum LL {
    /// 2-look last layer.
    TwoLook {
        /// The first look.
        first: FirstLook,
        /// The second look.
        second: SecondLook,
    },
    /// 1-look last layer, solved with ZBLL.
    OneLook,
}
/// The first step of a two-look ZZ last layer.
#[non_exhaustive]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(Sequence))]
pub enum FirstLook {
    /// Orientation of the Corners of the Last Layer.
    #[default]
    Ocll,
    /// Orientation and permutation of the Corners of the Last Layer.
    Coll,
}
/// The second step of a two-look ZZ last layer.
#[non_exhaustive]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(Sequence))]
pub enum SecondLook {
    /// Standard PLL
    #[default]
    OneLook,
    /// Two-look PLL
    TwoLook,
}

impl Default for LL {
    fn default() -> Self {
        Self::TwoLook {
            first: FirstLook::default(),
            second: SecondLook::default(),
        }
    }
}

impl Method<Cube3x3> for ZZ {
    fn to_technique(&self) -> Technique<Cube3x3> {
        let eoline_tech = match self.eoline {
            EoLine => chain_steps!(moves: &EO_LINE_MOVES, goal: &F2L_MOVES, "EO Line"),
        };

        let f2l_tech = match self.f2l {
            F2L => vec![Arc::from(split_by_blocks(
                "F2L",
                PieceSet::from_algset(&F2L_MOVES),
                F2L_BLOCKS.iter(),
                F2L_MOVES.clone(),
            )) as Arc<dyn Step<Cube3x3>>],
        };

        let ll_tech = match self.ll {
            LL::TwoLook {
                first: oll,
                second: pll,
            } => {
                let oll_tech = match oll {
                    FirstLook::Ocll => into_steps(LastLayer::ocll(OCLL_ALGS.clone())),
                    FirstLook::Coll => into_steps(LastLayer::coll(COLL_ALGS.clone())),
                };
                let pll_tech = match pll {
                    SecondLook::OneLook => into_steps(LastLayer::pll(PLL_ALGS.clone())),
                    SecondLook::TwoLook => {
                        let corners = into_steps(LastLayer::cpll(PLL_CORNER_ALGS.clone()));
                        let edges = into_steps(LastLayer::epll(PLL_EDGE_ALGS.clone()));
                        [corners, edges].concat()
                    }
                };
                [oll_tech, pll_tech].concat()
            }
            LL::OneLook => into_steps(LastLayer::zbll(ZBLL_ALGS.clone())),
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
    let f2l = PieceSet::from_algset(&OCLL_ALGS);
    ["R U", "L U"].map(|other_side| {
        BlockGoal::new(&f2l, &PieceSet::from_algset(&parts(other_side)), &F2L_MOVES)
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
