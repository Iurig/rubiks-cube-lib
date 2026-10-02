use crate::{AlgSet, Puzzle, Step, fast_hash::FxMap};
use std::{collections::VecDeque, fmt::Debug, hash::Hash, ops::Mul};

#[derive(Debug)]
struct PrunedCombine<'a, P: Puzzle> {
    name: &'a str,
    steps: Vec<Box<dyn DistanceStep<P>>>,
    moveset: AlgSet<P>,
}

trait DistanceStep<P: Puzzle>: Debug + Send + Sync {
    fn distance_from_solved(&self, puzzle: &P) -> Option<u8>;
}

#[derive(Debug, Clone)]
pub(crate) struct PruneTable<K> {
    table: FxMap<K, u8>,
}

impl<K: Eq + Hash> FromIterator<(K, u8)> for PruneTable<K> {
    fn from_iter<T: IntoIterator<Item = (K, u8)>>(iter: T) -> Self {
        Self {
            table: FxMap::from_iter(iter),
        }
    }
}

impl<'a, P: Puzzle> PrunedCombine<'a, P> {
    fn new<T: IntoIterator<Item = Box<dyn DistanceStep<P>>>>(
        name: &'a str,
        iter: T,
        moveset: AlgSet<P>,
    ) -> Self {
        Self {
            name,
            steps: Vec::<Box<dyn DistanceStep<P>>>::from_iter(iter),
            moveset,
        }
    }
}

impl<K: Clone + Eq + Hash> PruneTable<K> {
    /// Fills the table outward from the keys already in it. Each sequence of `moveset` is one
    /// step of distance, the same unit [`PrunedCombine`] searches in.
    pub(crate) fn populate<P: Puzzle>(&mut self, moveset: &AlgSet<P>)
    where
        K: Mul<P::Moves, Output = K>,
    {
        let mut to_investigate: VecDeque<(K, u8)> = self.table.clone().into_iter().collect();
        while let Some((mask, depth)) = to_investigate.pop_front() {
            for alg in moveset.algs() {
                let next = alg.iter().fold(mask.clone(), |k, &m| k * m);
                self.table.entry(next.clone()).or_insert_with(|| {
                    to_investigate.push_back((next, depth + 1));
                    depth + 1
                });
            }
        }
    }
}

impl<'a, P: Puzzle> Step<P> for PrunedCombine<'a, P> {
    fn name(&self) -> &str {
        self.name
    }
    fn is_done(&self, puzzle: &P) -> bool {
        self.steps
            .iter()
            .all(|s| s.distance_from_solved(puzzle) == Some(0))
    }
    fn solve(&self, puzzle: &mut P) -> Result<super::Solution<P>, super::StepError> {
        use super::{Segment, Solution, StepError};
        use crate::{Algorithm, Inv};
        use std::ops::ControlFlow;

        /// The largest distance any step reports: a lower bound on the moves left, because
        /// every step must be done at the end. `None` when some step cannot reach its goal.
        fn estimate<P: Puzzle>(steps: &[Box<dyn DistanceStep<P>>], puzzle: &P) -> Option<u8> {
            steps.iter().try_fold(0, |max, s| {
                s.distance_from_solved(puzzle).map(|d| max.max(d))
            })
        }

        /// One depth-first pass that never goes past `bound`, counting the sequences already
        /// applied (`depth`) plus the estimate. `last` is the sequence applied just before, and
        /// `path` holds every move applied so far. `Break` means `path` now solves every step.
        /// `Continue(Some(f))` is the smallest total over `bound` it saw, the next bound to
        /// try; `Continue(None)` means no branch can lead to the goal.
        fn search<'m, P: Puzzle>(
            steps: &[Box<dyn DistanceStep<P>>],
            moveset: &'m AlgSet<P>,
            last: Option<&'m Algorithm<P>>,
            puzzle: &P,
            path: &mut Vec<P::Moves>,
            depth: u8,
            bound: u8,
        ) -> ControlFlow<(), Option<u8>> {
            let Some(estimate) = estimate(steps, puzzle) else {
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
            // Undoing the previous sequence only returns to a state already searched. Inverted
            // once here, not once per candidate, because `inverse` allocates.
            let undo = last.map(Inv::inverse);
            for alg in moveset.algs() {
                if undo.as_ref() == Some(alg) {
                    continue;
                }
                let path_len = path.len();
                path.extend_from_slice(alg);
                match search(
                    steps,
                    moveset,
                    Some(alg),
                    &alg.iter().fold(puzzle.clone(), |state, &m| state * m),
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
        let mut path = Vec::new();
        loop {
            match search(
                &self.steps,
                &self.moveset,
                None,
                puzzle,
                &mut path,
                0,
                bound,
            ) {
                ControlFlow::Break(()) => break,
                ControlFlow::Continue(Some(next)) => bound = next,
                ControlFlow::Continue(None) => return Err(StepError::UnreachableGoal),
            }
        }
        *puzzle = path.iter().fold(puzzle.clone(), |state, &m| state * m);
        Ok(Solution::from_iter([Segment {
            moves: path,
            name: self.name.to_string(),
        }]))
    }
}
