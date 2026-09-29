use std::sync::Arc;

use crate::{
    Algorithm, Cube3x3, ParseSequenceError, Puzzle,
    fast_hash::FxSet,
    puzzles::cube3by3::moves::{MovablePart, Move3x3, MoveModifier},
};

/// A collection of move sequences, each applied as one unit: a single move, or a whole algorithm.
///
/// The collection carries no cost. Cost is decided by the [`SearchStep`](crate::SearchStep) that
/// uses it: every sequence of the collection given to [`SearchStep::new`](crate::SearchStep::new)
/// costs one, and [`SearchStep::new_with_free_algs`](crate::SearchStep::new_with_free_algs) also
/// takes a second collection of free sequences, such as a `U` turn between algorithms, which the
/// search only minimizes once the number of costly sequences is already minimal.
///
/// For the cube, build one from notation text with [`from_parts`](Self::from_parts),
/// [`from_algorithms`](Self::from_algorithms), or [`from_moves`](Self::from_moves), and join
/// several with [`combined_with`](Self::combined_with).
///
/// ```
/// use rubiks_cube::{Cube3x3, AlgSet};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Two algorithms to search with, and the AUF to pass as the free sequences.
/// let algorithms = AlgSet::<Cube3x3>::from_algorithms("R U R' U R U2 R'\nR U2 R' U' R U' R'")?;
/// let auf = AlgSet::<Cube3x3>::from_moves("U U2 U'")?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Default)]
pub struct AlgSet<P: Puzzle>(Arc<[Algorithm<P>]>);

impl<P: Puzzle> AlgSet<P> {
    /// Every sequence of either collection, each listed once.
    ///
    /// The order of the sequences is not kept, and a sequence in both collections, or twice in
    /// one, appears once.
    #[must_use]
    pub fn combined_with(&self, other: &Self) -> Self {
        Self(
            self.0
                .iter()
                .chain(other.0.iter())
                .collect::<FxSet<&Algorithm<P>>>()
                .into_iter()
                .cloned()
                .collect::<Arc<[Algorithm<P>]>>(),
        )
    }

    /// The sequences.
    pub(crate) fn algs(&self) -> &[Algorithm<P>] {
        &self.0
    }
}

impl<P: Puzzle> FromIterator<Vec<P::Moves>> for AlgSet<P> {
    fn from_iter<T: IntoIterator<Item = Vec<P::Moves>>>(iter: T) -> Self {
        let inner = iter.into_iter().collect();
        Self(inner)
    }
}

impl AlgSet<Cube3x3> {
    /// Every modifier of each part named in `text`, one move per sequence.
    ///
    /// The modifier in the text does not matter: `"U R M r"` and `"U' R2 M r'"` both give the
    /// clockwise, counterclockwise, and double turn of `U`, `R`, `M`, and `r`, twelve moves.
    /// A part named twice is listed once, and the order of the parts is not kept.
    ///
    /// # Errors
    /// If `text` contains an invalid move.
    pub fn from_parts(text: &str) -> Result<Self, ParseSequenceError> {
        Ok(Self(
            Move3x3::sequence(text)
                .try_fold(FxSet::<MovablePart>::default(), |mut parts, m| {
                    let part = m?.part;
                    parts.insert(part);
                    Ok(parts)
                })?
                .into_iter()
                .flat_map(|part| {
                    [
                        MoveModifier::Clockwise,
                        MoveModifier::CounterClockwise,
                        MoveModifier::Double,
                    ]
                    .map(|modifier| vec![Move3x3::new(part, modifier)])
                })
                .collect(),
        ))
    }

    /// One sequence per line of `text`, such as an algorithm set. Lines with no moves, such as
    /// blank or comment-only lines, are skipped.
    ///
    /// # Errors
    /// If `text` contains an invalid move. The error's line counts lines of the whole `text`.
    pub fn from_algorithms(text: &str) -> Result<Self, ParseSequenceError> {
        Ok(Self(
            text.lines()
                .enumerate()
                .filter_map(|(line_number, line)| {
                    Move3x3::sequence(line)
                        .collect::<Result<Vec<_>, _>>()
                        .map(|moves| (!moves.is_empty()).then_some(moves))
                        .map_err(|e| e.on_line(line_number + 1))
                        .transpose()
                })
                .collect::<Result<_, _>>()?,
        ))
    }

    /// Each move of `text` as its own sequence, keeping its modifier.
    ///
    /// A move written twice is listed once, and the order of the moves is not kept.
    ///
    /// # Errors
    /// If `text` contains an invalid move.
    pub fn from_moves(text: &str) -> Result<Self, ParseSequenceError> {
        Ok(Self(
            Move3x3::sequence(text)
                .map(|m| m.map(|m| vec![m]))
                .try_fold(FxSet::default(), |mut s, m| {
                    s.insert(m?);
                    Ok(s)
                })?
                .into_iter()
                .collect(),
        ))
    }
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "tests should panic if failed, and return result for `?` convenience"
)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn from_parts_gives_every_modifier() -> Result<(), Box<dyn Error>> {
        let algset = AlgSet::<Cube3x3>::from_parts("U R M r")?;

        assert_eq!(algset.algs().len(), 12);
        assert!(algset.algs().iter().all(|sequence| sequence.len() == 1));

        Ok(())
    }

    #[test]
    fn from_algorithms_gives_one_sequence_per_line() -> Result<(), Box<dyn Error>> {
        let algset = AlgSet::<Cube3x3>::from_algorithms("R U R' U'\n\nF R U R' U' F'\nM2 U2")?;

        let lengths: Vec<usize> = algset.algs().iter().map(Vec::len).collect();
        assert_eq!(lengths, [4, 6, 2]);
        Ok(())
    }

    #[test]
    fn from_moves_removes_duplicates() -> Result<(), Box<dyn Error>> {
        let expected: FxSet<Vec<Move3x3>> = Move3x3::sequence("U U2 U'")
            .map(|m| m.map(|m| vec![m]))
            .collect::<Result<_, _>>()?;

        let algset = AlgSet::<Cube3x3>::from_moves("U U2 U' U")?;

        let actual: FxSet<Vec<Move3x3>> = algset.algs().iter().cloned().collect();
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn an_invalid_move_names_its_line_and_position() {
        for result in [
            AlgSet::<Cube3x3>::from_parts("U R\nM Q2"),
            AlgSet::<Cube3x3>::from_algorithms("R U R'\nM Q2"),
            AlgSet::<Cube3x3>::from_moves("U\nU2 Q2"),
        ] {
            let Err(error) = result else {
                panic!("`Q2` should not parse");
            };
            assert_eq!((error.line(), error.position()), (2, 2), "{error:?}");
        }
    }
}
