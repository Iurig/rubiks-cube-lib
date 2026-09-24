use std::collections::HashMap;

use crate::{Inv, Mask, Puzzle};

#[derive(Debug, PartialEq, Eq)]
pub struct BFSMemo<P: Puzzle> {
    /// Every state is filtered through this, so memo keys match the forward search's masks.
    goal: Mask<P>,
    memorization: HashMap<Mask<P>, Vec<P::Moves>>,
    to_deepen: Vec<(Mask<P>, P)>,
    depth: usize,
}

impl<P: Puzzle> BFSMemo<P> {
    pub fn new(goal: &Mask<P>) -> Self {
        Self {
            goal: goal.clone(),
            memorization: HashMap::from([(goal.clone(), vec![])]),
            to_deepen: vec![(goal.clone(), P::default())],
            depth: 0,
        }
    }

    pub fn solution(&self, mask: &Mask<P>) -> Option<Vec<P::Moves>> {
        self.memorization.get(mask).cloned()
    }

    #[must_use]
    #[allow(clippy::indexing_slicing)]
    pub(crate) fn filter_through(puzzle: &P, goal: &Mask<P>) -> Mask<P> {
        Mask {
            permutation: P::ALL_PIECES
                .iter()
                .map(|&slot| {
                    if goal
                        .permutation
                        .iter()
                        .any(|piece| piece == &Some(puzzle.piece_at(&slot)))
                    {
                        Some(puzzle.piece_at(&slot))
                    } else {
                        None
                    }
                })
                .collect::<Box<[Option<P::Piece>]>>(),
            orientation: P::ALL_PIECES
                .iter()
                .map(|&slot| {
                    if goal
                        .permutation
                        .iter()
                        .any(|piece| piece == &Some(puzzle.piece_at(&slot)))
                        || goal.orientation[P::index(slot)].is_some()
                    {
                        Some(puzzle.orientation_at(&slot))
                    } else {
                        None
                    }
                })
                .collect::<Box<[Option<usize>]>>(),
        }
    }

    /// Number of states the next [`Self::deepen`] starts from; 0 once every reachable state is memorized.
    pub(crate) const fn frontier_len(&self) -> usize {
        self.to_deepen.len()
    }

    /// Deepens until level `depth` is expanded.
    #[cfg(test)]
    pub(crate) fn search_to(&mut self, depth: usize, possible_sequences: &[(Vec<P::Moves>, bool)]) {
        while self.depth <= depth {
            self.deepen(possible_sequences);
        }
    }

    /// Expands one level. `self.depth` is the next level to expand; expanding level `d` memorizes
    /// every state of cost `d` (free sequences stay on this level) and some of cost `d + 1`.
    pub(crate) fn deepen(&mut self, possible_sequences: &[(Vec<P::Moves>, bool)]) {
        let mut next_frontier = Vec::new();
        while let Some((mask, p)) = self.to_deepen.pop() {
            let parent_solution = self
                .memorization
                .get(&mask)
                .expect("every mask in to_deepen was memorized when pushed")
                .clone();
            for (move_sequence, has_cost) in possible_sequences {
                // Walk away from the goal by undoing `move_sequence`, so the way back applies
                // `move_sequence` as written, then the parent's path.
                let moved_puzzle = move_sequence
                    .iter()
                    .rev()
                    .fold(p.clone(), |puzzle, m| puzzle * m.inverse());
                let filtered_moved_puzzle = Self::filter_through(&moved_puzzle, &self.goal);

                if let std::collections::hash_map::Entry::Vacant(e) =
                    self.memorization.entry(filtered_moved_puzzle.clone())
                {
                    e.insert(
                        move_sequence
                            .iter()
                            .chain(parent_solution.iter())
                            .copied()
                            .collect(),
                    );
                    if *has_cost {
                        next_frontier.push((filtered_moved_puzzle.clone(), moved_puzzle));
                    } else {
                        self.to_deepen
                            .push((filtered_moved_puzzle.clone(), moved_puzzle));
                    }
                }
            }
        }
        self.to_deepen = next_frontier;
        self.depth += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use fastrand::Rng;
    use puzzles::cube3by3::moves::Move3x3;

    #[test]
    fn applying_mask_matches_filtering_through() {
        let mut random = Rng::new();
        random.seed(3);
        for _ in 0..100 {
            let pieces = Cube3x3::ALL_PIECES
                .iter()
                .filter(|_| random.bool())
                .copied()
                .collect::<Vec<Pieces3x3>>();
            assert_eq!(
                BFSMemo::<Cube3x3>::filter_through(
                    &Cube3x3::default(),
                    &Mask::new(pieces.clone(), pieces.clone())
                ),
                Mask::new(pieces.clone(), pieces)
            );
        }
    }

    fn moves(text: &str) -> Vec<Move3x3> {
        Move3x3::sequence(text).map(Result::unwrap).collect()
    }

    /// A memo over the whole cube, searched with R and U moves only.
    fn r_u_memo(depth: usize) -> (BFSMemo<Cube3x3>, Mask<Cube3x3>) {
        let goal = Mask::new_from_pieces(Cube3x3::ALL_PIECES.iter().copied());
        let allowed = ["R", "R'", "R2", "U", "U'", "U2"].map(|m| (moves(m), true));
        let mut memo = BFSMemo::new(&goal);
        memo.search_to(depth, &allowed);
        (memo, goal)
    }

    #[test]
    fn memo_solution_undoes_one_move() {
        let (memo, goal) = r_u_memo(1);
        let r = Cube3x3::from_solved("R").unwrap();
        assert_eq!(
            memo.solution(&BFSMemo::filter_through(&r, &goal)),
            Some(moves("R'"))
        );
    }

    #[test]
    fn memo_solution_undoes_the_whole_path() {
        let (memo, goal) = r_u_memo(2);
        let r_u = Cube3x3::from_solved("R U").unwrap();
        assert_eq!(
            memo.solution(&BFSMemo::filter_through(&r_u, &goal)),
            Some(moves("U' R'"))
        );
    }

    /// The FR edge passes through UR on the way; the memo key must not keep UR's orientation.
    #[test]
    fn memo_keys_match_forward_masks_after_a_piece_leaves_home() {
        let goal = Mask::<Cube3x3>::new_from_pieces([Pieces3x3::Edge(Edge::Fr)]);
        let allowed = [(moves("R'"), true), (moves("U'"), true)];
        let mut memo = BFSMemo::new(&goal);
        memo.search_to(1, &allowed);
        let r_u = Cube3x3::from_solved("R U").unwrap();
        assert_eq!(
            memo.solution(&BFSMemo::filter_through(&r_u, &goal)),
            Some(moves("U' R'"))
        );
    }

    /// Sune's inverse is not an allowed sequence, so the memo must solve with Sune as written.
    #[test]
    fn memo_solutions_use_the_allowed_sequences_as_written() {
        let goal = Mask::new_from_pieces(Cube3x3::ALL_PIECES.iter().copied());
        let sune = "R U R' U R U2 R'";
        let allowed = [(moves(sune), true)];
        let mut memo = BFSMemo::new(&goal);
        memo.search_to(0, &allowed);
        let before_sune = Cube3x3::from_solved(sune).unwrap().inverse();
        assert_eq!(
            memo.solution(&BFSMemo::filter_through(&before_sune, &goal)),
            Some(moves(sune))
        );
    }

    #[test]
    fn free_sequences_are_memorized_at_depth_zero() {
        let goal = Mask::new_from_pieces(Cube3x3::ALL_PIECES.iter().copied());
        let allowed = [(moves("R"), true), (moves("U'"), false)];
        let mut memo = BFSMemo::new(&goal);
        memo.search_to(0, &allowed);
        let u = Cube3x3::from_solved("U").unwrap();
        assert_eq!(
            memo.solution(&BFSMemo::filter_through(&u, &goal)),
            Some(moves("U'"))
        );
    }

    #[test]
    fn filter_through_follows_moves() {
        let mut mask = Mask::<Cube3x3>::new_empty();

        let slot = mask
            .permutation
            .get_mut(Cube3x3::index(Pieces3x3::Corner(Corner::Ufl)))
            .unwrap();

        *slot = Some(Pieces3x3::Corner(Corner::Ufr));

        let l_corner =
            BFSMemo::<Cube3x3>::filter_through(&Cube3x3::from_solved("U L").unwrap(), &mask);

        assert!(
            l_corner.permutation[Cube3x3::index(Pieces3x3::Corner(Corner::Ufr))].is_none(),
            "{:?}",
            l_corner.permutation[Cube3x3::index(Pieces3x3::Corner(Corner::Ufr))]
        );
        assert!(l_corner.permutation[Cube3x3::index(Pieces3x3::Corner(Corner::Ufl))].is_none());
        assert_eq!(
            l_corner.permutation[Cube3x3::index(Pieces3x3::Corner(Corner::Dfl))],
            Some(Pieces3x3::Corner(Corner::Ufr))
        );
    }

    #[test]
    fn fb_is_unchanged_by_sb_moves_when_filtered() {
        let mut cube = Cube3x3::default();
        let fb_pieces = [
            Pieces3x3::Center(crate::Center::L),
            Pieces3x3::Corner(crate::Corner::Dfl),
            Pieces3x3::Corner(crate::Corner::Dbl),
            Pieces3x3::Edge(crate::Edge::Fl),
            Pieces3x3::Edge(crate::Edge::Dl),
            Pieces3x3::Edge(crate::Edge::Bl),
        ];
        let sb_moves = [
            puzzles::cube3by3::moves::Move3x3 {
                part: puzzles::cube3by3::moves::MovablePart::Face(
                    puzzles::cube3by3::pieces::Faces::U,
                ),
                modifier: puzzles::cube3by3::moves::MoveModifier::Clockwise,
            },
            puzzles::cube3by3::moves::Move3x3 {
                part: puzzles::cube3by3::moves::MovablePart::Face(
                    puzzles::cube3by3::pieces::Faces::R,
                ),
                modifier: puzzles::cube3by3::moves::MoveModifier::Clockwise,
            },
            puzzles::cube3by3::moves::Move3x3 {
                part: puzzles::cube3by3::moves::MovablePart::Slice(
                    puzzles::cube3by3::pieces::Slices::M,
                ),
                modifier: puzzles::cube3by3::moves::MoveModifier::Clockwise,
            },
            puzzles::cube3by3::moves::Move3x3 {
                part: puzzles::cube3by3::moves::MovablePart::Wide(
                    puzzles::cube3by3::pieces::Faces::R,
                ),
                modifier: puzzles::cube3by3::moves::MoveModifier::Clockwise,
            },
        ];

        let fb_filter = Mask::<Cube3x3>::new(fb_pieces, fb_pieces);
        let mut rng = Rng::new();

        for _ in 0..100 {
            cube = cube * sb_moves[Rng::usize(&mut rng, 0..4)];
            assert_eq!(fb_filter, BFSMemo::filter_through(&cube, &fb_filter));
        }
        cube = cube.move_sequence("L").unwrap();

        assert_ne!(fb_filter, BFSMemo::filter_through(&cube, &fb_filter));
    }
}
