use std::{
    borrow::Cow,
    sync::{Arc, LazyLock},
};

use itertools::iproduct;

use crate::{
    AlgSet, Algorithm, ByIdentity, Cube3x3, Mask, Piece3x3, PieceSet, Puzzle, Solution, Step,
    StepError,
    methods::step::combine_pruned::{PruneTable, PrunedCombine, PrunedGoal},
};

pub fn parts(text: &str) -> AlgSet<Cube3x3> {
    AlgSet::from_parts(text).expect("hand-written part lists should always parse")
}

pub fn algs(text: &str) -> AlgSet<Cube3x3> {
    AlgSet::from_algs_in_str(text).expect("hand-written algorithm sets should always parse")
}

pub(super) fn into_steps<P: Puzzle>(step: impl Step<P> + 'static) -> Vec<Arc<dyn Step<P>>> {
    vec![Arc::new(step)]
}

macro_rules! mask {
    ($mark: ident) => {
        Mask::filter_by_piece(&Cube3x3::default(), &$mark)
    };
}

#[derive(Debug)]
pub struct BlockGoal {
    goal: Mask<Cube3x3>,
    table: PruneTable<Mask<Cube3x3>>,
}

impl BlockGoal {
    pub fn new(
        after: &PieceSet<Cube3x3>,
        block: &PieceSet<Cube3x3>,
        moves: &AlgSet<Cube3x3>,
    ) -> Self {
        let goal = mask!(after).and(&mask!(block));
        let table = PruneTable::from_goal(&goal, moves);
        Self { goal, table }
    }
}

impl PrunedGoal<Cube3x3> for &'static BlockGoal {
    type Marker = ByIdentity;

    fn goal(&self) -> &Mask<Cube3x3> {
        &self.goal
    }

    fn table(&self) -> &PruneTable<Mask<Cube3x3>> {
        &self.table
    }
}

pub fn split_by_blocks(
    name: impl Into<Cow<'static, str>>,
    before: PieceSet<Cube3x3>,
    blocks: impl IntoIterator<Item = &'static BlockGoal>,
    moves: AlgSet<Cube3x3>,
) -> PrunedCombine<Cube3x3> {
    PrunedCombine::new(name, move |cube| before.applies_to(cube), blocks, moves)
}

macro_rules! set_options {
    ($step_name: literal, $field_name: ident, $option_type: ty) => {
        #[doc = concat!("Changes the ", $step_name, " options of the current method to [`", stringify!($field_name), "`](", stringify!($option_type) ,")")]
        #[must_use]
        pub const fn $field_name(mut self, $field_name: $option_type) -> Self {
            self.$field_name = $field_name;
            self
        }
    };
}
pub(crate) use set_options;

macro_rules! chain_steps {
    // The chain starts from the pieces `moves` cannot touch, which must already be solved.
    // `free` sequences, such as an AUF, may be added to every step's solution without counting
    // toward its depth. This rule must come before the one without `free`, whose `$stages` would
    // otherwise swallow `free: ...` as a stage.
    (moves: $moves:expr, goal: $goal:expr, free: $free:expr, $($stages:tt)+) => {{
        let moves: &AlgSet<Cube3x3> = $moves;
        let goal: &AlgSet<Cube3x3> = $goal;
        let free: &AlgSet<Cube3x3> = $free;
        chain_steps!(@stages moves, free, goal, (PieceSet::from_algset(moves)), [], $($stages)+)
    }};

    (moves: $moves:expr, goal: $goal:expr, $($stages:tt)+) => {
        chain_steps!(moves: $moves, goal: $goal, free: &AlgSet::new(), $($stages)+)
    };

    (@stages $moves:ident, $free:ident, $goal:ident, $befores:tt, [$($steps:expr),*],
     $name:literal $(,)?) => {{
        let steps: Vec<Arc<dyn Step<Cube3x3>>> = vec![
            $($steps,)*
            chain_steps!(@choose $moves, $free, $name, $befores, (PieceSet::from_algset($goal))),
        ];
        steps
    }};

    (@stages $moves:ident, $free:ident, $goal:ident, $befores:tt, [$($steps:expr),*],
     $name:literal => ($($divider:expr),+ $(,)?), $($rest:tt)+) => {
        chain_steps!(
            @stages $moves, $free, $goal,
            ($(PieceSet::from_algset(&$goal.combined_with(&$divider))),+),
            [$($steps,)* chain_steps!(@choose $moves, $free, $name, $befores,
                ($(PieceSet::from_algset(&$goal.combined_with(&$divider))),+))],
            $($rest)+
        )
    };
    (@choose $moves:ident, $free:ident, $name:literal, ($($before:expr),+), $afters:tt) => {
        Arc::new(Shortest::named(
            $name,
            [$(chain_steps!(@row $moves, $free, $name, $before, $afters)),+].concat(),
        ))
    };

    (@row $moves:ident, $free:ident, $name:literal, $before:expr, ($($after:expr),+)) => {{
        let row: Vec<Arc<dyn Step<Cube3x3>>> = vec![$(
            Arc::new(
                $crate::SearchStep::builder($name, $after)
                    .expect_solved($before)
                    .search_algs($moves.clone())
                    .free_algs($free.clone())
                    .build()
                    .expect("each chain_steps stage keeps the stage before it solved"),
            )
        ),+];
        row
    }};
}
pub(crate) use chain_steps;

#[derive(Debug, Default)]
pub struct LastLayer {
    pub name: Cow<'static, str>,
    pub algs: AlgSet<Cube3x3>,
    oriented: Mask<Cube3x3>,
    permuted: Mask<Cube3x3>,
}

const LL_CORNERS: [Piece3x3; 4] = {
    [
        Piece3x3::Corner(crate::Corner::Ufr),
        Piece3x3::Corner(crate::Corner::Ufl),
        Piece3x3::Corner(crate::Corner::Ubl),
        Piece3x3::Corner(crate::Corner::Ubr),
    ]
};
const LL_EDGES: [Piece3x3; 4] = {
    [
        Piece3x3::Edge(crate::Edge::Uf),
        Piece3x3::Edge(crate::Edge::Ur),
        Piece3x3::Edge(crate::Edge::Ul),
        Piece3x3::Edge(crate::Edge::Ub),
    ]
};

static AUF: LazyLock<[Algorithm<Cube3x3>; 4]> = LazyLock::new(|| {
    [
        "".parse().expect("manually typed"),
        "U".parse().expect("manually typed"),
        "U2".parse().expect("manually typed"),
        "U'".parse().expect("manually typed"),
    ]
});

fn aufs() -> &'static [Algorithm<Cube3x3>; 4] {
    &AUF
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "rarely constructed, the fields names are documentation enough"
)]
#[derive(Clone, Copy, Debug)]
pub struct LastLayerGoal {
    corner_orientation: bool,
    corner_permutation: bool,
    edge_orientation: bool,
    edge_permutation: bool,
}

impl LastLayerGoal {
    const SOLVE_LL: Self = Self {
        corner_orientation: true,
        corner_permutation: true,
        edge_orientation: true,
        edge_permutation: true,
    };
    const SET_UP_EPLL: Self = Self {
        edge_permutation: false,
        ..Self::SOLVE_LL
    };
}

impl LastLayer {
    fn new(name: impl Into<Cow<'static, str>>, algs: AlgSet<Cube3x3>, goal: LastLayerGoal) -> Self {
        let pick =
            |wanted: bool, pieces: &[Piece3x3; 4]| wanted.then_some(*pieces).into_iter().flatten();
        Self {
            name: name.into(),
            algs,
            oriented: Mask::from_pieces_and_orientations(
                [],
                pick(goal.corner_orientation, &LL_CORNERS)
                    .chain(pick(goal.edge_orientation, &LL_EDGES)),
            ),
            permuted: Mask::from_pieces_and_orientations(
                pick(goal.corner_permutation, &LL_CORNERS)
                    .chain(pick(goal.edge_permutation, &LL_EDGES)),
                [],
            ),
        }
    }
    fn post_auf(&self, cube: &Cube3x3) -> Option<Algorithm<Cube3x3>> {
        aufs()
            .iter()
            .find(|auf| self.permuted.applies_to(&cube.apply(auf)))
            .cloned()
    }

    pub fn coll(algs: AlgSet<Cube3x3>) -> Self {
        Self::new("COLL", algs, LastLayerGoal::SET_UP_EPLL)
    }
    pub fn ocll(algs: AlgSet<Cube3x3>) -> Self {
        let ocll_goal = LastLayerGoal {
            corner_orientation: true,
            corner_permutation: false,
            edge_orientation: true,
            edge_permutation: false,
        };
        Self::new("OCLL", algs, ocll_goal)
    }
    pub fn pll(algs: AlgSet<Cube3x3>) -> Self {
        Self::new("PLL", algs, LastLayerGoal::SOLVE_LL)
    }
    pub fn cpll(algs: AlgSet<Cube3x3>) -> Self {
        Self::new("CPLL", algs, LastLayerGoal::SET_UP_EPLL)
    }
    pub fn epll(algs: AlgSet<Cube3x3>) -> Self {
        Self::new("EPLL", algs, LastLayerGoal::SOLVE_LL)
    }
    pub fn zbll(algs: AlgSet<Cube3x3>) -> Self {
        Self::new("ZBLL", algs, LastLayerGoal::SOLVE_LL)
    }
}

impl Step<Cube3x3> for LastLayer {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_done(&self, puzzle: &Cube3x3) -> bool {
        // U moves never change the orientation of U-layer pieces, so only the
        // permutation needs an AUF.
        self.oriented.applies_to(puzzle) && self.post_auf(puzzle).is_some()
    }

    fn solve(&self, puzzle: &mut Cube3x3) -> Result<Solution<Cube3x3>, StepError> {
        let (pre, alg) = iproduct!(aufs(), self.algs.iter())
            .find(|(pre, alg)| self.is_done(&puzzle.apply(pre).apply(alg)))
            .ok_or(StepError::UnreachableGoal)?;
        let solved = puzzle.apply(pre).apply(alg);
        let post = self.post_auf(&solved).ok_or(StepError::UnreachableGoal)?;
        *puzzle = solved.apply(&post);

        Ok(Solution::single_segment(
            self.name(),
            pre.iter()
                .chain(alg.iter())
                .chain(post.iter())
                .copied()
                .collect(),
        ))
    }
}
