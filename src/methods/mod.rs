pub mod choose;
pub mod cube3x3;
pub mod search_step;
#[cfg(test)]
mod test_steps;

use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    fmt::{Debug, Display},
    sync::Arc,
};

use crate::{ops::Inv, puzzles::Puzzle};

/// One stage of a solving method, such as building the first block in Roux.
///
/// A [`Method`] runs its steps in order. After each step it calls [`is_done`](Step::is_done),
/// and stops with a [`SolveError`] if the step failed or its goal is not met.
///
/// The crate has two step types: [`SearchStep`](crate::SearchStep) searches for its goal, and
/// [`Choose`](crate::Choose) runs several steps and keeps the shortest result. Implement this
/// trait to write another kind, such as a step that follows hand-written rules. A step is
/// `Send + Sync` so that one built step can be shared by many methods through an `Arc`.
pub trait Step<P: Puzzle>: Send + Sync + Debug {
    /// The step's name, as it appears in errors and in the [`Solution`] it returns.
    fn name(&self) -> &str;

    /// Whether `puzzle` meets this step's goal.
    fn is_done(&self, puzzle: &P) -> bool;

    /// Moves `puzzle` to a state that meets this step's goal and returns the moves it applied.
    ///
    /// The returned moves, applied to `puzzle` as it was, must give `puzzle` as this function
    /// leaves it. A printed [`Solution`] replays only if every step keeps this promise.
    ///
    /// # Errors
    /// A [`StepError`] when the step cannot start on `puzzle` or cannot reach its goal. The
    /// step may leave `puzzle` changed when it fails.
    fn solve(&self, puzzle: &mut P) -> Result<Solution<P>, StepError>;
}

/// The moves a method found, as one segment per step, in solving order.
///
/// Each segment is the moves a step applied and the name of the step. `Display` prints one line
/// per segment: the moves in notation, then a tab, `//`, and the step name. That text is valid
/// notation, so a scramble followed by its printed solution replays to the solved state:
///
/// ```text
/// F' Uw2 Rw Fw M' E' F2    //FB
/// U Rw2 U M' U2 Rw' U Rw2 U R    //SB
/// ```
///
/// Collect `(moves, name)` pairs to build one, or collect solutions to join them.
#[derive(Debug, PartialEq, Eq)]
pub struct Solution<P: Puzzle> {
    step_solutions: Vec<(Vec<P::Moves>, String)>,
}

impl<P: Puzzle> Solution<P> {
    /// A solution with no segments.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Each segment's moves and step name, in solving order.
    pub fn iter(&self) -> std::slice::Iter<'_, (Vec<P::Moves>, String)> {
        self.step_solutions.iter()
    }

    /// The number of moves in every segment, each move counting one whatever its part or
    /// modifier (slice turn metric). `M`, `Rw`, `y`, and `U2` each count one.
    #[must_use]
    pub fn move_count(&self) -> usize {
        self.step_solutions
            .iter()
            .map(|(moves, _)| moves.len())
            .sum()
    }
}

/// Builds a solution from `(moves, step name)` segments, in order.
impl<P: Puzzle, D: Display> FromIterator<(Vec<P::Moves>, D)> for Solution<P> {
    fn from_iter<I: IntoIterator<Item = (Vec<P::Moves>, D)>>(iter: I) -> Self {
        Self {
            step_solutions: iter
                .into_iter()
                .map(|(moves, name)| (moves, name.to_string()))
                .collect(),
        }
    }
}

/// Joins solutions into one, keeping every segment in order.
impl<P: Puzzle> FromIterator<Self> for Solution<P> {
    fn from_iter<I: IntoIterator<Item = Self>>(iter: I) -> Self {
        Self {
            step_solutions: iter.into_iter().flatten().collect(),
        }
    }
}

impl<P: Puzzle> Default for Solution<P> {
    fn default() -> Self {
        Self {
            step_solutions: Vec::<(Vec<P::Moves>, String)>::new(),
        }
    }
}
impl<'a, P: Puzzle> IntoIterator for &'a Solution<P> {
    type Item = &'a (Vec<<P as Puzzle>::Moves>, String);
    type IntoIter = std::slice::Iter<'a, (Vec<<P as Puzzle>::Moves>, String)>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<P: Puzzle> Extend<(Vec<P::Moves>, String)> for Solution<P> {
    fn extend<T: IntoIterator<Item = (Vec<P::Moves>, String)>>(&mut self, iter: T) {
        self.step_solutions.extend(iter);
    }
}
impl<P: Puzzle> IntoIterator for Solution<P> {
    type Item = (Vec<P::Moves>, String);
    type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.step_solutions.into_iter()
    }
}
impl<P: Puzzle> Display for Solution<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.step_solutions
                .iter()
                .map(|(move_sequence, name)| {
                    move_sequence
                        .iter()
                        .map(P::Moves::to_string)
                        .collect::<Vec<String>>()
                        .join(" ")
                        + "\t//"
                        + name
                        + "\n"
                })
                .collect::<String>()
        )
    }
}

/// A solving method: a name and an ordered list of [`Step`]s. Specific methods are constructors
/// of this struct.
///
/// [`Method::roux`] builds the Roux method for [`Cube3x3`](crate::Cube3x3). Any other list of
/// steps works too, and the steps can be of different types.
///
/// ```no_run
/// use rubiks_cube_lib::{Cube3x3, Method, Puzzle, RouxOptions};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut cube = Cube3x3::from_solved("R U R' F' L2 D B'")?;
/// let solution = Method::roux(RouxOptions::default()).solve(&mut cube)?;
/// assert!(cube.is_solved());
/// println!("{solution}");
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Method<P: Puzzle> {
    steps: Vec<Arc<dyn Step<P>>>,
    name: &'static str,
}

impl<P: Puzzle> Method<P> {
    /// The method's name, such as `"Roux"`.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// The method's steps, in solving order. Each item is a shared handle to the step the method
    /// holds, not a copy.
    pub fn steps(&self) -> impl Iterator<Item = Arc<dyn Step<P>>> {
        self.steps.iter().cloned()
    }

    /// A method that runs `steps` in the order given.
    ///
    /// A step that should keep its state across methods, such as a
    /// [`SearchStep`](crate::SearchStep) and its memo, can be shared by cloning its `Arc`.
    #[must_use]
    pub fn new(name: &'static str, steps: Vec<Arc<dyn Step<P>>>) -> Self {
        Self { steps, name }
    }

    /// Runs every step in order and joins their solutions: [`solve_steps`](Self::solve_steps),
    /// collected.
    ///
    /// # Errors
    /// The first [`SolveError`]. No later step runs, and `puzzle` is left as the failing step
    /// left it.
    pub fn solve(&self, puzzle: &mut P) -> Result<Solution<P>, SolveError> {
        self.solve_steps(puzzle).collect()
    }

    /// Solves `puzzle` one step at a time. Each call to `next()` runs the next step and yields
    /// its solution, so a caller can time a step or report progress before the next one runs.
    ///
    /// After each step the iterator checks the step's [`is_done`](Step::is_done). The first
    /// failure is yielded as an `Err`, and the iterator yields nothing after it. `puzzle` is
    /// left as the last step that ran left it.
    ///
    /// ```no_run
    /// use rubiks_cube_lib::{Cube3x3, Method, RouxOptions};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let roux = Method::roux(RouxOptions::default());
    /// let mut cube = Cube3x3::from_solved("R U R' F' L2 D B'")?;
    /// for segment in roux.solve_steps(&mut cube) {
    ///     print!("{}", segment?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn solve_steps(
        &self,
        puzzle: &mut P,
    ) -> impl Iterator<Item = Result<Solution<P>, SolveError>> {
        let mut failed = false;
        self.steps().map_while(move |step| {
            if failed {
                return None;
            }
            let result = match step.solve(puzzle) {
                Err(error) => Err(SolveError::Step {
                    step: step.name().to_string(),
                    error,
                }),
                Ok(_) if !step.is_done(puzzle) => Err(SolveError::NotDone {
                    step: step.name().to_string(),
                }),
                Ok(solution) => Ok(solution),
            };
            failed = result.is_err();
            Some(result)
        })
    }
}

/// Why a [`Step`] could not solve.
#[derive(Debug)]
pub enum StepError {
    /// The step's moves cannot bring the puzzle to its goal.
    UnreachableGoal,
    /// The puzzle does not meet what the step needs before it starts, such as a
    /// [`SearchStep`](crate::SearchStep)'s `before` mask. [`Choose`](crate::Choose) skips an
    /// alternative that returns this.
    InvalidStartingState,
    /// Another thread panicked while deepening the step's memo, so the memo may be inconsistent.
    MemoPoisoned,
    /// Any other error, for steps written outside the crate.
    Custom(Box<dyn Error + Send + Sync>),
}
/// Why a method's solve stopped, naming the step it stopped at.
#[derive(Debug)]
pub enum SolveError {
    /// The step reported an error.
    Step {
        /// The step's name.
        step: String,
        /// What the step reported.
        error: StepError,
    },
    /// The step returned a solution, but its `is_done` check failed.
    NotDone {
        /// The step's name.
        step: String,
    },
}

impl SolveError {
    /// The name of the step the solve stopped at.
    #[must_use]
    pub fn step(&self) -> String {
        match self {
            Self::Step { step, .. } | Self::NotDone { step } => step.clone(),
        }
    }
}

impl std::error::Error for StepError {}
impl std::error::Error for SolveError {}

impl Display for StepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::UnreachableGoal =>
                    "goal could not be reached with the given moveset".to_string(),
                Self::InvalidStartingState =>
                    "starting state doesn't fit expected properties".to_string(),
                Self::MemoPoisoned =>
                    "the step's memo is unusable: another solve panicked while deepening it"
                        .to_string(),
                Self::Custom(e) => e.to_string(),
            }
        )
    }
}
impl Display for SolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::NotDone { step } =>
                    format!("step {step} returned a solution, but its goal is not met"),
                Self::Step { step, error } => format!("step {step} could not finish: {error}"),
            }
        )
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::panic_in_result_fn,
        reason = "`?` reports setup failures; `assert!` reports the property under test failing"
    )]

    use std::error::Error;

    use crate::{
        Cube3x3, Edge, Mask, ParseSequenceError, Pieces3x3, SearchStep,
        puzzles::cube3by3::moves::Move3x3,
    };

    use super::{test_steps::FixedStep, *};

    fn fixed_failure() -> StepError {
        StepError::Custom("fixed failure".into())
    }

    #[test]
    fn move_count_counts_correctly() -> Result<(), Box<dyn Error>> {
        let s: Solution<Cube3x3> = Solution::from_iter([(
            Move3x3::sequence("y U2 r M'").collect::<Result<Vec<Move3x3>, ParseSequenceError>>()?,
            "Step 1",
        )]);
        assert_eq!(s.move_count(), 4);
        Ok(())
    }

    #[test]
    fn search_step_rejects_a_cube_that_misses_its_before_without_searching()
    -> Result<(), Box<dyn Error>> {
        let uf_solved = Mask::<Cube3x3>::new_from_pieces([Pieces3x3::Edge(Edge::Uf)]);
        // No moves: if the step searched, it could not move and would return `UnreachableGoal`.
        let step = SearchStep::new(
            "UF then DF",
            uf_solved,
            Mask::new_from_pieces([Pieces3x3::Edge(Edge::Uf), Pieces3x3::Edge(Edge::Df)]),
            Vec::new(),
        );
        let scrambled = Cube3x3::from_solved("U")?;
        let mut cube = scrambled;

        let result = step.solve(&mut cube);

        assert!(
            matches!(result, Err(StepError::InvalidStartingState)),
            "{result:?}"
        );
        assert_eq!(cube, scrambled, "a rejected step must not move the cube");
        Ok(())
    }

    #[test]
    fn a_step_that_returns_but_is_not_done_fails_the_solve_naming_it() {
        let method = Method::<Cube3x3>::new("test", vec![FixedStep::new("Never done", "", false)]);

        let result = method.solve(&mut Cube3x3::default());

        assert!(
            matches!(
                &result,
                Err(SolveError::NotDone { step }) if step == "Never done"
            ),
            "{result:?}"
        );
    }

    #[test]
    fn a_custom_step_error_comes_back_wrapped_and_naming_the_step() {
        let method = Method::<Cube3x3>::new(
            "test",
            vec![FixedStep::failing("Always fails", fixed_failure)],
        );

        let result = method.solve(&mut Cube3x3::default());

        let Err(SolveError::Step {
            step,
            error: StepError::Custom(custom),
        }) = result
        else {
            panic!("expected a wrapped custom error, got {result:?}");
        };
        assert_eq!(step, "Always fails");
        assert_eq!(custom.to_string(), "fixed failure");
    }

    #[test]
    fn each_next_runs_one_step_and_leaves_the_cube_after_it() -> Result<(), Box<dyn Error>> {
        let uf_solved = Mask::<Cube3x3>::new_from_pieces([Pieces3x3::Edge(Edge::Uf)]);
        let u_turns = Move3x3::sequence("U U' U2")
            .map(|m| m.map(|m| (vec![m], true)))
            .collect::<Result<Vec<_>, ParseSequenceError>>()?;
        let solve_uf = Arc::new(SearchStep::new(
            "UF",
            Mask::default(),
            uf_solved.clone(),
            u_turns,
        ));
        let later = FixedStep::new("Later", "", true);
        let method = Method::<Cube3x3>::new("test", vec![solve_uf, later.clone()]);
        let mut cube = Cube3x3::from_solved("U")?;

        let first = method.solve_steps(&mut cube).next();

        assert!(matches!(first, Some(Ok(_))), "{first:?}");
        assert!(
            uf_solved.applies_to(&cube),
            "the first step's goal is met:\n{cube}"
        );
        assert_eq!(
            later.runs(),
            0,
            "taking one item must not run the next step"
        );
        Ok(())
    }

    #[test]
    fn after_an_error_the_iterator_yields_nothing_more() {
        let failing = FixedStep::failing("Fails", fixed_failure);
        let later = FixedStep::new("Later", "", true);
        let method = Method::<Cube3x3>::new("test", vec![failing, later.clone()]);
        let mut cube = Cube3x3::default();
        let mut steps = method.solve_steps(&mut cube);

        assert!(matches!(steps.next(), Some(Err(_))));
        assert!(steps.next().is_none());
        assert!(steps.next().is_none());
        drop(steps);
        assert_eq!(later.runs(), 0, "no step runs after an error");
    }
}
