use std::sync::Arc;

use crate::{Puzzle, Solution, Step, StepError};

/// A step that tries several steps and keeps the solution with the fewest moves.
///
/// Each alternative solves its own copy of the puzzle. An alternative that returns
/// [`StepError::InvalidStartingState`] is skipped, so `Choose` also picks whichever
/// alternatives can start at all. Any other error ends the whole choice. On a tie, the earlier
/// alternative wins. The solution keeps the name of the alternative that ran, and the puzzle is
/// left as that alternative left it.
///
/// Roux uses it to build either the front or the back square of a block, and then the pair
/// that square leaves.
///
/// [`is_done`](Step::is_done) holds when any alternative's `is_done` holds.
#[derive(Debug)]
pub struct Choose<P: Puzzle> {
    steps: Vec<Arc<dyn Step<P>>>,
    name: String,
}

impl<P: Puzzle> Choose<P> {
    /// A choice between `steps`, named `name`. The name appears in errors about the choice
    /// itself. When an alternative runs, the solution names that alternative instead.
    pub fn named(name: impl Into<String>, steps: Vec<Arc<dyn Step<P>>>) -> Self {
        Self {
            steps,
            name: name.into(),
        }
    }
}

impl<P: Puzzle> Step<P> for Choose<P> {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn solve(&self, puzzle: &mut P) -> Result<Solution<P>, StepError> {
        let best = self
            .steps
            .iter()
            .map(|s| {
                let mut candidate = puzzle.clone();
                s.solve(&mut candidate)
                    .map(|solution| (solution, candidate))
            })
            .try_fold(
                None,
                |best: Option<(Solution<P>, P)>, candidate| match candidate {
                    Err(StepError::InvalidStartingState) => Ok(best),
                    Err(
                        e @ (StepError::MemoPoisoned
                        | StepError::UnreachableGoal
                        | StepError::Custom(_)),
                    ) => Err(e),
                    // On a tie the earlier alternative stays.
                    Ok(candidate) => Ok(match best {
                        Some(best) if best.0.move_count() <= candidate.0.move_count() => Some(best),
                        _ => Some(candidate),
                    }),
                },
            )?;
        let (solution, solved) = best.ok_or(StepError::InvalidStartingState)?;
        *puzzle = solved;
        Ok(solution)
    }

    fn is_done(&self, puzzle: &P) -> bool {
        self.steps.iter().any(|s| s.is_done(puzzle))
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::panic_in_result_fn,
        reason = "`?` reports setup failures; `assert!` reports the property under test failing"
    )]

    use std::error::Error;

    use super::*;
    use crate::{Cube3x3, methods::test_steps::FixedStep};

    fn cannot_start() -> StepError {
        StepError::InvalidStartingState
    }

    fn unreachable() -> StepError {
        StepError::UnreachableGoal
    }

    #[test]
    fn the_shorter_alternative_is_kept_and_the_longer_leaves_no_trace() -> Result<(), Box<dyn Error>>
    {
        // The longer one comes first, so the win is not just list order.
        let choose = Choose::named(
            "Either",
            vec![
                FixedStep::new("Long", "R U", true),
                FixedStep::new("Short", "R", true),
            ],
        );
        let mut cube = Cube3x3::default();

        let solution = choose.solve(&mut cube)?;

        assert_eq!(
            cube,
            Cube3x3::from_solved("R")?,
            "only the short moves are applied"
        );
        assert_eq!(solution.move_count(), 1);
        let names: Vec<&str> = solution.iter().map(|(_, name)| name.as_str()).collect();
        assert_eq!(
            names,
            ["Short"],
            "the segment is named after the step that ran"
        );
        Ok(())
    }

    #[test]
    fn an_alternative_that_cannot_start_is_skipped() -> Result<(), Box<dyn Error>> {
        let choose = Choose::named(
            "Either",
            vec![
                FixedStep::failing("Cannot start", cannot_start),
                FixedStep::new("Runs", "U", true),
            ],
        );
        let mut cube = Cube3x3::default();

        let solution = choose.solve(&mut cube)?;

        assert_eq!(cube, Cube3x3::from_solved("U")?);
        let names: Vec<&str> = solution.iter().map(|(_, name)| name.as_str()).collect();
        assert_eq!(names, ["Runs"]);
        Ok(())
    }

    #[test]
    fn any_other_error_ends_the_choice_even_if_another_alternative_succeeds() {
        let choose = Choose::named(
            "Either",
            vec![
                FixedStep::failing("Broken", unreachable),
                FixedStep::new("Would run", "U", true),
            ],
        );
        let mut cube = Cube3x3::default();

        let result = choose.solve(&mut cube);

        assert!(
            matches!(result, Err(StepError::UnreachableGoal)),
            "{result:?}"
        );
        assert_eq!(
            cube,
            Cube3x3::default(),
            "a failed choice leaves the cube alone"
        );
    }

    #[test]
    fn when_no_alternative_can_start_the_choice_cannot_start() {
        let choose = Choose::named(
            "Either",
            vec![
                FixedStep::failing("First", cannot_start),
                FixedStep::failing("Second", cannot_start),
            ],
        );
        let mut cube = Cube3x3::default();

        let result = choose.solve(&mut cube);

        assert!(
            matches!(result, Err(StepError::InvalidStartingState)),
            "{result:?}"
        );
        assert_eq!(cube, Cube3x3::default());
    }
}
