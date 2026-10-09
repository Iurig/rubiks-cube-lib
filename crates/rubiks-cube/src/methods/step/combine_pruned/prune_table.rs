use crate::{
    AlgSet, Labeled, Marker, Puzzle,
    fast_hash::{FxMap, FxSet},
};

use std::{hash::Hash, ops::Mul};

use rayon::prelude::*;

#[derive(Clone, Debug)]
pub struct PruneTable<K> {
    table: FxMap<K, u8>,
}

impl<K: Eq + Hash> FromIterator<(K, u8)> for PruneTable<K> {
    fn from_iter<T: IntoIterator<Item = (K, u8)>>(iter: T) -> Self {
        Self {
            table: FxMap::from_iter(iter),
        }
    }
}

impl<P: Puzzle, L: Marker<P, Label: Send + Sync> + Clone + Eq + Hash + Send + Sync>
    PruneTable<Labeled<P, L>>
{
    /// The distance from `goal` of every state `moveset` can reach, with `goal` at 0.
    pub(crate) fn from_goal(goal: &Labeled<P, L>, moveset: &AlgSet<P>) -> Self {
        let mut table = Self::from_iter([(goal.clone(), 0)]);
        table.populate(moveset);
        table
    }
}

impl<K: Clone + Eq + Hash> PruneTable<K> {
    pub(crate) fn get_distance(&self, state: &K) -> Option<u8> {
        self.table.get(state).copied()
    }

    /// Fills the table outward from the keys already in it. Each sequence of `moveset` is one
    /// step of distance, the same unit [`PrunedCombine`](super::PrunedCombine) searches in.
    pub(crate) fn populate<P: Puzzle>(&mut self, algset: &AlgSet<P>)
    where
        K: Mul<P::Move, Output = K> + Send + Sync,
    {
        // Every key in the frontier is at `depth`, so the depth lives once, outside the loop.
        let Some(&first_depth) = self.table.values().next() else {
            return;
        };
        debug_assert!(
            self.table.values().all(|&d| d == first_depth),
            "the keys already in the table must all be at the same distance"
        );
        let mut depth = first_depth;
        let mut frontier: Vec<K> = self.table.keys().cloned().collect();

        while !frontier.is_empty() {
            log::debug!("depth = {depth}, frontier = {}", frontier.len());
            depth += 1;

            let next: FxSet<K> = frontier
                .par_iter()
                .flat_map_iter(|representation| {
                    algset
                        .algs()
                        .iter()
                        .map(|alg| alg.iter().fold(representation.clone(), |rep, m| rep * *m))
                        .filter(|state| !self.table.contains_key(state))
                })
                .collect();
            self.table
                .par_extend(next.par_iter().map(|state| (state.clone(), depth)));
            frontier = next.into_iter().collect();
        }
    }
}
