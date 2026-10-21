#[cfg(test)]
use enum_iterator::{Sequence, all};
use std::sync::{Arc, LazyLock};

use super::helpers::{include_multiple_str, set_options};
use crate::methods::cube3x3::helpers::{
    BlockGoal, LastLayer, chain_steps, into_steps, parts, split_by_blocks,
};

use crate::{AlgSet, Cube3x3, Method, PieceSet, Step, Technique};

/// The ZZ method for the 3x3 Rubik's Cube: EO line, first two layers, then last layer.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
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
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[cfg_attr(test, derive(Sequence))]
pub struct EoLine;
/// How [`ZZ`] builds its F2L.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[cfg_attr(test, derive(Sequence))]
pub struct F2L;
/// How [`ZZ`] builds its last layer.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
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
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
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
                    FirstLook::Ocll => into_steps(LastLayer::ocll(&AlgSet::from_static(
                        include_str!("algsets/oll/ocll.txt"),
                    ))),
                    FirstLook::Coll => into_steps(LastLayer::coll(&AlgSet::from_static(
                        include_str!("algsets/oll/coll.txt"),
                    ))),
                };
                let pll_tech = match pll {
                    SecondLook::OneLook => into_steps(LastLayer::pll(&AlgSet::from_static(
                        include_str!("algsets/pll/one_look.txt"),
                    ))),
                    SecondLook::TwoLook => {
                        let corners = into_steps(LastLayer::cpll(&AlgSet::from_static(
                            include_str!("algsets/pll/corner.txt"),
                        )));
                        let edges = into_steps(LastLayer::epll(&AlgSet::from_static(
                            include_str!("algsets/pll/edge.txt"),
                        )));
                        [corners, edges].concat()
                    }
                };
                [oll_tech, pll_tech].concat()
            }
            LL::OneLook => into_steps(LastLayer::zbll(&ZBLL_ALGS)),
        };

        [eoline_tech, f2l_tech, ll_tech]
            .into_iter()
            .flatten()
            .collect()
    }
}

static EO_LINE_MOVES: LazyLock<AlgSet<Cube3x3>> = LazyLock::new(|| parts!("F B U R L D"));
static F2L_MOVES: LazyLock<AlgSet<Cube3x3>> = LazyLock::new(|| parts!("U R L"));
static F2L_BLOCKS: LazyLock<[BlockGoal; 2]> = LazyLock::new(|| {
    let f2l = PieceSet::from_algset(&ZBLL_ALGS);
    [
        BlockGoal::new(&f2l, &PieceSet::from_algset(&parts!("R U")), &F2L_MOVES),
        BlockGoal::new(&f2l, &PieceSet::from_algset(&parts!("L U")), &F2L_MOVES),
    ]
});
static ZBLL_ALGS: LazyLock<AlgSet<Cube3x3>> = LazyLock::new(|| {
    AlgSet::from_static(include_multiple_str!(
        "algsets/zbll/t.txt",
        "algsets/zbll/u.txt",
        "algsets/zbll/l.txt",
        "algsets/zbll/pi.txt",
        "algsets/zbll/h.txt",
        "algsets/zbll/sune.txt",
        "algsets/zbll/anti_sune.txt"
    ))
    .or_skip()
});

#[cfg(test)]
mod test {
    #![expect(clippy::panic_in_result_fn, reason = "standard procedure in our tests")]
    use std::error::Error;

    use crate::{Puzzle, Solution};

    use super::*;

    #[test]
    fn zz_solves_the_cube() -> Result<(), Box<dyn Error>> {
        let mut cube = Cube3x3::scrambled_with_seed(2);

        let _ = ZZ::default()
            .solve_steps(&mut cube)
            .collect::<Result<Solution<Cube3x3>, _>>()?;
        assert!(cube.is_solved());
        Ok(())
    }

    #[test]
    fn zz_works_with_all_options() -> Result<(), Box<dyn Error>> {
        let settings = all::<ZZ>().collect::<Vec<_>>();
        let scrambled_cubes = (0..u64::try_from(settings.len())?)
            .map(Cube3x3::scrambled_with_seed)
            .collect::<Vec<_>>();
        for (&cube, setting) in scrambled_cubes.iter().zip(settings) {
            dbg!(&setting);
            let mut solving_cube = cube;
            setting.solve(&mut solving_cube)?;
            assert!(solving_cube.is_solved());
        }
        Ok(())
    }
}
