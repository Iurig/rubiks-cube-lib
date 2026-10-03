use std::sync::{Arc, LazyLock};

use crate::{
    AlgSet, Algorithm, ByPiece, Cube3x3, Edge, Labeled, Method, Pieces3x3, Puzzle, Tracked,
    methods::combine_pruned::{DistanceStep, PruneTable, PrunedCombine, PrunedGoal},
    puzzles::cube3x3::{
        moves::{MovablePart, MoveModifier},
        pieces::Faces,
    },
};

/// The four E-slice edges. Phase 1 brings them into the E slice; phase 2 puts them in place.
const RIM: [Pieces3x3; 4] = [
    Pieces3x3::Edge(Edge::Fl),
    Pieces3x3::Edge(Edge::Fr),
    Pieces3x3::Edge(Edge::Bl),
    Pieces3x3::Edge(Edge::Br),
];

/// Every face turn, one per sequence: phase 1's moveset. `2'` is left out, since on a face it
/// reaches the same state as `2` and would only double the branching.
fn face_turns() -> AlgSet<Cube3x3> {
    Cube3x3::ALL_MOVES
        .iter()
        .filter(|m| {
            matches!(m.part, MovablePart::Face(_)) && m.modifier != MoveModifier::CounterDouble
        })
        .map(|&m| Algorithm::from_iter([m]))
        .collect()
}

/// Phase 2's moveset, the moves that keep a cube in the domino subgroup: any turn of U or D,
/// and half turns of the other faces.
fn domino_turns() -> AlgSet<Cube3x3> {
    Cube3x3::ALL_MOVES
        .iter()
        .filter(|m| match m.part {
            MovablePart::Face(Faces::U | Faces::D) => m.modifier != MoveModifier::CounterDouble,
            MovablePart::Face(_) => m.modifier == MoveModifier::Double,
            _ => false,
        })
        .map(|&m| Algorithm::from_iter([m]))
        .collect()
}

fn corners() -> impl Iterator<Item = Pieces3x3> + Clone {
    Cube3x3::ALL_PIECES
        .iter()
        .filter(|&p| matches!(p, Pieces3x3::Corner(_)))
        .copied()
}

fn edges() -> impl Iterator<Item = Pieces3x3> + Clone {
    Cube3x3::ALL_PIECES
        .iter()
        .filter(|&p| matches!(p, Pieces3x3::Edge(_)))
        .copied()
}

static CORNERS_PHASE_1_GOAL: LazyLock<Labeled<Cube3x3, Tracked>> =
    LazyLock::new(|| Labeled::<Cube3x3, Tracked>::from_double_iter([], corners()));

static CORNERS_PHASE_1_TABLE: LazyLock<PruneTable<Labeled<Cube3x3, Tracked>>> =
    LazyLock::new(|| PruneTable::from_goal(&CORNERS_PHASE_1_GOAL, &face_turns()));

static EDGES_PHASE_1_GOAL: LazyLock<Labeled<Cube3x3, Tracked>> =
    LazyLock::new(|| Labeled::<Cube3x3, Tracked>::from_double_iter(RIM, edges()));

static EDGES_PHASE_1_TABLE: LazyLock<PruneTable<Labeled<Cube3x3, Tracked>>> =
    LazyLock::new(|| PruneTable::from_goal(&EDGES_PHASE_1_GOAL, &face_turns()));

#[derive(Debug)]
struct Phase1Corners;
impl PrunedGoal<Cube3x3> for Phase1Corners {
    type Marker = Tracked;
    fn goal(&self) -> &Labeled<Cube3x3, Tracked> {
        &CORNERS_PHASE_1_GOAL
    }
    fn table(&self) -> &PruneTable<Labeled<Cube3x3, Tracked>> {
        &CORNERS_PHASE_1_TABLE
    }
}

#[derive(Debug)]
struct Phase1Edges;
impl PrunedGoal<Cube3x3> for Phase1Edges {
    type Marker = Tracked;
    fn goal(&self) -> &Labeled<Cube3x3, Tracked> {
        &EDGES_PHASE_1_GOAL
    }
    fn table(&self) -> &PruneTable<Labeled<Cube3x3, Tracked>> {
        &EDGES_PHASE_1_TABLE
    }
}

/// Phase 1: orient every piece and bring the E-slice edges into the E slice, with face turns.
fn phase_1() -> PrunedCombine<'static, Cube3x3> {
    PrunedCombine::<Cube3x3>::new(
        "Phase 1",
        [
            Box::new(Phase1Corners) as Box<dyn DistanceStep<Cube3x3>>,
            Box::new(Phase1Edges) as Box<dyn DistanceStep<Cube3x3>>,
        ],
        face_turns(),
    )
}

// Phase 2's goals use `ByPiece`, because they must tell the pieces of a group apart: `Tracked`
// would only say which slots hold them. As in Kociemba's own solver, each goal pairs a group of
// pieces with the E-slice edges, so one table knows how both interact: 8! * 4! = 967680 entries
// each. Separate tables for the three groups give a lower bound only as good as the worst-placed
// group, which leaves the search to explore far more states.

static CORNERS_AND_E_PHASE_2_GOAL: LazyLock<Labeled<Cube3x3, ByPiece>> =
    LazyLock::new(|| Labeled::<Cube3x3, ByPiece>::new_from_pieces(corners().chain(RIM)));

static CORNERS_AND_E_PHASE_2_TABLE: LazyLock<PruneTable<Labeled<Cube3x3, ByPiece>>> =
    LazyLock::new(|| PruneTable::from_goal(&CORNERS_AND_E_PHASE_2_GOAL, &domino_turns()));

static EDGES_PHASE_2_GOAL: LazyLock<Labeled<Cube3x3, ByPiece>> =
    LazyLock::new(|| Labeled::<Cube3x3, ByPiece>::new_from_pieces(edges()));

static EDGES_PHASE_2_TABLE: LazyLock<PruneTable<Labeled<Cube3x3, ByPiece>>> =
    LazyLock::new(|| PruneTable::from_goal(&EDGES_PHASE_2_GOAL, &domino_turns()));

#[derive(Debug)]
struct Phase2CornersAndE;
impl PrunedGoal<Cube3x3> for Phase2CornersAndE {
    type Marker = ByPiece;
    fn goal(&self) -> &Labeled<Cube3x3, ByPiece> {
        &CORNERS_AND_E_PHASE_2_GOAL
    }
    fn table(&self) -> &PruneTable<Labeled<Cube3x3, ByPiece>> {
        &CORNERS_AND_E_PHASE_2_TABLE
    }
}

/// The U/D-layer edges with the E-slice edges, which is every edge.
#[derive(Debug)]
struct Phase2Edges;
impl PrunedGoal<Cube3x3> for Phase2Edges {
    type Marker = ByPiece;
    fn goal(&self) -> &Labeled<Cube3x3, ByPiece> {
        &EDGES_PHASE_2_GOAL
    }
    fn table(&self) -> &PruneTable<Labeled<Cube3x3, ByPiece>> {
        &EDGES_PHASE_2_TABLE
    }
}

/// Phase 2: solve a cube in the domino subgroup, with only the moves that keep it there. A cube
/// outside the subgroup gets [`StepError::UnreachableGoal`](crate::StepError::UnreachableGoal),
/// since every goal asks for oriented pieces and these moves never change orientation.
fn phase_2() -> PrunedCombine<'static, Cube3x3> {
    PrunedCombine::<Cube3x3>::new(
        "Phase 2",
        [
            Box::new(Phase2CornersAndE) as Box<dyn DistanceStep<Cube3x3>>,
            Box::new(Phase2Edges) as Box<dyn DistanceStep<Cube3x3>>,
        ],
        domino_turns(),
    )
}

impl Method<Cube3x3> {
    /// Kociemba's two-phase method: phase 1 orients every piece and brings the E-slice edges
    /// into the E slice using face turns, then phase 2 solves using only `U`, `D`, and half
    /// turns. The first call builds the pruning tables, which takes a few seconds.
    #[must_use]
    pub fn kociemba() -> Self {
        Self::new("kociemba", vec![Arc::new(phase_1()), Arc::new(phase_2())])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Orientation3x3, zn::Zn};

    /// Phase 1's goal written from the definition, independent of the prune tables: every
    /// corner twist and edge flip is zero, and the four E-slice edges sit in E-slice slots.
    fn in_domino_subgroup(puzzle: &Cube3x3) -> bool {
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

    #[test]
    fn domino_turns_are_u_and_d_turns_and_other_half_turns() {
        let names: Vec<String> = domino_turns()
            .algs()
            .iter()
            .map(|alg| alg.iter().map(ToString::to_string).collect())
            .collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(
            sorted,
            ["B2", "D", "D'", "D2", "F2", "L2", "R2", "U", "U'", "U2"],
            "{names:?}"
        );
    }

    #[test]
    fn domino_turns_keep_a_cube_in_the_domino_subgroup() {
        let mut cube = Cube3x3::default();
        for alg in domino_turns().algs() {
            cube = alg.iter().fold(cube, |c, &m| c * m);
            assert!(
                in_domino_subgroup(&cube),
                "after {alg:?}
{cube}"
            );
        }
    }
}
