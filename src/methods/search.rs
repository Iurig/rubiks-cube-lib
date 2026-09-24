use std::{collections::HashMap, error::Error};

use crate::{Mask, Puzzle};

pub struct BFSMemo<P: Puzzle> {
    memorization: HashMap<Mask<P>, Vec<P::Moves>>,
    to_deepen: Vec<(Mask<P>, P)>,
    depth: usize,
}

#[derive(Debug)]
pub struct MovableOrientationOnlyGoal;

impl Error for MovableOrientationOnlyGoal {}
impl std::fmt::Display for MovableOrientationOnlyGoal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl<P: Puzzle> BFSMemo<P> {
    pub fn new_empty() -> Self {
        Self {
            memorization: HashMap::new(),
            to_deepen: vec![],
            depth: 0,
        }
    }

    fn initialize(mask: Mask<P>) -> Self {
        Self {
            memorization: HashMap::from([(mask.clone(), Vec::new())]),
            to_deepen: vec![(mask, P::default())],
            depth: 0,
        }
    }

    #[must_use]
    fn filter_through(puzzle: &P, goal: &Mask<P>) -> Mask<P> {
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

    pub(crate) fn search_to(
        &mut self,
        depth: usize,
        possible_sequences: &[(Vec<P::Moves>, bool)],
    ) -> Result<(), MovableOrientationOnlyGoal> {
        let mut next_frontier = Vec::new();

        while self.depth < depth {
            while let Some((mask, p)) = self.to_deepen.pop() {
                for (move_sequence, has_cost) in possible_sequences {
                    let moved_puzzle = move_sequence
                        .iter()
                        .fold(p.clone(), |puzzle, m| puzzle * *m);
                    let filtered_moved_puzzle = Self::filter_through(&moved_puzzle, &mask);

                    if let std::collections::hash_map::Entry::Vacant(e) =
                        self.memorization.entry(filtered_moved_puzzle.clone())
                    {
                        e.insert(move_sequence.clone());
                        if *has_cost {
                            next_frontier.push((filtered_moved_puzzle.clone(), moved_puzzle));
                        } else {
                            self.to_deepen
                                .push((filtered_moved_puzzle.clone(), moved_puzzle));
                        }
                    }
                    if !(mask
                        .orientation
                        .iter()
                        .enumerate()
                        .filter(|(index, orientation)| {
                            mask.permutation[*index] == None && orientation.is_some()
                        })
                        .all(|(index, _)| {
                            filtered_moved_puzzle.clone().orientation[index].is_some()
                        }))
                    {
                        return Err(MovableOrientationOnlyGoal);
                    };
                }
            }

            self.to_deepen = next_frontier;
            next_frontier = Vec::new();
            self.depth += 1;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use fastrand::Rng;

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
