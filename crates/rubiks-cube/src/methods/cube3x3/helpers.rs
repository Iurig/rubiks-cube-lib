use std::sync::{Arc, LazyLock};

use itertools::iproduct;

use crate::{
    AlgSet, Algorithm, Cube3x3, Marked, Mask, Pieces3x3, Puzzle, SearchStep, Segment, Solution,
    Step, StepError,
};

pub fn parts(text: &str) -> AlgSet<Cube3x3> {
    AlgSet::from_parts(text).expect("hand-written part lists should always parse")
}

pub fn algs(text: &str) -> AlgSet<Cube3x3> {
    AlgSet::from_algs_in_str(text).expect("hand-written algorithm sets should always parse")
}

pub fn search(
    name: &'static str,
    before: &Marked<Cube3x3>,
    after: &Marked<Cube3x3>,
    moves: &AlgSet<Cube3x3>,
) -> Arc<SearchStep<Cube3x3>> {
    search_with_free_algs(name, before, after, moves, &AlgSet::default())
}

pub fn search_with_free_algs(
    name: &'static str,
    before: &Marked<Cube3x3>,
    after: &Marked<Cube3x3>,
    moves: &AlgSet<Cube3x3>,
    free_algs: &AlgSet<Cube3x3>,
) -> Arc<SearchStep<Cube3x3>> {
    Arc::new(SearchStep::sharing_memo(
        &super::MEMOS,
        name,
        before.clone(),
        after.clone(),
        moves.clone(),
        free_algs.clone(),
    ))
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
                $name, &$before, &$after, $moves, $free,
            )
        ),+];
        row
    }};
}

macro_rules! apply {
    ($cube: expr, $moves: literal) => {
        $cube
            .move_sequence($moves)
            .expect("manually typed move sequences should always parse")
    };
}

macro_rules! parse {
    ($moves: literal) => {
        <Cube3x3 as Puzzle>::Moves::sequence($moves)
            .map(|m| m.expect("manually typed move sequences should always parse"))
            .collect::<Algorithm<Cube3x3>>()
    };
}

#[derive(Debug, Default)]
pub struct LastLayer<'a> {
    pub name: &'a str,
    pub algs: AlgSet<Cube3x3>,
    oriented: Mask<Cube3x3>,
    permuted: Mask<Cube3x3>,
}

static LL_CORNERS: LazyLock<[Pieces3x3; 4]> = LazyLock::new(|| {
    [
        Pieces3x3::Corner(crate::Corner::Ufr),
        Pieces3x3::Corner(crate::Corner::Ufl),
        Pieces3x3::Corner(crate::Corner::Ubl),
        Pieces3x3::Corner(crate::Corner::Ubr),
    ]
});
static LL_EDGES: LazyLock<[Pieces3x3; 4]> = LazyLock::new(|| {
    [
        Pieces3x3::Edge(crate::Edge::Uf),
        Pieces3x3::Edge(crate::Edge::Ur),
        Pieces3x3::Edge(crate::Edge::Ul),
        Pieces3x3::Edge(crate::Edge::Ub),
    ]
});

fn aufs() -> [Algorithm<Cube3x3>; 4] {
    [parse!(""), parse!("U"), parse!("U2"), parse!("U'")]
}

impl<'a> LastLayer<'a> {
    #[expect(
        clippy::fn_params_excessive_bools,
        reason = "Private function to helper file - only used here."
    )]
    fn new(name: &'a str, algs: AlgSet<Cube3x3>, co: bool, cp: bool, eo: bool, ep: bool) -> Self {
        let pick =
            |wanted: bool, pieces: &[Pieces3x3; 4]| wanted.then_some(*pieces).into_iter().flatten();
        Self {
            name,
            algs,
            oriented: Mask::from_double_iter([], pick(co, &LL_CORNERS).chain(pick(eo, &LL_EDGES))),
            permuted: Mask::from_double_iter(pick(cp, &LL_CORNERS).chain(pick(ep, &LL_EDGES)), []),
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

impl Step<Cube3x3> for LastLayer<'_> {
    fn name(&self) -> &str {
        self.name
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
        let solved = puzzle.apply(&pre).apply(&alg);
        let post = self.post_auf(&solved).ok_or(StepError::UnreachableGoal)?;
        *puzzle = solved.apply(&post);

        Ok(Solution::from_iter([Segment {
            moves: pre
                .iter()
                .chain(alg.iter())
                .chain(post.iter())
                .copied()
                .collect(),
            name: self.name.to_string(),
        }]))
    }
}
