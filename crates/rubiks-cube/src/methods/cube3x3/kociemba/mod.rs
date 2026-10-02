use std::{
    collections::VecDeque,
    hash::Hash,
    ops::Mul,
    sync::{Arc, LazyLock},
};

use crate::{
    AlgSet, Cube3x3, Edge, Labeled, Mask, Method, Orientation3x3, Pieces3x3, Puzzle, Step,
    StepError, Tracked, methods::combine_pruned::PruneTable, puzzles::cube3by3::moves::Move3x3,
    zn::Zn,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DominoEdges(Labeled<Cube3x3, Tracked>);

impl Mul<Move3x3> for DominoEdges {
    type Output = Self;
    fn mul(self, rhs: Move3x3) -> Self::Output {
        Self(self.0 * rhs)
    }
}

const RIM: [Pieces3x3; 4] = [
    Pieces3x3::Edge(Edge::Fl),
    Pieces3x3::Edge(Edge::Fr),
    Pieces3x3::Edge(Edge::Bl),
    Pieces3x3::Edge(Edge::Br),
];

static CORNERS_PHASE_1_GOAL: LazyLock<Labeled<Cube3x3, Tracked>> = LazyLock::new(|| {
    Labeled::<Cube3x3, Tracked>::from_double_iter(
        [],
        Cube3x3::ALL_PIECES
            .iter()
            .filter(|&p| matches!(p, Pieces3x3::Corner(_)))
            .copied(),
    )
});

static CORNERS_PHASE_1_TABLE: LazyLock<PruneTable<Mask<Cube3x3>>> = LazyLock::new(|| {
    let mut prune_table = PruneTable::from_iter([(
        Mask::<Cube3x3>::filter_by_piece(&Cube3x3::default(), &CORNERS_PHASE_1_GOAL),
        0,
    )]);
    prune_table.populate(&AlgSet::<Cube3x3>::from_iter(
        Cube3x3::ALL_MOVES.iter().map(|&m| vec![m]),
    ));
    prune_table
});

static EDGES_PHASE_1_GOAL: LazyLock<DominoEdges> = LazyLock::new(|| {
    DominoEdges(Labeled::<Cube3x3, Tracked>::from_iter(
        Cube3x3::ALL_PIECES.iter().filter_map(|&p| {
            [
                Pieces3x3::Edge(Edge::Fl),
                Pieces3x3::Edge(Edge::Fr),
                Pieces3x3::Edge(Edge::Bl),
                Pieces3x3::Edge(Edge::Br),
            ]
            .contains(&p)
            .then_some((p, ()))
        }),
        Cube3x3::ALL_PIECES
            .iter()
            .filter(|&p| matches!(p, Pieces3x3::Edge(_)))
            .copied(),
    ))
});

static EDGES_PHASE_1_TABLE: LazyLock<PruneTable<DominoEdges>> = LazyLock::new(|| {
    let mut prune_table = PruneTable::from_iter([(EDGES_PHASE_1_GOAL.clone(), 0)]);
    prune_table.populate(&AlgSet::<Cube3x3>::from_iter(
        Cube3x3::ALL_MOVES.iter().map(|&m| vec![m]),
    ));
    prune_table
});

#[derive(Debug)]
pub struct Phase1;
impl Step<Cube3x3> for Phase1 {
    fn name(&self) -> &'static str {
        "Phase 1"
    }
    fn is_done(&self, puzzle: &Cube3x3) -> bool {
        Cube3x3::ALL_PIECES
            .iter()
            .filter(|&p| matches!(p, Pieces3x3::Corner(_)))
            .all(|&slot| puzzle.orientation_at(&slot) == Orientation3x3::Twist(Zn::ZERO))
            && Cube3x3::ALL_PIECES
                .iter()
                .filter(|&p| matches!(p, Pieces3x3::Edge(_)))
                .all(|&slot| puzzle.orientation_at(&slot) == Orientation3x3::Flip(Zn::ZERO))
            && RIM.iter().all(|p| RIM.contains(&puzzle.piece_location(p)))
    }
    fn solve(&self, puzzle: &mut Cube3x3) -> Result<crate::Solution<Cube3x3>, crate::StepError> {
        for depth in 0..20 {
            let puzzle_corners =
                Labeled::<Cube3x3, Tracked>::filter_by_piece(&puzzle, &CORNERS_PHASE_1_GOAL);
            let puzzle_edges =
                Labeled::<Cube3x3, Tracked>::filter_by_piece(&puzzle, &EDGES_PHASE_1_GOAL.0);
            let mut to_investigate = VecDeque::from([puzzle.clone()]);

            while let Some(state) = to_investigate.pop_front() {}
        }
        Err(StepError::UnreachableGoal)
    }
}

#[derive(Debug)]
pub struct Phase2;
impl Step<Cube3x3> for Phase2 {
    fn name(&self) -> &'static str {
        "Phase 2"
    }
    fn is_done(&self, puzzle: &Cube3x3) -> bool {
        Cube3x3::is_solved(puzzle)
    }
    fn solve(&self, puzzle: &mut Cube3x3) -> Result<crate::Solution<Cube3x3>, crate::StepError> {
        todo!()
    }
}

impl Method<Cube3x3> {
    #[must_use]
    pub fn kociemba() -> Self {
        Self::new("kociemba", vec![Arc::from(Phase1), Arc::from(Phase2)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corners_phase_1_prune_table_builds() {
        let table = CORNERS_PHASE_1_TABLE.clone();
    }
}
