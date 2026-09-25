use std::{collections::hash_map::Entry, sync::Mutex};

use crate::{Mask, methods::search::BFSMemo};
#[allow(
    clippy::wildcard_imports,
    reason = "`allow`, not `expect`: the lint is skipped when the library is compiled with `cfg(test)`"
)]
use crate::{Puzzle, methods::*};

#[derive(Debug)]
pub struct SearchStep<P: Puzzle> {
    name: &'static str,
    before: Mask<P>,
    after: Mask<P>,
    allowed_moves: Vec<(Vec<P::Moves>, bool)>,
    memo: Mutex<BFSMemo<P>>,
}

impl<P: Puzzle> SearchStep<P> {
    #[must_use]
    pub fn new(
        name: &'static str,
        before: Mask<P>,
        after: Mask<P>,
        allowed_moves: Vec<(Vec<P::Moves>, bool)>,
    ) -> Self {
        Self {
            memo: Mutex::new(BFSMemo::new(&after)),
            name,
            before,
            after,
            allowed_moves,
        }
    }

    /// Meet in the middle: a forward search from `p`, one level at a time, against the memo's
    /// backward search from the goal. Each round grows whichever side has fewer states to expand.
    #[expect(
        clippy::significant_drop_tightening,
        reason = "the memo is read and deepened on every round, so the lock is held for the whole search"
    )]
    fn solve_bfs(&self, p: &mut P) -> Result<Vec<P::Moves>, StepError> {
        let mut memo = self.memo.lock().map_err(|_| StepError::MemoPoisoned)?;
        let mut investigated = HashMap::from([(self.mask(p), None)]);
        let mut level = vec![p.clone()];
        self.close_under_free_sequences(&mut level, &mut investigated);
        let mut forward_depth = 0;

        loop {
            // Checking only the newest forward level is enough: an optimal solution passes
            // through this level, and whatever it has left is in the memo once it is short enough.
            let best = level
                .iter()
                .filter_map(|cube| {
                    let tail = memo.solution(&self.mask(cube))?;
                    let mut path = self.path_to(cube, &investigated);
                    path.extend(tail.iter().copied());
                    Some((path, tail, cube))
                })
                .min_by_key(|(path, _, _)| path.len());
            if let Some((path, tail, cube)) = best {
                *p = tail.iter().fold(cube.clone(), |c, m| c * *m);
                return Ok(path);
            }

            let memo_frontier = memo.frontier_len();
            if memo_frontier != 0 && memo_frontier <= level.len() {
                memo.deepen(&self.allowed_moves);
            } else {
                level = self.next_level(&level, &mut investigated);
                forward_depth += 1;
                if level.is_empty() {
                    return Err(StepError::UnrecheableGoal);
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
    fn next_level(
        &self,
        level: &[P],
        investigated: &mut HashMap<Mask<P>, Option<Vec<P::Moves>>>,
    ) -> Vec<P> {
        let mut next = Vec::new();
        for cube in level {
            for (sequence, _) in self.allowed_moves.iter().filter(|(_, has_cost)| *has_cost) {
                let moved = sequence.iter().fold(cube.clone(), |c, m| c * *m);
                if let Entry::Vacant(e) = investigated.entry(self.mask(&moved)) {
                    e.insert(Some(sequence.clone()));
                    next.push(moved);
                }
            }
        }
        self.close_under_free_sequences(&mut next, investigated);
        next
    }

    /// Adds to `level` every new state its free sequences reach, repeatedly.
    fn close_under_free_sequences(
        &self,
        level: &mut Vec<P>,
        investigated: &mut HashMap<Mask<P>, Option<Vec<P::Moves>>>,
    ) {
        let mut i = 0;
        while let Some(cube) = level.get(i).cloned() {
            i += 1;
            for (sequence, _) in self.allowed_moves.iter().filter(|(_, has_cost)| !has_cost) {
                let moved = sequence.iter().fold(cube.clone(), |c, m| c * *m);
                if let Entry::Vacant(e) = investigated.entry(self.mask(&moved)) {
                    e.insert(Some(sequence.clone()));
                    level.push(moved);
                }
            }
        }
    }

    /// The moves from the searched state to `cube`, read back through `investigated`.
    fn path_to(
        &self,
        cube: &P,
        investigated: &HashMap<Mask<P>, Option<Vec<P::Moves>>>,
    ) -> Vec<P::Moves> {
        let mut path = VecDeque::new();
        let mut cube = cube.clone();
        while let Some(sequence) = investigated
            .get(&self.mask(&cube))
            .expect("every state in a level was investigated")
        {
            cube = sequence.iter().rev().fold(cube, |c, m| c * m.inverse());
            path.push_front(sequence);
        }
        path.into_iter().flatten().copied().collect()
    }

    fn can_apply(&self, cube: &P) -> bool {
        self.before.applies_to(cube)
    }

    pub(crate) fn before(&self) -> Mask<P> {
        self.before.clone()
    }
    pub(crate) fn after(&self) -> Mask<P> {
        self.after.clone()
    }
    fn mask(&self, puzzle: &P) -> Mask<P> {
        BFSMemo::<P>::filter_through(puzzle, &self.after)
    }
}

impl<P: Puzzle> Step<P> for SearchStep<P> {
    fn name(&self) -> &'static str {
        self.name
    }

    fn is_done(&self, puzzle: &P) -> bool {
        self.after.applies_to(puzzle)
    }

    fn allowed_move_sequences(&self) -> Vec<Vec<<P as Puzzle>::Moves>> {
        self.allowed_moves.iter().map(|(m, _)| m).cloned().collect()
    }

    fn solve(&self, p: &mut P) -> Result<Solution<P>, StepError> {
        Ok(Solution::from_iter([(self.solve_bfs(p)?, self.name())]))
    }
}
