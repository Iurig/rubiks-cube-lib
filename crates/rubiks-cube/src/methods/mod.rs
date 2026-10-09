pub mod cube3x3;
mod solution;
pub mod step;
#[cfg(test)]
mod test_steps;

pub use solution::{Segment, Solution};

use std::{fmt::Debug, ops::Index, sync::Arc};

use crate::{SolveError, methods::step::Step, puzzles::Puzzle};

/// The ordered list of [`Step`]s that a [`Method`] runs, built by
/// [`to_technique`](Method::to_technique) from the method's options.
///
/// [`Roux`](crate::Roux) gives the Roux steps for [`Cube3x3`](crate::Cube3x3). Any other list
/// of steps works too, and the steps can be of different types.
///
/// ```rust
/// use rubiks_cube::{Cube3x3, Method, Puzzle, Roux};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut cube = Cube3x3::from_moves("R U R' F' L2 D B'")?;
/// let solution = Roux::default().solve(&mut cube)?;
/// assert!(cube.is_solved());
/// println!("{solution}");
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct Technique<P: Puzzle> {
    steps: Vec<Arc<dyn Step<P>>>,
}

impl<P: Puzzle> Index<usize> for Technique<P> {
    type Output = dyn Step<P>;
    #[expect(
        clippy::indexing_slicing,
        reason = "implementing indexing is supposed to be able to panic"
    )]
    fn index(&self, index: usize) -> &Self::Output {
        &*self.steps[index]
    }
}

impl<P: Puzzle> FromIterator<Arc<dyn Step<P>>> for Technique<P> {
    fn from_iter<T: IntoIterator<Item = Arc<dyn Step<P>>>>(iter: T) -> Self {
        Self {
            steps: iter.into_iter().collect(),
        }
    }
}

impl<P: Puzzle> IntoIterator for Technique<P> {
    type IntoIter = <Vec<Arc<dyn Step<P>>> as IntoIterator>::IntoIter;
    type Item = Arc<dyn Step<P>>;
    fn into_iter(self) -> Self::IntoIter {
        self.steps.into_iter()
    }
}

impl<P: Puzzle> Technique<P> {
    /// Constructor for a technique: takes in any
    pub fn new(steps: impl IntoIterator<Item = Arc<dyn Step<P>>>) -> Self {
        Self {
            steps: steps.into_iter().collect(),
        }
    }

    /// # Errors
    /// Errors if any step fails to start, to complete, or if it completes but isn't solved at the
    /// end.
    pub fn solve(&self, puzzle: &mut P) -> Result<Solution<P>, SolveError> {
        self.solve_steps(puzzle).collect()
    }

    /// Returns the steps of the Technique as an [`Iterator`](core::iter::Iterator).
    pub fn steps(&self) -> impl Iterator<Item = &dyn Step<P>> {
        self.steps.iter().map(|s| &**s)
    }

    /// Returns an [`Iterator`](core::iter::Iterator) over the solutions of every step which
    /// iterating over solves the [`puzzle`](crate::Puzzle).
    pub fn solve_steps(
        &self,
        puzzle: &mut P,
    ) -> impl Iterator<Item = Result<Solution<P>, SolveError>> {
        run_steps(self.steps.iter().cloned(), puzzle)
    }
}

/// Runs `steps` in order on `puzzle`, one per `next()`. Before each step it checks
/// [`can_solve`](Step::can_solve), after it [`is_done`](Step::is_done), and it yields nothing
/// after the first error.
fn run_steps<P: Puzzle>(
    steps: impl IntoIterator<Item = Arc<dyn Step<P>>>,
    puzzle: &mut P,
) -> impl Iterator<Item = Result<Solution<P>, SolveError>> {
    let mut failed = false;
    steps.into_iter().map_while(move |step| {
        if failed {
            return None;
        }
        if !step.can_solve(puzzle) {
            failed = true;
            return Some(Err(SolveError::Requirements {
                step: step.name().to_string(),
            }));
        }
        let result = match step.solve(puzzle) {
            Err(error) => Err(SolveError::Step {
                step: step.name().to_string(),
                source: error,
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

/// The main Method trait, implemented by specifying a conversion to [`Technique`] (a sequence of
/// steps) through [`to_technique`](Method::to_technique).
///
/// It can then be solved directly without conversion by using [`self.solve()`](Method::solve).
/// Methods in real life aren't a simple sequence of steps, but something that, depending on
/// parameters, collapses to a different [`Technique`]. Implementation of [`Method`] usually starts
/// by defining a type to hold such parameters, then how to get a sequence of steps from such
/// parameters.
pub trait Method<P: Puzzle>: Debug {
    /// Converts a parametrized [`Method`] to a specific [`Technique`]
    fn to_technique(&self) -> Technique<P>;

    /// Runs every step in order and joins their solutions: [`solve_steps`](Self::solve_steps),
    /// collected.
    ///
    /// # Errors
    /// The first [`SolveError`]. No later step runs, and `puzzle` is left as the failing step
    /// left it.
    fn solve(&self, puzzle: &mut P) -> Result<Solution<P>, SolveError>
    where
        Self: Sized,
    {
        self.solve_steps(puzzle).collect()
    }

    /// Solves `puzzle` one step at a time. Each call to `next()` runs the next step and yields
    /// its solution, so a caller can time a step or report progress before the next one runs.
    ///
    /// After each step the iterator checks the step's [`is_done`](Step::is_done). The first
    /// failure is yielded as an `Err`, and the iterator yields nothing after it. `puzzle` is
    /// left as the last step that ran left it.
    ///
    /// ```rust
    /// use rubiks_cube::{Cube3x3, Method, Roux};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let roux = Roux::default();
    /// let mut cube = Cube3x3::from_moves("R U R' F' L2 D B'")?;
    /// for segment in roux.solve_steps(&mut cube) {
    ///     print!("{}", segment?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn solve_steps(&self, puzzle: &mut P) -> impl Iterator<Item = Result<Solution<P>, SolveError>>
    where
        Self: Sized,
    {
        run_steps(self.to_technique().steps, puzzle)
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::panic_in_result_fn,
        reason = "`?` reports setup failures; `assert!` reports the property under test failing"
    )]

    use std::error::Error;

    use crate::{Cube3x3, Edge, Piece3x3, PieceSet, SearchStep, StepError};

    use super::{test_steps::FixedStep, *};
    use crate::AlgSet;

    fn fixed_failure() -> StepError {
        StepError::Custom("fixed failure".into())
    }

    #[test]
    fn a_method_step_that_returns_but_is_not_done_fails_the_solve_naming_it() {
        let never_done: Arc<dyn Step<Cube3x3>> =
            Arc::new(FixedStep::new("Never done", "", true, false));
        let method = Technique::new([never_done]);

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
    fn after_a_step_that_cannot_start_the_iterator_yields_nothing_more() {
        let later = FixedStep::new("Later", "", true, true);
        let method = Technique::new([
            Arc::new(FixedStep::new("Cannot start", "", false, true)) as Arc<dyn Step<Cube3x3>>,
            Arc::new(later.clone()),
        ]);
        let mut cube = Cube3x3::default();
        let mut steps = method.solve_steps(&mut cube);

        assert!(
            matches!(
                steps.next(),
                Some(Err(SolveError::Requirements { step })) if step == "Cannot start"
            ),
            "the step that cannot start is reported"
        );
        assert!(steps.next().is_none());
        assert_eq!(later.runs(), 0, "no step runs after an error");
    }

    #[test]
    fn search_step_rejects_a_cube_that_misses_its_before_without_searching()
    -> Result<(), Box<dyn Error>> {
        let uf_solved = PieceSet::<Cube3x3>::from_pieces([Piece3x3::Edge(Edge::Uf)]);
        // No moves: if the step searched, it could not move and would return `UnreachableGoal`.
        let step = SearchStep::builder(
            "UF then DF",
            PieceSet::<Cube3x3>::from_pieces([Piece3x3::Edge(Edge::Uf), Piece3x3::Edge(Edge::Df)]),
        )
        .expect_solved(uf_solved)
        .build()?;
        let scrambled = Cube3x3::from_moves("U")?;
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
        let never_done: Arc<dyn Step<Cube3x3>> =
            Arc::new(FixedStep::new("Never done", "", true, false));
        let method = Technique::new([never_done]);

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
        let always_fails: Arc<dyn Step<Cube3x3>> =
            FixedStep::failing("Always fails", fixed_failure);
        let method = Technique::new([always_fails]);

        let result = method.solve(&mut Cube3x3::default());

        let Err(SolveError::Step {
            step,
            source: StepError::Custom(custom),
        }) = result
        else {
            panic!("expected a wrapped custom error, got {result:?}");
        };
        assert_eq!(step, "Always fails");
        assert_eq!(custom.to_string(), "fixed failure");
    }

    #[test]
    fn each_next_runs_one_step_and_leaves_the_cube_after_it() -> Result<(), Box<dyn Error>> {
        let uf_solved = PieceSet::<Cube3x3>::from_pieces([Piece3x3::Edge(Edge::Uf)]);
        let u_turns = AlgSet::from_parts("U")?;
        let solve_uf = SearchStep::builder("UF", uf_solved.clone())
            .search_algs(u_turns)
            .build()?;
        let later = FixedStep::new("Later", "", true, true);
        let method = Technique::new([
            Arc::new(solve_uf) as Arc<dyn Step<Cube3x3>>,
            Arc::new(later.clone()),
        ]);
        let mut cube = Cube3x3::from_moves("U")?;

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
        let later = FixedStep::new("Later", "", true, true);
        let method = Technique::new([failing as Arc<dyn Step<Cube3x3>>, Arc::new(later.clone())]);
        let mut cube = Cube3x3::default();
        let mut steps = method.solve_steps(&mut cube);

        assert!(matches!(steps.next(), Some(Err(_))));
        assert!(steps.next().is_none());
        assert!(steps.next().is_none());
        assert_eq!(later.runs(), 0, "no step runs after an error");
    }
}
