use crate::{
    Cube3x3, ParseSequenceError, Puzzle,
    puzzles::cube3by3::moves::{MovablePart, Move3x3, MoveModifier},
};

/// The move sequences a [`SearchStep`](crate::SearchStep) may apply, each as one unit.
///
/// A sequence is costly, so it counts toward the step's cost when searching, or free,
/// such as a `U` turn between algorithms, which is only minimized if the number of algorithms is
/// already minimal. For the cube, build one from notation text with
/// [`from_parts`](Self::from_parts), [`from_algorithms`](Self::from_algorithms), or
/// [`from_moves`](Self::from_moves), and join several with [`combined_with`](Self::combined_with).
/// Parts and moves take `costly`: `true` for cost one, `false` for free. Algorithms always cost one
/// per sequence.
///
/// ```
/// use rubiks_cube_lib::{Cube3x3, Moveset};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // An algorithm set plus a free AUF.
/// let moveset = Moveset::<Cube3x3>::from_algorithms("R U R' U R U2 R'\nR U2 R' U' R U' R'")?
///     .combined_with(Moveset::from_moves("U U2 U'", false)?);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Moveset<P: Puzzle>(Vec<(Vec<P::Moves>, bool)>);

impl<P: Puzzle> Moveset<P> {
    /// Combines both movesets, with this moveset's sequences first.
    ///
    /// Preserves each sequence's free or costly status and retains duplicates.
    #[must_use]
    pub fn combined_with(mut self, other: Self) -> Self {
        self.0.extend(other.0);
        self
    }

    /// Each sequence with whether it is costly (`true`) or free (`false`).
    pub(crate) fn sequences(&self) -> &[(Vec<P::Moves>, bool)] {
        &self.0
    }
}

impl Moveset<Cube3x3> {
    /// Every modifier of each part named in `text`, one move per sequence.
    ///
    /// The modifier in the text does not matter: `"U R M r"` and `"U' R2 M r'"` both give the
    /// clockwise, counterclockwise, and double turn of `U`, `R`, `M`, and `r`, twelve moves.
    /// A part named twice is listed once.
    /// Each sequence costs one if `costly` is `true`, or is free if `false`.
    ///
    /// # Errors
    /// If `text` contains an invalid move.
    pub fn from_parts(text: &str, costly: bool) -> Result<Self, ParseSequenceError> {
        Ok(Self(
            Move3x3::sequence(text)
                .try_fold(Vec::<MovablePart>::new(), |mut parts, m| {
                    let part = m?.part;
                    if !parts.contains(&part) {
                        parts.push(part);
                    }
                    Ok::<_, ParseSequenceError>(parts)
                })?
                .into_iter()
                .flat_map(|part| {
                    [
                        MoveModifier::Clockwise,
                        MoveModifier::CounterClockwise,
                        MoveModifier::Double,
                    ]
                    .map(|modifier| (vec![Move3x3::new(part, modifier)], costly))
                })
                .collect(),
        ))
    }

    /// One sequence per line of `text`, each costing one, such as an algorithm set. Lines with no
    /// moves, such as blank or comment-only lines, are skipped.
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
                        .map(|moves| (!moves.is_empty()).then_some((moves, true)))
                        .map_err(|e| e.on_line(line_number + 1))
                        .transpose()
                })
                .collect::<Result<_, _>>()?,
        ))
    }

    /// Each move of `text` as its own sequence, preserving its modifier and duplicates.
    ///
    /// Each sequence costs one if `costly` is `true`, or is free if `false`.
    ///
    /// # Errors
    /// If `text` contains an invalid move.
    pub fn from_moves(text: &str, costly: bool) -> Result<Self, ParseSequenceError> {
        Ok(Self(
            Move3x3::sequence(text)
                .map(|m| m.map(|m| (vec![m], costly)))
                .collect::<Result<_, _>>()?,
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
    fn from_parts_gives_every_modifier_with_the_requested_cost() -> Result<(), Box<dyn Error>> {
        for costly in [false, true] {
            let moveset = Moveset::<Cube3x3>::from_parts("U R M r", costly)?;

            assert_eq!(moveset.sequences().len(), 12);
            assert!(
                moveset
                    .sequences()
                    .iter()
                    .all(|(sequence, cost)| sequence.len() == 1 && *cost == costly)
            );
        }
        Ok(())
    }

    #[test]
    fn from_parts_ignores_the_modifier_and_repeated_parts() -> Result<(), Box<dyn Error>> {
        let plain = Moveset::<Cube3x3>::from_parts("U R", true)?;
        let modified = Moveset::<Cube3x3>::from_parts("U' R2 U2", true)?;

        assert_eq!(plain.sequences(), modified.sequences());
        Ok(())
    }

    #[test]
    fn from_algorithms_gives_one_costly_sequence_per_line() -> Result<(), Box<dyn Error>> {
        let moveset = Moveset::<Cube3x3>::from_algorithms("R U R' U'\n\nF R U R' U' F'\nM2 U2")?;

        let lengths: Vec<(usize, bool)> = moveset
            .sequences()
            .iter()
            .map(|(sequence, cost)| (sequence.len(), *cost))
            .collect();
        assert_eq!(lengths, [(4, true), (6, true), (2, true)]);
        Ok(())
    }

    #[test]
    fn from_moves_preserves_exact_moves_with_the_requested_cost() -> Result<(), Box<dyn Error>> {
        let text = "U U2 U' U";
        let expected = Move3x3::sequence(text).collect::<Result<Vec<_>, _>>()?;
        for costly in [false, true] {
            let moveset = Moveset::<Cube3x3>::from_moves(text, costly)?;

            assert_eq!(moveset.sequences().len(), expected.len());
            assert!(
                moveset
                    .sequences()
                    .iter()
                    .zip(&expected)
                    .all(|((sequence, cost), m)| sequence == &[*m] && *cost == costly)
            );
        }
        Ok(())
    }

    #[test]
    fn combined_with_keeps_both_movesets_in_order() -> Result<(), Box<dyn Error>> {
        let moveset = Moveset::<Cube3x3>::from_algorithms("R U R'")?
            .combined_with(Moveset::from_moves("U", false)?);

        let costs: Vec<bool> = moveset.sequences().iter().map(|&(_, c)| c).collect();
        assert_eq!(costs, [true, false]);
        Ok(())
    }

    #[test]
    fn an_invalid_move_names_its_line_and_position() {
        for result in [
            Moveset::<Cube3x3>::from_parts("U R\nM Q2", true),
            Moveset::<Cube3x3>::from_algorithms("R U R'\nM Q2"),
            Moveset::<Cube3x3>::from_moves("U\nU2 Q2", false),
        ] {
            let Err(error) = result else {
                panic!("`Q2` should not parse");
            };
            assert_eq!((error.line(), error.position()), (2, 2), "{error:?}");
        }
    }
}
