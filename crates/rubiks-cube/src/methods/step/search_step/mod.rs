pub mod memorization;
use std::{
    borrow::Cow,
    collections::{VecDeque, hash_map::Entry},
    sync::{Arc, Mutex},
};

use rayon::prelude::*;

#[allow(
    clippy::wildcard_imports,
    reason = "`allow`, not `expect`: the lint is skipped when the library is compiled with `cfg(test)`"
)]
use crate::Puzzle;
use crate::{
    AlgSet, Algorithm, Inv, Marked, Mask, Segment, Solution, Step, StepError, fast_hash::FxMap,
    methods::step::search_step::memorization::BfsMemo,
};

/// The pieces a memo brings home, and the sequences it searches with: everything a memo's
/// contents depend on.
type MemoKey<P> = (Marked<P>, AlgSet<P>, AlgSet<P>);

/// A memo that several steps can grow, one search at a time.
type SharedMemo<P> = Arc<Mutex<BfsMemo<P>>>;

/// One shared memo per [`MemoKey`], so steps that search for the same thing with the same
/// sequences grow the same memo, even when built at different times by different methods.
///
/// Statics cannot be generic, so each puzzle type that has methods keeps its own cache in a
/// static.
#[derive(Debug)]
pub struct MemoCache<P: Puzzle> {
    memos: Mutex<FxMap<MemoKey<P>, SharedMemo<P>>>,
}

impl<P: Puzzle> MemoCache<P> {
    pub fn new() -> Self {
        Self {
            memos: Mutex::new(FxMap::default()),
        }
    }

    /// The memo that brings `solved` home with these sequences, created empty on first use.
    fn memo(
        &self,
        solved: Marked<P>,
        search_algs: &AlgSet<P>,
        free_search_algs: &AlgSet<P>,
    ) -> SharedMemo<P> {
        // Inserting cannot leave the map half-changed, so a panic elsewhere while it was locked
        // leaves it usable.
        let mut memos = self
            .memos
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Arc::clone(
            memos
                .entry((
                    solved.clone(),
                    search_algs.clone(),
                    free_search_algs.clone(),
                ))
                .or_insert_with(|| Arc::new(Mutex::new(BfsMemo::new(solved)))),
        )
    }
}

/// A step that searches for the cheapest way to bring a set of pieces home, using only the
/// move sequences it was given.
///
/// The goal is the `after` [`Mask`]: the pieces that must end in place, and those that must
/// also be oriented. Pieces outside the mask may end anywhere. Before it searches, the step
/// checks its `before` mask and returns [`StepError::InvalidStartingState`] if the puzzle does
/// not meet it.
///
/// The search meets in the middle. A forward search from the puzzle meets a backward search
/// from the goal, which the step keeps in a memo. The memo keeps growing across solves, so
/// later solves are faster, and methods that share this step through an `Arc` share its memo.
#[derive(Debug)]
pub struct SearchStep<P: Puzzle> {
    name: Cow<'static, str>,
    before: Marked<P>,
    after: Marked<P>,
    search_algs: AlgSet<P>,
    free_search_algs: AlgSet<P>,
    memo: SharedMemo<P>,
}

impl<P: Puzzle> SearchStep<P> {
    /// A step named `name` that starts when `before` holds and searches until `after` holds.
    ///
    /// Each sequence of `search_algs` is applied as a unit: a single move, or a whole algorithm.
    #[must_use]
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        before: Marked<P>,
        after: Marked<P>,
        search_algs: AlgSet<P>,
    ) -> Self {
        Self::with_free_algs(name, before, after, search_algs, AlgSet::<P>::default())
    }

    /// Like [`new`](Self::new), but the sequences of `free_search_algs` add
    /// no cost: the search finds the fewest sequences of `search_algs`, with any number of free
    /// ones between them, such as an AUF between algorithms.
    #[must_use]
    pub fn with_free_algs(
        name: impl Into<Cow<'static, str>>,
        before: Marked<P>,
        after: Marked<P>,
        search_algs: AlgSet<P>,
        free_search_algs: AlgSet<P>,
    ) -> Self {
        let memo = Arc::new(Mutex::new(BfsMemo::new(after.or(&before))));
        Self {
            name: name.into(),
            before,
            after,
            search_algs,
            free_search_algs,
            memo,
        }
    }

    /// Like [`with_free_algs`](Self::with_free_algs), but the memo comes from `cache`,
    /// shared with every other step of `cache` that brings the same pieces home with the same
    /// sequences.
    pub(crate) fn sharing_memo(
        cache: &MemoCache<P>,
        name: impl Into<Cow<'static, str>>,
        before: Marked<P>,
        after: Marked<P>,
        search_algs: AlgSet<P>,
        free_search_algs: AlgSet<P>,
    ) -> Self {
        let memo = cache.memo(after.or(&before), &search_algs, &free_search_algs);
        Self {
            name: name.into(),
            before,
            after,
            search_algs,
            free_search_algs,
            memo,
        }
    }

    /// Whether this step and `other` grow the same memo.
    #[cfg(test)]
    pub(crate) fn shares_memo_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.memo, &other.memo)
    }

    /// Meet in the middle: a forward search from `p`, one level at a time, against the memo's
    /// backward search from the goal. Each round grows whichever side has fewer states to expand.
    #[expect(
        clippy::significant_drop_tightening,
        reason = "the memo is read and deepened on every round, so the lock is held for the whole search"
    )]
    fn solve_bfs(&self, p: &mut P) -> Result<Algorithm<P>, StepError> {
        let mut memo = self
            .memo
            .lock()
            .map_err(|_memo_error| StepError::MemoPoisoned)?;
        let mut investigated = FxMap::from_iter([(self.mask(p), None)]);
        let mut level = vec![p.clone()];
        self.close_under_free_sequences(&mut level, &mut investigated);
        let mut forward_depth = 0;

        loop {
            let best = level
                .par_iter()
                .filter_map(|cube| {
                    let tail = memo.solution(&self.mask(cube))?;
                    let mut path = self.path_to(cube, &investigated);
                    path.extend(tail.iter().copied());
                    Some((path, tail, cube))
                })
                .min_by_key(|(path, _, _)| path.len());
            if let Some((path, tail, cube)) = best {
                *p = cube.apply(tail);
                return Ok(path);
            }

            let memo_frontier = memo.frontier_len();
            if memo_frontier != 0 && memo_frontier <= level.len() {
                memo.deepen(&self.search_algs, &self.free_search_algs);
            } else {
                level = self.next_level(&level, &mut investigated);
                forward_depth += 1;
                if level.is_empty() {
                    return Err(StepError::UnreachableGoal);
                }
            }
            log::trace!(
                "Step: {}\t forward states: {}\t forward depth: {forward_depth}\t memo frontier: {}",
                self.name(),
                level.len(),
                memo.frontier_len()
            );
        }
    }

    /// The states the costly sequences reach from `level`, closed under the free sequences.
    fn next_level<'a>(
        &'a self,
        level: &[P],
        investigated: &mut FxMap<Mask<P>, Option<&'a Algorithm<P>>>,
    ) -> Vec<P> {
        let candidates: Vec<_> = level
            .par_iter()
            .flat_map_iter(|p| {
                self.search_algs.algs().iter().filter_map(|s| {
                    let moved = p.apply(s);
                    let mask = self.mask(&moved);
                    (!investigated.contains_key(&mask)).then_some((mask, s, moved))
                })
            })
            .collect();

        let mut next = Vec::new();

        for (mask, sequence, moved) in candidates {
            if let Entry::Vacant(e) = investigated.entry(mask) {
                e.insert(Some(sequence));
                next.push(moved);
            }
        }

        self.close_under_free_sequences(&mut next, investigated);
        next
    }

    /// Adds to `level` every new state its free sequences reach, repeatedly.
    fn close_under_free_sequences<'a>(
        &'a self,
        level: &mut Vec<P>,
        investigated: &mut FxMap<Mask<P>, Option<&'a Algorithm<P>>>,
    ) {
        let mut i = 0;
        while let Some(cube) = level.get(i).cloned() {
            i += 1;
            for sequence in self.free_search_algs.algs() {
                let moved = cube.apply(sequence);
                if let Entry::Vacant(e) = investigated.entry(self.mask(&moved)) {
                    e.insert(Some(sequence));
                    level.push(moved);
                }
            }
        }
    }

    /// The moves from the searched state to `cube`, read back through `investigated`.
    fn path_to(
        &self,
        cube: &P,
        investigated: &FxMap<Mask<P>, Option<&Algorithm<P>>>,
    ) -> Algorithm<P> {
        let mut path = VecDeque::new();
        let mut cube = cube.clone();
        while let &Some(sequence) = investigated
            .get(&self.mask(&cube))
            .expect("every state in a level was investigated")
        {
            cube = sequence.iter().rev().fold(cube, |c, m| c * m.inverse());
            path.push_front(sequence);
        }
        path.into_iter().flatten().copied().collect()
    }

    fn mask(&self, puzzle: &P) -> Mask<P> {
        Mask::<P>::filter_by_piece(puzzle, &self.after)
    }
}

impl<P: Puzzle> Step<P> for SearchStep<P> {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_solve(&self, puzzle: &P) -> bool {
        self.before.applies_to(puzzle)
    }

    fn is_done(&self, puzzle: &P) -> bool {
        self.before.applies_to(puzzle) && self.after.applies_to(puzzle)
    }

    fn solve(&self, p: &mut P) -> Result<Solution<P>, StepError> {
        if !self.can_solve(p) {
            return Err(StepError::InvalidStartingState);
        }
        Ok(Solution::from_iter([Segment {
            moves: self.solve_bfs(p)?,
            name: self.name().to_string(),
        }]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Cube3x3;

    /// ZZ's EO Line: the goal names a piece in two edge slots and asks only for orientation in
    /// the other ten. When the search key left out the flip of a stray edge in a named slot, seed
    /// 2 ended this step with two edges flipped.
    #[test]
    fn a_goal_mixing_named_and_orientation_only_slots_is_met() {
        let eo_line_moves = AlgSet::from_parts("F B U R L D").unwrap();
        let step = SearchStep::new(
            "EO Line",
            Marked::from_algset(&eo_line_moves),
            Marked::from_algset(&AlgSet::from_parts("U R L").unwrap()),
            eo_line_moves,
        );
        for seed in 0..4 {
            let mut cube = Cube3x3::apply_scramble_with_seed(seed);
            step.solve(&mut cube).unwrap();
            assert!(step.is_done(&cube), "seed {seed}:\n{cube}");
        }
    }
}
