use std::{error::Error, fmt::Display};

/// Why a [`Step`](crate::Step) could not solve.
#[derive(Debug)]
#[non_exhaustive]
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
#[non_exhaustive]
pub enum SolveError {
    /// A step's requirement isn't met when it should be solved.
    Requirements {
        /// The step's name.
        step: String,
    },
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
    pub const fn step(&self) -> &str {
        match self {
            Self::Step { step, .. } | Self::NotDone { step } | Self::Requirements { step } => {
                step.as_str()
            }
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
                Self::UnreachableGoal => "goal could not be reached with the given algset",
                Self::InvalidStartingState => "starting state doesn't fit expected properties",
                Self::MemoPoisoned =>
                    "the step's memo is unusable: another solve panicked while deepening it",
                Self::Custom(e) => return e.fmt(f),
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
                Self::Requirements { step } => format!(
                    "step {step} couldn't start solving because its requirements was not met"
                ),
                Self::NotDone { step } =>
                    format!("step {step} returned a solution, but its goal is not met"),
                Self::Step { step, error } => format!("step {step} could not finish: {error}"),
            }
        )
    }
}
