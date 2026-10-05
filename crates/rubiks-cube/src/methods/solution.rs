use std::fmt::Display;

use crate::{Algorithm, puzzles::Puzzle};

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
#[derive(Debug, Eq, PartialEq)]
pub struct Solution<P: Puzzle> {
    step_solutions: Vec<Segment<P>>,
}

/// Segment of a solution: includes a name and the moves that make the solution up.
#[derive(Debug, Eq, PartialEq)]
pub struct Segment<P: Puzzle> {
    pub(super) moves: Algorithm<P>,
    pub(super) name: String,
}

impl<P: Puzzle> Segment<P> {
    /// Getter for the name of the step that generated the segment.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Getter for the moves that make the segment up.
    #[must_use]
    pub const fn moves(&self) -> &Algorithm<P> {
        &self.moves
    }
}

impl<P: Puzzle> Solution<P> {
    /// A solution with no segments.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Each segment's moves and step name, in solving order.
    pub fn iter(&self) -> std::slice::Iter<'_, Segment<P>> {
        self.step_solutions.iter()
    }

    /// The number of moves in every segment, each move counting one whatever its part or
    /// modifier (slice turn metric). `M`, `Rw`, `y`, and `U2` each count one.
    #[must_use]
    pub fn move_count(&self) -> usize {
        self.step_solutions
            .iter()
            .map(|segment| segment.moves.len())
            .sum()
    }
}

/// Builds a solution from `(moves, step name)` segments, in order.
impl<P: Puzzle> FromIterator<Segment<P>> for Solution<P> {
    fn from_iter<I: IntoIterator<Item = Segment<P>>>(iter: I) -> Self {
        Self {
            step_solutions: iter.into_iter().collect(),
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
            step_solutions: Vec::<Segment<P>>::new(),
        }
    }
}
impl<'a, P: Puzzle> IntoIterator for &'a Solution<P> {
    type Item = &'a Segment<P>;
    type IntoIter = std::slice::Iter<'a, Segment<P>>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<P: Puzzle> Extend<Segment<P>> for Solution<P> {
    fn extend<T: IntoIterator<Item = Segment<P>>>(&mut self, iter: T) {
        self.step_solutions.extend(iter);
    }
}
impl<P: Puzzle> IntoIterator for Solution<P> {
    type Item = Segment<P>;
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
                .map(|segment| {
                    segment
                        .moves
                        .iter()
                        .map(P::Moves::to_string)
                        .collect::<Vec<String>>()
                        .join(" ")
                        + "\t//"
                        + &segment.name
                        + "\n"
                })
                .collect::<String>()
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

    use crate::{Cube3x3, ParseSequenceError, puzzles::cube3x3::moves::Move3x3};

    use super::*;

    #[test]
    fn move_count_counts_correctly() -> Result<(), Box<dyn Error>> {
        let s: Solution<Cube3x3> = Solution::from_iter([Segment {
            moves: Move3x3::sequence("y U2 r M'")
                .collect::<Result<Algorithm<Cube3x3>, ParseSequenceError>>()?,
            name: "Step 1".to_string(),
        }]);
        assert_eq!(s.move_count(), 4);
        Ok(())
    }

    fn segment(moves: &str, name: &str) -> Result<Segment<Cube3x3>, ParseSequenceError> {
        Ok(Segment {
            moves: Move3x3::sequence(moves).collect::<Result<_, _>>()?,
            name: name.to_string(),
        })
    }

    #[test]
    fn borrowed_and_owned_iteration_yield_the_same_segments_in_order() -> Result<(), Box<dyn Error>>
    {
        let solution: Solution<Cube3x3> = [segment("R U", "First")?, segment("M'", "Second")?]
            .into_iter()
            .collect();

        let mut borrowed = Vec::new();
        for segment in &solution {
            borrowed.push((segment.name.clone(), segment.moves.clone()));
        }
        let owned: Vec<(String, Algorithm<Cube3x3>)> = solution
            .into_iter()
            .map(|segment| (segment.name, segment.moves))
            .collect();

        let names: Vec<&str> = borrowed.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(names, ["First", "Second"]);
        assert_eq!(borrowed, owned);
        Ok(())
    }

    #[test]
    fn extend_appends_segments_after_the_existing_ones() -> Result<(), Box<dyn Error>> {
        let mut solution: Solution<Cube3x3> = Solution::from_iter([segment("R", "First")?]);

        solution.extend([segment("U2", "Second")?, segment("F' L", "Third")?]);

        let names: Vec<&str> = solution.iter().map(Segment::name).collect();
        assert_eq!(names, ["First", "Second", "Third"]);
        assert_eq!(solution.move_count(), 4);
        Ok(())
    }
}
