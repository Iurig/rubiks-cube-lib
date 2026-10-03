use std::fmt::{self, Display};

use crate::{Inv, Puzzle};

/// A sequence of moves applied in order, such as an algorithm or a step's solution.
///
/// It prints as its moves separated by spaces, in the crate's notation, so what it prints reads
/// back as the same moves.
///
/// ```no_run
/// use rubiks_cube::{Algorithm, Cube3x3, Inv, Solution};
/// use rubiks_cube::Method;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut cube = Cube3x3::from_solved("R U R' F2")?;
/// let solution: Solution<Cube3x3> = Method::kociemba().solve(&mut cube)?;
/// let moves: Algorithm<Cube3x3> = solution.iter().flat_map(|s| s.moves()).copied().collect();
/// // The inverse of a solution is a scramble that gives back the state it solved.
/// assert_eq!(Cube3x3::from_solved(&moves.inverse().to_string())?, Cube3x3::from_solved("R U R' F2")?);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Algorithm<P: Puzzle>(Vec<P::Moves>);

impl<P: Puzzle> Algorithm<P> {
    /// An algorithm with no moves.
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// The moves, in the order they are applied.
    pub fn iter(&self) -> std::slice::Iter<'_, P::Moves> {
        self.0.iter()
    }

    /// The number of moves.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the algorithm has no moves.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Adds the moves of `other` after the last move.
    pub fn extend_from(&mut self, other: &Self) {
        self.0.extend_from_slice(&other.0);
    }

    /// Keeps the first `len` moves and drops the rest.
    pub fn truncate(&mut self, len: usize) {
        self.0.truncate(len);
    }
}

/// The moves separated by spaces, such as `R U R' U'`. An empty algorithm prints nothing.
impl<P: Puzzle> Display for Algorithm<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut moves = self.0.iter();
        if let Some(first) = moves.next() {
            write!(f, "{first}")?;
            for m in moves {
                write!(f, " {m}")?;
            }
        }
        Ok(())
    }
}

/// The moves in reverse order, each inverted: applying an algorithm and then its inverse leaves
/// any state unchanged.
impl<P: Puzzle> Inv for Algorithm<P> {
    fn inverse(&self) -> Self {
        Self(self.0.inverse())
    }
}

impl<P: Puzzle> FromIterator<P::Moves> for Algorithm<P> {
    fn from_iter<I: IntoIterator<Item = P::Moves>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<P: Puzzle> Extend<P::Moves> for Algorithm<P> {
    fn extend<I: IntoIterator<Item = P::Moves>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<P: Puzzle> IntoIterator for Algorithm<P> {
    type Item = P::Moves;
    type IntoIter = std::vec::IntoIter<P::Moves>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, P: Puzzle> IntoIterator for &'a Algorithm<P> {
    type Item = &'a P::Moves;
    type IntoIter = std::slice::Iter<'a, P::Moves>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(test)]
#[expect(clippy::panic_in_result_fn, reason = "Standardized across tests")]
mod test {
    use std::error::Error;

    use crate::{Cube3x3, puzzles::cube3x3::moves::Move3x3};

    use super::*;

    #[test]
    fn algorithm_into_iter_and_iter_agree() -> Result<(), Box<dyn Error>> {
        let alg = Move3x3::sequence("R U R' U' R' F R2 U' R' U' R U R' F'")
            .collect::<Result<Algorithm<Cube3x3>, _>>()?;
        let alg_vec_2 = alg.iter().copied().collect::<Vec<_>>();
        let alg_vec_1 = alg.into_iter().collect::<Vec<_>>();
        assert_eq!(alg_vec_1, alg_vec_2);
        Ok(())
    }
}
