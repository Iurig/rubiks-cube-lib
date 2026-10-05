mod prune_table;

pub use prune_table::PruneTable;

use crate::{AlgSet, Algorithm, Labeled, Marker, Puzzle, Segment, Solution, Step, StepError};

use std::{fmt::Debug, hash::Hash, ops::ControlFlow};

pub struct PrunedCombine<'a, P: Puzzle> {
    name: &'a str,
    steps: Vec<Box<dyn DistanceStep<P>>>,
    ready_to_solve: &'a (dyn Fn(&P) -> bool + Send + Sync),
    moveset: AlgSet<P>,
    /// `may_follow[a][b]`: whether the search tries sequence `b` of the moveset right after
    /// sequence `a`. See [`may_follow`].
    may_follow: Vec<Vec<bool>>,
}

impl<P: Puzzle> Debug for PrunedCombine<'_, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)?;
        self.steps.fmt(f)?;
        self.moveset.fmt(f)?;
        self.may_follow.fmt(f)?;
        Ok(())
    }
}

/// One goal of a [`PrunedCombine`]: how many moveset sequences a puzzle is from it.
pub trait DistanceStep<P: Puzzle>: Debug + Send + Sync {
    /// How many moveset sequences `puzzle` is from the goal, or `None` if no sequence reaches
    /// it.
    fn distance_from_solved(&self, puzzle: &P) -> Option<u8>;
}

/// A goal with a table of how far each state is from it. Every `PrunedGoal` is a
/// [`DistanceStep`] through the impl below, which is the only code that reads the table, so the
/// table is always read with the same key it was filled with.
pub trait PrunedGoal<P: Puzzle>: Debug + Send + Sync {
    /// The labels the goal uses: [`ByPiece`](crate::ByPiece) when the table must tell pieces
    /// apart, [`Tracked`](crate::Tracked) when it only needs to know which slots hold them.
    type Marker: Marker<P, Label: Send + Sync> + Clone + Eq + Hash + Debug + Send + Sync;

    /// The goal the table was built from.
    fn goal(&self) -> &Labeled<P, Self::Marker>;

    /// The table [`PruneTable::from_goal`] built from [`goal`](Self::goal).
    fn table(&self) -> &PruneTable<Labeled<P, Self::Marker>>;
}

impl<P: Puzzle, T: PrunedGoal<P>> DistanceStep<P> for T {
    fn distance_from_solved(&self, puzzle: &P) -> Option<u8> {
        // The table holds the goal carried along by sequences of moves, so the key is the goal
        // carried along by the whole puzzle. `filter_by_piece` would not do: it keeps some
        // orientation requirements on fixed slots, which builds keys the table never stored.
        self.table()
            .get_distance(&self.goal().composed_with(puzzle))
    }
}

impl<'a, P: Puzzle> PrunedCombine<'a, P> {
    pub fn new<T: IntoIterator<Item = Box<dyn DistanceStep<P>>>>(
        name: &'a str,
        ready_to_solve: &'a (dyn Fn(&P) -> bool + Send + Sync),
        iter: T,
        moveset: AlgSet<P>,
    ) -> Self {
        Self {
            name,
            steps: Vec::<Box<dyn DistanceStep<P>>>::from_iter(iter),
            ready_to_solve,
            may_follow: may_follow(&moveset),
            moveset,
        }
    }
}

/// For each pair of sequences `(a, b)` of `moveset`, whether a search should try `b` right after
/// `a`. It should not when `a b` does no more than something shorter, or than the same pair in
/// the other order:
///
/// - `a b` leaves the puzzle as it was (`U U'`), or as one sequence of the moveset does (`U U` is
///   `U2`): the shorter path reaches the same state, at a smaller depth.
/// - `a` and `b` commute (`U D` is `D U`) and `b` comes before `a` in the moveset: only the order
///   with the earlier sequence first is tried.
///
/// Neither rule loses a state. Among the shortest paths to a state, the one that comes first in
/// moveset order has no pair either rule rejects: the first rule would give a shorter path, and
/// the second the same path with an earlier sequence first.
fn may_follow<P: Puzzle>(moveset: &AlgSet<P>) -> Vec<Vec<bool>> {
    let algs = moveset.algs();
    let alone: Vec<P> = algs.iter().map(|alg| P::default().apply(alg)).collect();
    (0..algs.len())
        .zip(alone.iter().zip(algs))
        .map(|(a_index, (after_a, a))| {
            (0..algs.len())
                .zip(alone.iter().zip(algs))
                .map(|(b_index, (after_b, b))| {
                    let a_then_b = after_a.apply(b);
                    let shortens = a_then_b == P::default() || alone.contains(&a_then_b);
                    let commutes_out_of_order = b_index < a_index && a_then_b == after_b.apply(a);
                    !shortens && !commutes_out_of_order
                })
                .collect()
        })
        .collect()
}

impl<P: Puzzle> Step<P> for PrunedCombine<'_, P> {
    fn name(&self) -> &str {
        self.name
    }

    fn can_solve(&self, puzzle: &P) -> bool {
        (self.ready_to_solve)(puzzle)
    }

    fn is_done(&self, puzzle: &P) -> bool {
        self.steps
            .iter()
            .all(|s| s.distance_from_solved(puzzle) == Some(0))
    }

    fn solve(&self, puzzle: &mut P) -> Result<crate::Solution<P>, crate::StepError> {
        /// The largest distance any step reports: a lower bound on the moves left, because
        /// every step must be done at the end. `None` when some step cannot reach its goal.
        fn estimate<P: Puzzle>(steps: &[Box<dyn DistanceStep<P>>], puzzle: &P) -> Option<u8> {
            steps.iter().try_fold(0, |max, s| {
                s.distance_from_solved(puzzle).map(|d| max.max(d))
            })
        }

        /// One depth-first pass that never goes past `bound`, counting the sequences already
        /// applied (`depth`) plus the estimate. `last` is the index of the sequence applied just
        /// before, and `path` holds every move applied so far. `Break` means `path` now
        /// solves every step. `Continue(Some(f))` is the smallest total over `bound` it
        /// saw, the next bound to try; `Continue(None)` means no branch can lead to the
        /// goal.
        fn search<P: Puzzle>(
            combine: &PrunedCombine<'_, P>,
            last: Option<usize>,
            puzzle: &P,
            path: &mut Algorithm<P>,
            depth: u8,
            bound: u8,
        ) -> ControlFlow<(), Option<u8>> {
            let Some(estimate) = estimate(&combine.steps, puzzle) else {
                return ControlFlow::Continue(None);
            };
            let Some(total) = depth.checked_add(estimate) else {
                return ControlFlow::Continue(None);
            };
            if total > bound {
                return ControlFlow::Continue(Some(total));
            }
            if estimate == 0 {
                return ControlFlow::Break(());
            }
            let Some(next_depth) = depth.checked_add(1) else {
                return ControlFlow::Continue(None);
            };
            let mut next_bound: Option<u8> = None;
            // The sequences `may_follow` rejects after `last` only reach states that a shorter
            // or earlier-ordered path also reaches. No row on the first sequence: all are tried.
            let allowed = last.and_then(|l| combine.may_follow.get(l));
            for (index, alg) in combine.moveset.algs().iter().enumerate() {
                if allowed.is_some_and(|row| row.get(index) == Some(&false)) {
                    continue;
                }
                let path_len = path.len();
                path.extend_from(alg);
                match search(
                    combine,
                    Some(index),
                    &puzzle.apply(alg),
                    path,
                    next_depth,
                    bound,
                ) {
                    ControlFlow::Break(()) => return ControlFlow::Break(()),
                    ControlFlow::Continue(Some(f)) => {
                        next_bound = Some(next_bound.map_or(f, |n| n.min(f)));
                    }
                    ControlFlow::Continue(None) => {}
                }
                path.truncate(path_len);
            }
            ControlFlow::Continue(next_bound)
        }

        let mut bound = estimate(&self.steps, puzzle).ok_or(StepError::UnreachableGoal)?;
        let mut path = Algorithm::new();
        loop {
            match search(self, None, puzzle, &mut path, 0, bound) {
                ControlFlow::Break(()) => break,
                ControlFlow::Continue(Some(next)) => {
                    debug_assert!(next > bound);
                    bound = next;
                }
                ControlFlow::Continue(None) => return Err(StepError::UnreachableGoal),
            }
        }
        *puzzle = puzzle.apply(&path);
        Ok(Solution::from_iter([Segment {
            moves: path,
            name: self.name.to_string(),
        }]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cube3x3, fast_hash::FxSet};

    /// Every face, clockwise, counterclockwise, and double: 18 single moves.
    fn face_turns() -> AlgSet<Cube3x3> {
        AlgSet::from_parts("U D F B L R").unwrap()
    }

    /// Whether `second` may follow `first` in the `may_follow` table of `moveset`, both written
    /// as one move.
    fn follows(moveset: &AlgSet<Cube3x3>, first: &str, second: &str) -> bool {
        let index = |name: &str| {
            moveset
                .algs()
                .iter()
                .position(|alg| alg.iter().map(ToString::to_string).collect::<String>() == name)
                .unwrap()
        };
        may_follow(moveset)[index(first)][index(second)]
    }

    #[test]
    fn a_face_never_follows_itself() {
        let moveset = face_turns();
        for first in ["U", "U'", "U2"] {
            for second in ["U", "U'", "U2"] {
                assert!(!follows(&moveset, first, second), "{first} {second}");
            }
        }
    }

    #[test]
    fn opposite_faces_follow_each_other_in_one_order_only() {
        let moveset = face_turns();
        for (first, second) in [("U", "D"), ("U'", "D2"), ("R", "L'"), ("F2", "B")] {
            assert!(
                follows(&moveset, first, second) != follows(&moveset, second, first),
                "{first} {second}"
            );
        }
    }

    #[test]
    fn faces_that_do_not_commute_follow_each_other_both_ways() {
        let moveset = face_turns();
        for (first, second) in [("U", "R"), ("R'", "F2"), ("D2", "L")] {
            assert!(follows(&moveset, first, second), "{first} {second}");
            assert!(follows(&moveset, second, first), "{second} {first}");
        }
    }

    /// The pruning must not lose a state: up to three moves, the paths `may_follow` allows reach
    /// every state that unpruned paths reach.
    #[test]
    fn pruned_paths_reach_every_state_within_three_moves() {
        let moveset = face_turns();
        let table = may_follow(&moveset);
        let algs = moveset.algs();
        let mut all: FxSet<Cube3x3> = FxSet::from_iter([Cube3x3::default()]);
        let mut pruned = all.clone();
        let mut all_frontier = vec![Cube3x3::default()];
        let mut pruned_frontier = vec![(Cube3x3::default(), None::<usize>)];
        for _ in 0..3 {
            all_frontier = all_frontier
                .iter()
                .flat_map(|cube| algs.iter().map(|alg| cube.apply(alg)))
                .collect();
            all.extend(all_frontier.iter().copied());
            pruned_frontier = pruned_frontier
                .iter()
                .flat_map(|&(cube, last)| {
                    let table = &table;
                    algs.iter()
                        .enumerate()
                        .filter(move |&(index, _)| last.is_none_or(|l| table[l][index]))
                        .map(move |(index, alg)| (cube.apply(alg), Some(index)))
                })
                .collect();
            pruned.extend(pruned_frontier.iter().map(|&(cube, _)| cube));
        }
        assert_eq!(pruned.len(), all.len());
    }
}
