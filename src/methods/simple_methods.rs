#[allow(clippy::wildcard_imports)]
use crate::{
    Cube3x3,
    methods::*,
    puzzles::cube3by3::{moves::Move3x3, pieces::Pieces3x3},
};

#[derive(Clone, Debug)]
pub struct NamedMoveSequence3x3(String, Vec<Move3x3>);
#[derive(Debug)]
pub struct NamedSolution3x3(Vec<NamedMoveSequence3x3>);
#[derive(Clone)]
pub struct SimpleStep3x3 {
    pub name: String,
    pub before: Box<[Pieces3x3]>,
    pub after: Box<[Pieces3x3]>,
    pub allowed: Vec<Vec<Move3x3>>,
}
#[derive(Default)]
pub struct NoOptions();
pub struct SimpleMethod3x3(pub Vec<SimpleStep3x3>);

impl Solution for NamedSolution3x3 {
    type ReconOptions = NoOptions;
    fn new() -> Self {
        Self(Vec::new())
    }
    fn recon_with_options(&self, _: Self::ReconOptions) -> String {
        self.0
            .iter()
            .map(|NamedMoveSequence3x3(name, move_sequence)| {
                move_sequence
                    .iter()
                    .map(Move3x3::to_string)
                    .collect::<Vec<String>>()
                    .join(" ")
                    + "\t//"
                    + name
                    + "\n"
            })
            .collect::<String>()
    }
    fn then(&self, next: Self) -> Self {
        let mut concatenation = self.0.clone();
        concatenation.extend(next.0);
        Self(concatenation)
    }
}

impl SolveStep<Cube3x3, NamedSolution3x3, SimpleMethod3x3> for SimpleStep3x3 {
    fn can_apply(&self, cube: &Cube3x3) -> bool {
        (*self.before)
            .iter()
            .all(|piece| cube.piece_at(piece) == *piece && cube.orientation_at(piece) == 0)
    }
    fn options_allow(
        &self,
        _options: &<SimpleMethod3x3 as SolveMethod<Cube3x3, NamedSolution3x3>>::MethodOptions,
    ) -> bool {
        true
    }
    fn step_is_solved(&self, cube: &Cube3x3) -> bool {
        (*self.after)
            .iter()
            .all(|piece| cube.piece_at(piece) == *piece && cube.orientation_at(piece) == 0)
    }
    fn step_name(&self) -> String {
        self.name.clone()
    }
    fn allowed_move_sequences(&self) -> Vec<Vec<<Cube3x3 as Puzzle>::Moves>> {
        self.allowed.clone()
    }

    type PartialCube = Vec<(Pieces3x3, usize)>;
    fn mask(&self, puzzle: &Cube3x3) -> Self::PartialCube {
        self.after
            .iter()
            .map(|p| {
                (
                    puzzle.piece_location(p),
                    puzzle.orientation_at(&puzzle.piece_location(p)),
                )
            })
            .collect::<Vec<(Pieces3x3, usize)>>()
    }
    fn solve(&self, p: &mut Cube3x3) -> NamedSolution3x3 {
        NamedSolution3x3(vec![NamedMoveSequence3x3(
            self.step_name(),
            self.solve_bfs(p),
        )])
    }
    /*
    #[expect(clippy::panic)]
    fn solve(&self, p: &mut Cube3x3) -> NamedSolution3x3 {
        let mut to_investigate = VecDeque::from([(*p, None, 0)]);
        let mut investigated: HashMap<Self::PartialCube, Option<Vec<Move3x3>>> = HashMap::new();
        let mut prev_depth = 0;

        while let Some((current_cube, current_move, depth)) = to_investigate.pop_front() {
            if investigated.contains_key(&self.mask(&current_cube)) {
                continue;
            }
            investigated.insert(self.mask(&current_cube), current_move);
            for m in self.allowed.clone() {
                let moved_cube = m.iter().fold(current_cube, |c, m| c * *m);
                if self.step_is_solved(&moved_cube) {
                    *p = moved_cube;
                    let mut solution = VecDeque::from([m]);
                    let mut cube = current_cube;
                    while let Some(backtracking_move) = investigated
                        .get(&self.mask(&cube))
                        .expect("previously investigated cube was not found")
                    {
                        solution.push_front(backtracking_move.clone());
                        cube = backtracking_move
                            .iter()
                            .rev()
                            .map(|&m| m.inverse())
                            .fold(cube, |c, m| c * m);
                    }
                    return NamedSolution3x3(vec![NamedMoveSequence3x3(
                        self.name.clone(),
                        solution.iter().flatten().copied().collect(),
                    )]);
                }
                to_investigate.push_back((moved_cube, Some(m), depth + 1));
            }
            if depth != prev_depth {
                println!("Step: {}\t Depth:{depth}", self.name);
                prev_depth = depth;
            }
        }
        panic!("Step is unsolvable");
    }*/
}

impl SolveMethod<Cube3x3, NamedSolution3x3> for SimpleMethod3x3 {
    type MethodOptions = NoOptions;

    fn steps(&self) -> Vec<impl SolveStep<Cube3x3, NamedSolution3x3, Self>> {
        self.0.clone()
    }
}
