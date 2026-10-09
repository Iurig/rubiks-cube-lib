use std::{
    any::{Any, TypeId},
    sync::{Arc, LazyLock, Mutex, PoisonError},
};

use crate::{
    AlgSet, Algorithm, Inv, Mask, PieceSet, Puzzle, fast_hash::FxMap,
    methods::step::search_step::MemoCache,
};

type UntypedCache = Arc<dyn Any + Send + Sync>;

/// The search memos of every [`Method`](crate::Method) of every [`Puzzle`], shared for the whole
/// program. Two steps that bring the same pieces home with the same sequences grow one memo, even
/// when different methods, or different options of one method, build them.
static GLOBAL_MEMO: LazyLock<Mutex<FxMap<TypeId, UntypedCache>>> = LazyLock::new(Default::default);

pub fn global_cache<P: Puzzle>() -> Arc<MemoCache<P>> {
    let type_erased = {
        let mut caches = GLOBAL_MEMO.lock().unwrap_or_else(PoisonError::into_inner);
        Arc::clone(
            caches
                .entry(TypeId::of::<P>())
                .or_insert_with(|| Arc::new(MemoCache::<P>::new())),
        )
    };
    type_erased
        .downcast::<MemoCache<P>>()
        .expect("global_cache stores only a MemoCache<P> under TypeId::of::<P>()")
}

#[derive(Debug, Eq, PartialEq)]
pub struct BfsMemo<P: Puzzle> {
    /// Every state is filtered through this, so memo keys match the forward search's masks.
    goal: PieceSet<P>,
    memorization: FxMap<Mask<P>, Algorithm<P>>,
    to_deepen: Vec<(Mask<P>, P)>,
    depth: usize,
}

impl<P: Puzzle> BfsMemo<P> {
    pub fn new(goal: PieceSet<P>) -> Self {
        let mask = Mask::filter_by_piece(&P::default(), &goal);
        Self {
            goal,
            memorization: FxMap::from_iter([(mask.clone(), Algorithm::new())]),
            to_deepen: vec![(mask, P::default())],
            depth: 0,
        }
    }

    pub fn solution(&self, mask: &Mask<P>) -> Option<&Algorithm<P>> {
        self.memorization.get(mask)
    }

    /// Number of states the next [`Self::deepen`] starts from; 0 once every reachable state is
    /// memorized.
    pub(crate) const fn frontier_len(&self) -> usize {
        self.to_deepen.len()
    }

    /// Deepens until level `depth` is expanded.
    #[cfg(test)]
    pub(crate) fn search_to(
        &mut self,
        depth: usize,
        possible_sequences: &AlgSet<P>,
        free_possible_sequences: &AlgSet<P>,
    ) {
        while self.depth <= depth {
            self.deepen(possible_sequences, free_possible_sequences);
        }
    }

    /// Expands one level. `self.depth` is the next level to expand; expanding level `d` memorizes
    /// every state of cost `d` (free sequences stay on this level) and some of cost `d + 1`.
    pub(crate) fn deepen(
        &mut self,
        possible_sequences: &AlgSet<P>,
        free_possible_sequences: &AlgSet<P>,
    ) {
        // Close the level under the free sequences before any costly one runs: a state a free
        // sequence reaches costs the same as the level, and a costly sequence that reached it
        // first would memorize it, and everything after it, one level too deep.
        let mut i = 0;
        while let Some((mask, p)) = self.to_deepen.get(i).cloned() {
            i += 1;
            for sequence in free_possible_sequences.algs() {
                if let Some(state) = self.memorize(&mask, &p, sequence) {
                    self.to_deepen.push(state);
                }
            }
        }

        let mut next_frontier = Vec::new();
        for (mask, p) in std::mem::take(&mut self.to_deepen) {
            for sequence in possible_sequences.algs() {
                if let Some(state) = self.memorize(&mask, &p, sequence) {
                    next_frontier.push(state);
                }
            }
        }
        self.to_deepen = next_frontier;
        self.depth += 1;
    }

    /// Memorizes the state `sequence` leads back from, if it is new, and returns it with its mask.
    fn memorize(
        &mut self,
        parent_mask: &Mask<P>,
        parent: &P,
        sequence: &Algorithm<P>,
    ) -> Option<(Mask<P>, P)> {
        // Walk away from the goal by undoing `sequence`, so the way back applies `sequence` as
        // written, then the parent's path.
        let moved = sequence
            .iter()
            .rev()
            .fold(parent.clone(), |puzzle, m| puzzle * m.inv());
        let mask = Mask::filter_by_piece(&moved, &self.goal);
        if self.memorization.contains_key(&mask) {
            return None;
        }
        let solution = sequence
            .iter()
            .chain(
                self.memorization
                    .get(parent_mask)
                    .expect("a parent is memorized before its children"),
            )
            .copied()
            .collect();
        self.memorization.insert(mask.clone(), solution);
        Some((mask, moved))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use fastrand::Rng;

    #[test]
    fn applying_mask_matches_filtering_through() {
        let mut random = Rng::with_seed(3);
        for _ in 0..100 {
            let pieces = Piece3x3::all()
                .filter(|_| random.bool())
                .collect::<Vec<Piece3x3>>();
            assert_eq!(
                Mask::<Cube3x3>::filter_by_piece(
                    &Cube3x3::default(),
                    &PieceSet::<Cube3x3>::from_pieces_and_orientations(
                        pieces.clone(),
                        pieces.clone()
                    )
                ),
                Mask::from_pieces_and_orientations(pieces.clone(), pieces)
            );
        }
    }

    /// A memo over the whole cube, searched with R and U moves only.
    fn r_u_memo(depth: usize) -> (BfsMemo<Cube3x3>, PieceSet<Cube3x3>) {
        let goal = PieceSet::<Cube3x3>::from_pieces(Piece3x3::all());
        let allowed = AlgSet::from_moves("R R' R2 U U' U2").unwrap();
        let mut memo = BfsMemo::new(goal.clone());
        memo.search_to(depth, &allowed, &AlgSet::new());
        (memo, goal)
    }

    #[test]
    fn memo_solution_undoes_one_move() {
        let (memo, goal) = r_u_memo(1);
        let r = Cube3x3::from_moves("R").unwrap();
        assert_eq!(
            memo.solution(&Mask::filter_by_piece(&r, &goal)),
            Some(&"R'".parse().unwrap())
        );
    }

    #[test]
    fn memo_solution_undoes_the_whole_path() {
        let (memo, goal) = r_u_memo(2);
        let r_u = Cube3x3::from_moves("R U").unwrap();
        assert_eq!(
            memo.solution(&Mask::filter_by_piece(&r_u, &goal)),
            Some(&"U' R'".parse().unwrap())
        );
    }

    /// The FR edge passes through UR on the way; the memo key must not keep UR's orientation.
    #[test]
    fn memo_keys_match_forward_masks_after_a_piece_leaves_home() {
        let goal = PieceSet::<Cube3x3>::from_pieces([Piece3x3::Edge(Edge::Fr)]);
        let allowed = AlgSet::from_moves("R' U'").unwrap();
        let mut memo = BfsMemo::new(goal.clone());
        memo.search_to(1, &allowed, &AlgSet::new());
        let r_u = Cube3x3::from_moves("R U").unwrap();
        assert_eq!(
            memo.solution(&Mask::filter_by_piece(&r_u, &goal)),
            Some(&"U' R'".parse().unwrap())
        );
    }

    /// Sune's inverse is not an allowed sequence, so the memo must solve with Sune as written.
    #[test]
    fn memo_solutions_use_the_allowed_sequences_as_written() {
        let goal = PieceSet::<Cube3x3>::from_pieces(Piece3x3::all());
        let sune = "R U R' U R U2 R'";
        let allowed = AlgSet::from_algs_in_str(sune).unwrap();
        let mut memo = BfsMemo::new(goal.clone());
        memo.search_to(0, &allowed, &AlgSet::new());
        let before_sune = Cube3x3::from_moves(sune).unwrap().inv();
        assert_eq!(
            memo.solution(&Mask::filter_by_piece(&before_sune, &goal)),
            Some(&sune.parse().unwrap())
        );
    }

    /// The `U'` state is reachable by the costly `U` and by the free `U`; the costly one is tried
    /// first. It still costs nothing, so what `R` reaches from it costs 1 and is in the memo after
    /// level 0 is expanded.
    #[test]
    fn free_sequences_are_closed_before_costly_ones_claim_their_states() {
        let goal = PieceSet::<Cube3x3>::from_pieces(Piece3x3::all());
        let allowed = AlgSet::from_moves("U R").unwrap();
        let free = AlgSet::from_moves("U").unwrap();
        let mut memo = BfsMemo::new(goal.clone());
        memo.search_to(0, &allowed, &free);
        let before_r_u = Cube3x3::from_moves("R U").unwrap().inv();
        assert_eq!(
            memo.solution(&Mask::filter_by_piece(&before_r_u, &goal)),
            Some(&"R U".parse().unwrap())
        );
    }

    /// The goal tracks the FR edge wherever it goes, orientation included. After `F` another
    /// edge sits in the FR slot, and its orientation is not part of the goal.
    #[test]
    fn mask_ignores_orientation_of_other_pieces_in_a_goal_pieces_home_slot() {
        let fr = Piece3x3::Edge(Edge::Fr);
        let goal = PieceSet::<Cube3x3>::from_pieces([fr]);
        let f = Cube3x3::from_moves("F").unwrap();
        assert_ne!(f.piece_at(fr), fr);
        assert_eq!(Mask::filter_by_piece(&f, &goal).condition(fr).orient, None);
    }

    #[test]
    fn free_sequences_are_memorized_at_depth_zero() {
        let goal = PieceSet::<Cube3x3>::from_pieces(Piece3x3::all());
        let allowed = AlgSet::from_moves("R").unwrap();
        let free = AlgSet::from_moves("U'").unwrap();
        let mut memo = BfsMemo::new(goal.clone());
        memo.search_to(0, &allowed, &free);
        let u = Cube3x3::from_moves("U").unwrap();
        assert_eq!(
            memo.solution(&Mask::filter_by_piece(&u, &goal)),
            Some(&"U'".parse().unwrap())
        );
    }

    #[test]
    fn fb_is_unchanged_by_sb_moves_when_filtered() {
        let mut cube = Cube3x3::default();
        let fb_pieces = [
            Piece3x3::Center(crate::Center::L),
            Piece3x3::Corner(crate::Corner::Dfl),
            Piece3x3::Corner(crate::Corner::Dbl),
            Piece3x3::Edge(crate::Edge::Fl),
            Piece3x3::Edge(crate::Edge::Dl),
            Piece3x3::Edge(crate::Edge::Bl),
        ];
        let sb_moves = [
            puzzles::cube3x3::moves::Move3x3 {
                part: puzzles::cube3x3::moves::MovablePart::Face(puzzles::cube3x3::pieces::Face::U),
                modifier: puzzles::cube3x3::moves::MoveModifier::Clockwise,
            },
            puzzles::cube3x3::moves::Move3x3 {
                part: puzzles::cube3x3::moves::MovablePart::Face(puzzles::cube3x3::pieces::Face::R),
                modifier: puzzles::cube3x3::moves::MoveModifier::Clockwise,
            },
            puzzles::cube3x3::moves::Move3x3 {
                part: puzzles::cube3x3::moves::MovablePart::Slice(
                    puzzles::cube3x3::pieces::Slice::M,
                ),
                modifier: puzzles::cube3x3::moves::MoveModifier::Clockwise,
            },
            puzzles::cube3x3::moves::Move3x3 {
                part: puzzles::cube3x3::moves::MovablePart::Wide(puzzles::cube3x3::pieces::Face::R),
                modifier: puzzles::cube3x3::moves::MoveModifier::Clockwise,
            },
        ];

        let fb_filter = PieceSet::<Cube3x3>::from_pieces_and_orientations(fb_pieces, fb_pieces);
        let mut rng = Rng::with_seed(0);

        for _ in 0..100 {
            cube = cube * sb_moves[rng.usize(0..4)];
            assert_eq!(
                fb_filter,
                PieceSet::<Cube3x3>::filter_by_piece(&cube, &fb_filter)
            );
        }
        cube = cube.apply_moves("L").unwrap();

        assert_ne!(
            fb_filter,
            PieceSet::<Cube3x3>::filter_by_piece(&cube, &fb_filter)
        );
    }
}
