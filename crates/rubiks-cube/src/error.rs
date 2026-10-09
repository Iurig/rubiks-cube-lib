use std::error::Error;

use thiserror::Error;

use crate::{PieceSet, Puzzle};

/// Why a [`Step`](crate::Step) could not solve.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StepError {
    /// The step's moves cannot bring the puzzle to its goal.
    #[error("goal could not be reached with the given algset")]
    UnreachableGoal,
    /// The puzzle does not meet what the step needs before it starts, such as a
    /// [`SearchStep`](crate::SearchStep)'s `before` mask. [`Shortest`](crate::Shortest) skips an
    /// alternative that returns this.
    #[error("starting state doesn't fit expected properties")]
    InvalidStartingState,
    /// Another thread panicked while deepening the step's memo, so the memo may be inconsistent.
    #[error("the step's memo is unusable: another solve panicked while deepening it")]
    MemoPoisoned,
    #[error("{}", *.0)]
    /// Any other error, for steps written outside the crate.
    Custom(#[source] Box<dyn Error + Send + Sync>),
}
/// Why a method's solve stopped, naming the step it stopped at.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SolveError {
    /// A step's requirement isn't met when it should be solved.
    #[error("step {step} couldn't start solving because its requirements were not met")]
    Requirements {
        /// The step's name.
        step: String,
    },
    /// The step reported an error.
    #[error("step {step} could not finish: {source}")]
    Step {
        /// The step's name.
        step: String,
        /// What the step reported.
        source: StepError,
    },
    /// The step returned a solution, but its `is_done` check failed.
    #[error("step {step} returned a solution, but its goal is not met")]
    NotDone {
        /// The step's name.
        step: String,
    },
}

/// Why one move failed to parse.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ParseMoveError {
    /// Reachable only through `"".parse::<Move3x3>()`; a sequence parse never tries to parse an
    /// empty move, so it carries no offending text.
    #[error("empty string cannot be parsed into moves")]
    EmptyString,
    /// The text after the part is not a modifier.
    #[error("{modifier} is not a valid modifier in {invalid_move}")]
    BadModifier {
        /// The whole invalid move.
        invalid_move: String,
        /// The text that failed as a modifier.
        modifier: String,
    },
    /// The text before the modifier is not a part.
    #[error("{part} in {invalid_move} is not a valid part")]
    BadPart {
        /// The whole invalid move.
        invalid_move: String,
        /// The text that failed as a part.
        part: String,
    },
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
/// A move in a sequence failed to parse; `line` and `position` count from 1.
#[error("at line {line}, position {position}: {source}")]
pub struct ParseSequenceError {
    pub(crate) source: ParseMoveError,
    pub(crate) line: usize,
    pub(crate) position: usize,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
/// Attempted to construct a [`SearchStep`](crate::SearchStep) with invalid
/// [`SearchStepBuilder`](crate::SearchStepBuilder).
#[non_exhaustive]
pub enum SearchStepError<P: Puzzle> {
    /// The step prerequisite was not guaranteed by the goal.
    #[error("the pieces expected solved are not all in the goal")]
    InvalidPrerequisite {
        /// The prerequisite
        before: PieceSet<P>,
        /// The goal
        after: PieceSet<P>,
    },
}

impl ParseSequenceError {
    /// The 1-based line of the invalid move.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }
    /// The 1-based position of the invalid move within its line.
    #[must_use]
    pub const fn position(&self) -> usize {
        self.position
    }
    /// The same error on `line`, for text parsed one line at a time.
    pub(crate) fn on_line(self, line: usize) -> Self {
        Self { line, ..self }
    }
}

impl SolveError {
    /// The name of the step the solve stopped at.
    #[must_use]
    pub const fn step(&self) -> &str {
        match self {
            Self::Step { step, .. } | Self::NotDone { step } | Self::Requirements { step } => {
                step.as_str()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn solve_error_source_works() {
        let solve_error = SolveError::Step {
            step: "name".into(),
            source: StepError::MemoPoisoned,
        };

        let source = solve_error.source().expect("a step error has a source");

        assert!(matches!(
            source.downcast_ref::<StepError>(),
            Some(StepError::MemoPoisoned)
        ));
    }
}
