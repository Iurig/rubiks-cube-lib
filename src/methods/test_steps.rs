//! Hand-written steps for tests of the solve loop and of steps that hold other steps.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use crate::{Cube3x3, Solution, Step, StepError, puzzles::cube3by3::moves::Move3x3};

/// A step whose result is fixed in advance. `solve` either returns the error from `error`,
/// or applies `moves` to the cube and returns them as one segment named after the step.
/// `is_done` reports `done`, whatever the cube. It counts how many times `solve` ran.
#[derive(Debug)]
pub struct FixedStep {
    name: &'static str,
    moves: &'static str,
    done: bool,
    error: Option<fn() -> StepError>,
    runs: AtomicUsize,
}

impl FixedStep {
    /// A step that applies `moves` (notation text; `""` for none) and reports `done`.
    pub fn new(name: &'static str, moves: &'static str, done: bool) -> Arc<Self> {
        Arc::new(Self {
            name,
            moves,
            done,
            error: None,
            runs: AtomicUsize::new(0),
        })
    }

    /// A step whose `solve` always returns `error()` and leaves the cube alone.
    pub fn failing(name: &'static str, error: fn() -> StepError) -> Arc<Self> {
        Arc::new(Self {
            name,
            moves: "",
            done: false,
            error: Some(error),
            runs: AtomicUsize::new(0),
        })
    }

    /// How many times `solve` ran.
    pub fn runs(&self) -> usize {
        self.runs.load(Ordering::Relaxed)
    }
}

impl Step<Cube3x3> for FixedStep {
    fn name(&self) -> &str {
        self.name
    }

    fn is_done(&self, _: &Cube3x3) -> bool {
        self.done
    }

    fn solve(&self, puzzle: &mut Cube3x3) -> Result<Solution<Cube3x3>, StepError> {
        self.runs.fetch_add(1, Ordering::Relaxed);
        if let Some(error) = self.error {
            return Err(error());
        }
        let moves = Move3x3::sequence(self.moves)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| StepError::Custom(Box::new(e)))?;
        *puzzle = moves.iter().fold(*puzzle, |cube, m| cube * *m);
        Ok(Solution::from_iter([(moves, self.name)]))
    }
}
