use std::sync::Arc;

use crate::{
    Algorithm, Cube3x3, ParseSequenceError, Puzzle,
    fast_hash::FxSet,
    puzzles::cube3x3::moves::{MovablePart, Move3x3, MoveModifier},
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
/// [`from_algs_in_str`](Self::from_algs_in_str), or [`from_moves`](Self::from_moves), and join
/// several with [`combined_with`](Self::combined_with).
///
/// ```
/// use rubiks_cube::{Cube3x3, AlgSet};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Two algorithms to search with, and the AUF to pass as the free sequences.
/// let algorithms = AlgSet::<Cube3x3>::from_algs_in_str("R U R' U R U2 R'\nR U2 R' U' R U' R'")?;
/// let auf = AlgSet::<Cube3x3>::from_moves("U U2 U'")?;
/// # Ok(())
/// # }
/// ```
///
/// Two collections with the same sequences are equal and hash alike, whatever order the
/// sequences were given in and however often each was repeated.
// Every constructor goes through `FromIterator`, which sorts and dedups, so the derived
// `PartialEq` and `Hash` compare the sequences as a set.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct AlgSet<P: Puzzle>(Arc<[Algorithm<P>]>);

impl<P: Puzzle> AlgSet<P> {
    /// Every sequence of either collection, each listed once.
    ///
    /// The order of the sequences is not kept, and a sequence in both collections, or twice in
    /// one, appears once.
    #[must_use]
    pub fn combined_with(&self, other: &Self) -> Self {
        self.0.iter().chain(other.0.iter()).cloned().collect()
    }

    /// The sequences.
    pub(crate) fn algs(&self) -> &[Algorithm<P>] {
        &self.0
    }
}

impl<P: Puzzle> FromIterator<Algorithm<P>> for AlgSet<P> {
    fn from_iter<T: IntoIterator<Item = Algorithm<P>>>(iter: T) -> Self {
        let mut algs: Vec<Algorithm<P>> = iter.into_iter().collect();
        algs.sort_unstable_by(|a, b| a.iter().cmp(b.iter()));
        algs.dedup();
        Self(algs.into())
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
        Ok(Move3x3::sequence(text)
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
                .map(|modifier| Algorithm::from_iter([Move3x3::new(part, modifier)]))
            })
            .collect())
    }

    /// One sequence per line of `text`, such as an algorithm set. Lines with no moves, such as
    /// blank or comment-only lines, are skipped. A line written twice is listed once, and the
    /// order of the lines is not kept.
    ///
    /// # Errors
    /// If `text` contains an invalid move. The error's line counts lines of the whole `text`.
    pub fn from_algs_in_str(text: &str) -> Result<Self, ParseSequenceError> {
        text.lines()
            .enumerate()
            .filter_map(|(line_number, line)| {
                Move3x3::sequence(line)
                    .collect::<Result<Algorithm<Cube3x3>, _>>()
                    .map(|moves| (!moves.is_empty()).then_some(moves))
                    .map_err(|e| e.on_line(line_number + 1))
                    .transpose()
            })
            .collect()
    }

    /// Each move of `text` as its own sequence, keeping its modifier.
    ///
    /// A move written twice is listed once, and the order of the moves is not kept.
    ///
    /// # Errors
    /// If `text` contains an invalid move.
    pub fn from_moves(text: &str) -> Result<Self, ParseSequenceError> {
        Move3x3::sequence(text)
            .map(|m| m.map(|m| Algorithm::from_iter([m])))
            .collect()
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
    fn combined_with_reconstructs_an_algset() -> Result<(), Box<dyn Error>> {
        let algset = AlgSet::from_parts("R U L F B D")?;

        for (splitpoint, _) in algset.algs().iter().enumerate() {
            let first_section = algset.algs()[..splitpoint].to_vec();
            let second_section = algset.algs()[splitpoint..].to_vec();
            assert_eq!(
                algset,
                AlgSet::from_iter(first_section).combined_with(&AlgSet::from_iter(second_section))
            );
        }
        Ok(())
    }

    #[test]
    fn algsets_are_equal_when_they_hold_the_same_sequences() -> Result<(), Box<dyn Error>> {
        let algset = AlgSet::<Cube3x3>::from_algs_in_str("R U\nF")?;

        // Order and repeats don't matter.
        for same in ["R U\nF", "F\nR U", "R U\nF\nR U"] {
            let same = AlgSet::from_algs_in_str(same)?;
            assert_eq!(algset, same);
            assert_eq!(same, algset);
        }

        // A missing, extra, or different sequence does.
        for different in ["", "R U", "R U\nF\nF'", "U R\nF"] {
            let different = AlgSet::from_algs_in_str(different)?;
            assert_ne!(algset, different);
            assert_ne!(different, algset);
        }

        Ok(())
    }

    #[test]
    fn from_parts_gives_every_modifier() -> Result<(), Box<dyn Error>> {
        let algset = AlgSet::<Cube3x3>::from_parts("U R M r")?;

        assert_eq!(algset.algs().len(), 12);
        assert!(algset.algs().iter().all(|sequence| sequence.len() == 1));

        Ok(())
    }

    #[test]
    fn from_algs_in_str_gives_one_sequence_per_line() -> Result<(), Box<dyn Error>> {
        let algset = AlgSet::<Cube3x3>::from_algs_in_str("R U R' U'\n\nF R U R' U' F'\nM2 U2")?;

        let mut lengths: Vec<usize> = algset.algs().iter().map(Algorithm::len).collect();
        lengths.sort_unstable();
        assert_eq!(lengths, [2, 4, 6]);
        Ok(())
    }

    #[test]
    fn from_moves_removes_duplicates() -> Result<(), Box<dyn Error>> {
        let expected: FxSet<Algorithm<Cube3x3>> = Move3x3::sequence("U U2 U'")
            .map(|m| m.map(|m| Algorithm::from_iter([m])))
            .collect::<Result<_, _>>()?;

        let algset = AlgSet::<Cube3x3>::from_moves("U U2 U' U")?;

        let actual: FxSet<Algorithm<Cube3x3>> = algset.algs().iter().cloned().collect();
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn an_invalid_move_names_its_line_and_position() {
        for result in [
            AlgSet::<Cube3x3>::from_parts("U R\nM Q2"),
            AlgSet::<Cube3x3>::from_algs_in_str("R U R'\nM Q2"),
            AlgSet::<Cube3x3>::from_moves("U\nU2 Q2"),
        ] {
            let Err(error) = result else {
                panic!("`Q2` should not parse");
            };
            assert_eq!((error.line(), error.position()), (2, 2), "{error:?}");
        }
    }
}
