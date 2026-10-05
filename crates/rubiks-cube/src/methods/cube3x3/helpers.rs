use std::sync::Arc;

use crate::{AlgSet, Cube3x3, Marked, SearchStep};

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
