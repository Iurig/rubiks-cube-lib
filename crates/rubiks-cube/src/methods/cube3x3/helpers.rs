use std::{
    borrow::Cow,
    sync::{Arc, LazyLock},
};

use itertools::iproduct;

use crate::{
    AlgSet, Algorithm, ByPiece, Cube3x3, Marked, Mask, Piece3x3, Puzzle, SearchStep, Solution,
    Step, StepError,
    methods::step::combine_pruned::{DistanceStep, PruneTable, PrunedCombine, PrunedGoal},
};

pub fn parts(text: &str) -> AlgSet<Cube3x3> {
    AlgSet::from_parts(text).expect("hand-written part lists should always parse")
}

pub fn algs(text: &str) -> AlgSet<Cube3x3> {
    AlgSet::from_algs_in_str(text).expect("hand-written algorithm sets should always parse")
}

pub fn search(
    name: impl Into<Cow<'static, str>>,
    before: Marked<Cube3x3>,
    after: Marked<Cube3x3>,
    moves: AlgSet<Cube3x3>,
) -> Arc<SearchStep<Cube3x3>> {
    search_with_free_algs(name, before, after, moves, AlgSet::default())
}

pub(super) fn into_steps<P: Puzzle>(step: impl Step<P> + 'static) -> Vec<Arc<dyn Step<P>>> {
    vec![Arc::new(step)]
}

pub fn search_with_free_algs(
    name: impl Into<Cow<'static, str>>,
    before: Marked<Cube3x3>,
    after: Marked<Cube3x3>,
    moves: AlgSet<Cube3x3>,
    free_algs: AlgSet<Cube3x3>,
) -> Arc<SearchStep<Cube3x3>> {
    Arc::new(SearchStep::sharing_memo(
        &super::MEMOS,
        name,
        before,
        after,
        moves,
        free_algs,
    ))
}

macro_rules! r#dyn {
    ($goal: expr) => {
        Box::new($goal) as Box<dyn DistanceStep<Cube3x3>>
    };
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
    pub fn new(after: &Marked<Cube3x3>, block: &Marked<Cube3x3>, moves: &AlgSet<Cube3x3>) -> Self {
        let goal = mask!(after).and(&mask!(block));
        let table = PruneTable::from_goal(&goal, moves);
        Self { goal, table }
    }
}

impl PrunedGoal<Cube3x3> for &'static BlockGoal {
    type Marker = ByPiece;

    fn goal(&self) -> &Mask<Cube3x3> {
        &self.goal
    }

    fn table(&self) -> &PruneTable<Mask<Cube3x3>> {
        &self.table
    }
}

pub fn split_by_blocks(
    name: impl Into<Cow<'static, str>>,
    before: Marked<Cube3x3>,
    blocks: impl IntoIterator<Item = &'static BlockGoal>,
    moves: AlgSet<Cube3x3>,
) -> PrunedCombine<Cube3x3> {
    PrunedCombine::new(
        name,
        Box::new(move |cube| before.applies_to(cube)),
        blocks.into_iter().map(|block| r#dyn!(block)),
        moves,
    )
}

macro_rules! chain_steps {
    // The chain starts from the pieces `moves` cannot touch, which must already be solved.
    // `free` sequences, such as an AUF, may be added to every step's solution without counting
    // toward its depth. This rule must come before the one without `free`, whose `$stages` would
    // otherwise swallow `free: ...` as a stage.
    (moves: $moves:expr, goal: $goal:expr, free: $free:expr, $($stages:tt)+) => {{
        let moves: &AlgSet<Cube3x3> = $moves;
        let goal: &AlgSet<Cube3x3> = $goal;
        let free: &AlgSet<Cube3x3> = $free;
        chain_steps!(@stages moves, free, goal, (Marked::from_algset(moves)), [], $($stages)+)
    }};

    (moves: $moves:expr, goal: $goal:expr, $($stages:tt)+) => {
        chain_steps!(moves: $moves, goal: $goal, free: &AlgSet::default(), $($stages)+)
    };

    (@stages $moves:ident, $free:ident, $goal:ident, $befores:tt, [$($steps:expr),*],
     $name:literal $(,)?) => {{
        let steps: Vec<Arc<dyn Step<Cube3x3>>> = vec![
            $($steps,)*
            chain_steps!(@choose $moves, $free, $name, $befores, (Marked::from_algset($goal))),
        ];
        steps
    }};

    (@stages $moves:ident, $free:ident, $goal:ident, $befores:tt, [$($steps:expr),*],
     $name:literal => ($($divider:expr),+ $(,)?), $($rest:tt)+) => {
        chain_steps!(
            @stages $moves, $free, $goal,
            ($(Marked::from_algset(&$goal.combined_with(&$divider))),+),
            [$($steps,)* chain_steps!(@choose $moves, $free, $name, $befores,
                ($(Marked::from_algset(&$goal.combined_with(&$divider))),+))],
            $($rest)+
        )
    };
    (@choose $moves:ident, $free:ident, $name:literal, ($($before:expr),+), $afters:tt) => {
        Arc::new(Choose::named(
            $name,
            [$(chain_steps!(@row $moves, $free, $name, $before, $afters)),+].concat(),
        ))
    };

    (@row $moves:ident, $free:ident, $name:literal, $before:expr, ($($after:expr),+)) => {{
        let row: Vec<Arc<dyn Step<Cube3x3>>> = vec![$(
            $crate::methods::cube3x3::helpers::search_with_free_algs(
                $name, $before, $after, $moves.clone(), $free.clone(),
            )
        ),+];
        row
    }};
}

macro_rules! parse {
    ($moves: literal) => {
        <Cube3x3 as Puzzle>::Move::sequence($moves)
            .map(|m| m.expect("manually typed move sequences should always parse"))
            .collect::<Algorithm<Cube3x3>>()
    };
}

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

static AUF: LazyLock<[Algorithm<Cube3x3>; 4]> =
    LazyLock::new(|| [parse!(""), parse!("U"), parse!("U2"), parse!("U'")]);

fn aufs() -> [Algorithm<Cube3x3>; 4] {
    AUF.clone()
}

impl LastLayer {
    #[expect(
        clippy::fn_params_excessive_bools,
        reason = "Private function to helper file - only used here."
    )]
    fn new(
        name: impl Into<Cow<'static, str>>,
        algs: AlgSet<Cube3x3>,
        co: bool,
        cp: bool,
        eo: bool,
        ep: bool,
    ) -> Self {
        let pick =
            |wanted: bool, pieces: &[Piece3x3; 4]| wanted.then_some(*pieces).into_iter().flatten();
        Self {
            name: name.into(),
            algs,
            oriented: Mask::from_pieces_and_orientations(
                [],
                pick(co, &LL_CORNERS).chain(pick(eo, &LL_EDGES)),
            ),
            permuted: Mask::from_pieces_and_orientations(
                pick(cp, &LL_CORNERS).chain(pick(ep, &LL_EDGES)),
                [],
            ),
        }
    }
    fn post_auf(&self, cube: &Cube3x3) -> Option<Algorithm<Cube3x3>> {
        aufs()
            .into_iter()
            .find(|auf| self.permuted.applies_to(&cube.apply(auf)))
    }

    pub fn coll(algs: AlgSet<Cube3x3>) -> Self {
        Self::new("COLL", algs, true, true, true, false)
    }
    pub fn ocll(algs: AlgSet<Cube3x3>) -> Self {
        Self::new("OCLL", algs, true, false, true, false)
    }
    pub fn pll(algs: AlgSet<Cube3x3>) -> Self {
        Self::new("PLL", algs, true, true, true, true)
    }
    pub fn cpll(algs: AlgSet<Cube3x3>) -> Self {
        Self::new("CPLL", algs, true, true, true, false)
    }
    pub fn epll(algs: AlgSet<Cube3x3>) -> Self {
        Self::new("EPLL", algs, true, true, true, true)
    }
    pub fn zbll(algs: AlgSet<Cube3x3>) -> Self {
        Self::new("ZBLL", algs, true, true, true, true)
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
        let solved = puzzle.apply(&pre).apply(alg);
        let post = self.post_auf(&solved).ok_or(StepError::UnreachableGoal)?;
        *puzzle = solved.apply(&post);

        Ok(Solution::single_segment(
            self.name.to_string(),
            pre.iter()
                .chain(alg.iter())
                .chain(post.iter())
                .copied()
                .collect(),
        ))
    }
}
