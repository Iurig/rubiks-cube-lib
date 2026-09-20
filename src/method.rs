use crate::Piece;
use crate::puzzle::{Move, Puzzle};

pub struct StepByPiece(&'static [crate::Corner]);

impl Step for StepByPiece {
    type Puzzle = crate::Cube3By3;
    fn solve(&self, scrambled_puzzle: Self::Puzzle) -> (Vec<Move<Self::Puzzle>>, String) {
        todo!()
    }
}

pub const do_all: StepByPiece = StepByPiece(crate::Corner::ALL);

pub trait Step {
    type Puzzle: Puzzle;
    fn solve(&self, scrambled_puzzle: Self::Puzzle) -> (Vec<Move<Self::Puzzle>>, String) {
        todo!()
    }
}

pub trait Method {
    type Puzzle: Puzzle;
    fn solve(
        &self,
        scrambled_puzzle: Self::Puzzle,
    ) -> impl Iterator<Item = (Vec<Move<Self::Puzzle>>, String)>;
}

#[derive(Clone)]
struct Solve<'a, P: Puzzle>(std::slice::Iter<'a, (Vec<Move<P>>, String)>);

impl<'a, P: Puzzle> Iterator for Solve<'a, P> {
    type Item = &'a (Vec<Move<P>>, String);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[derive(Clone)]
struct FormatingOptions {
    pub show_move_count: bool,
    pub show_name: bool,
}

impl Default for FormatingOptions {
    fn default() -> Self {
        Self {
            show_move_count: false,
            show_name: true,
        }
    }
}

trait ToRecon<P: Puzzle> {
    fn to_recon(&self) -> String;
}

impl<P: Puzzle> ToRecon<P> for Vec<Move<P>> {
    fn to_recon(&self) -> String {
        self.iter()
            .map(|m| m.to_string())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

impl<P: Puzzle> ToRecon<P> for (&(Vec<Move<P>>, String), FormatingOptions) {
    fn to_recon(&self) -> String {
        let mut moves = self.0.0.to_recon();
        let name = format!("\t\\{}", self.0.1);
        let move_count = self.0.0.len();
        if self.1.show_name || self.1.show_move_count {
            moves += "\t";
        }
        if self.1.show_name {
            moves += &name;
        }
        if self.1.show_move_count {
            moves += move_count.to_string().as_str();
        }
        moves
    }
}

impl<'a, P: Puzzle> ToRecon<P> for (Solve<'a, P>, FormatingOptions) {
    fn to_recon(&self) -> String {
        self.0
            .clone()
            .into_iter()
            .map(move |step| (step, self.1.clone()).to_recon())
            .collect::<Vec<String>>()
            .join("\n")
    }
}
impl<'a, P: Puzzle> ToRecon<P> for Solve<'a, P> {
    fn to_recon(&self) -> String {
        (self.clone(), FormatingOptions::default()).to_recon()
    }
}

pub struct SteppedMethod<const N: usize, S: Step>(pub [S; N]);

impl<P: Puzzle, S: Step<Puzzle = P>, const N: usize> Method for SteppedMethod<N, S> {
    type Puzzle = P;
    fn solve(
        &self,
        scrambled_puzzle: Self::Puzzle,
    ) -> impl Iterator<Item = (Vec<Move<P>>, String)> {
        self.0
            .iter()
            .map(move |step| step.solve(scrambled_puzzle.clone()))
    }
}
/*
struct FlowMethod<P: Puzzle>([(FlowStep<P>, bool)]);

struct FlowStep<P: Puzzle> {
    required: Vec<dyn Piece<P>>,
    step: Box<dyn Fn(P) -> (Vec<Move<P>>, String)>,
    promissed: Vec<dyn Piece<P>>,
}

impl<P: Puzzle> Method<P> for FlowMethod<P> {
    fn solve(&self) -> impl Iterator<Item = (Vec<Move<P>>, String)> {
        todo!("return")
    }
}*/
