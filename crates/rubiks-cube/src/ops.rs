/// A value with a multiplicative inverse.
pub trait Inv: Sized {
    /// Inverts a state multiplicatively, possibly fallibly
    ///
    /// # Examples
    ///
    /// ```
    /// use rubiks_cube::Inv;
    /// #[derive(Clone, Debug, PartialEq)]
    /// enum FieldZ2 {
    ///     Zero,
    ///     One,
    /// }
    /// impl std::ops::Mul for FieldZ2 {
    ///     type Output = Self;
    ///     fn mul(self, rhs: Self) -> Self {
    ///         let product_table = [[FieldZ2::Zero, FieldZ2::One], [FieldZ2::One, FieldZ2::Zero]];
    ///         product_table[self as usize][rhs as usize].clone()
    ///     }
    /// }
    /// impl Inv for FieldZ2 {
    ///     fn inverse(&self) -> Self {
    ///         self.clone()
    ///     }
    /// }
    ///
    /// assert_eq!(FieldZ2::Zero, FieldZ2::Zero * FieldZ2::Zero.inverse());
    /// assert_eq!(FieldZ2::Zero, FieldZ2::One * FieldZ2::One.inverse());
    /// ```
    #[must_use = "this returns the inverse of a state, without modifying the original state"]
    fn inverse(&self) -> Self;
}

/// The inverse of a move sequence, such as an [`Algorithm`](crate::Algorithm): the moves in
/// reverse order, each one inverted. `R U` gives `U' R'`.
impl<M: Inv> Inv for Vec<M> {
    fn inverse(&self) -> Self {
        self.iter().rev().map(Inv::inverse).collect()
    }
}

/// Repeated multiplication by fast exponentiation.
pub trait Pow: std::ops::Mul<Self, Output = Self> + Clone {
    /// The result of an empty product.
    fn identity() -> Self;

    /// Performs the power operation based on `std::ops::Mul` assuming an empty multiplication
    /// returns [`Self::identity`].
    ///
    /// # Examples
    ///
    /// ```
    /// use rubiks_cube::Pow;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct TurnCount(u32);
    ///
    /// impl std::ops::Mul for TurnCount {
    ///     type Output = Self;
    ///     fn mul(self, rhs: Self) -> Self::Output {
    ///         TurnCount((self.0 + rhs.0) % 4)
    ///     }
    /// }
    ///
    /// impl Pow for TurnCount {
    ///     fn identity() -> Self {
    ///         TurnCount(0)
    ///     }
    /// }
    ///
    /// assert_eq!(TurnCount(2).pow(3), TurnCount(2));
    /// ```
    #[must_use = "returns the power instead of applying it in place"]
    fn pow(&self, exponent: u32) -> Self {
        // values 0, 1 and 2 are needed for recursion on `_` branch
        match exponent {
            0 => Self::identity(),
            1 => self.clone(),
            2 => self.clone() * self.clone(),
            _ => self.pow(exponent % 2) * self.pow(exponent / 2).pow(2),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "tests should panic if failed, and return result for `?` convenience"
)]
mod tests {
    use super::*;
    use crate::{Algorithm, Cube3x3, ParseSequenceError, Puzzle, puzzles::cube3x3::moves::Move3x3};
    use std::error::Error;

    fn algorithm(text: &str) -> Result<Algorithm<Cube3x3>, ParseSequenceError> {
        Move3x3::sequence(text).collect()
    }

    #[test]
    fn algorithm_inverse_reverses_and_inverts_each_move() -> Result<(), Box<dyn Error>> {
        assert_eq!(algorithm("R U R' F2")?.inverse(), algorithm("F2' R U' R'")?);
        Ok(())
    }

    #[test]
    fn empty_algorithm_is_its_own_inverse() {
        assert_eq!(Algorithm::<Cube3x3>::new().inverse(), Algorithm::new());
    }

    #[test]
    fn inverting_an_algorithm_twice_gives_it_back() -> Result<(), Box<dyn Error>> {
        let alg = algorithm("r U' M2 x E' Fw2 D")?;
        assert_eq!(alg.inverse().inverse(), alg);
        Ok(())
    }

    #[test]
    fn algorithm_then_its_inverse_returns_to_the_start() -> Result<(), Box<dyn Error>> {
        let alg = algorithm("R U R' U' R' F R2 U' R' U' R U R' F'")?;
        for seed in 0..20 {
            let start = Cube3x3::apply_scramble_with_seed(seed);
            let end = alg
                .iter()
                .chain(&alg.inverse())
                .fold(start, |cube, &m| cube * m);
            assert_eq!(end, start, "seed {seed}");
        }
        Ok(())
    }

    #[test]
    fn clockwise_moves_have_order_exactly_4() -> Result<(), Box<dyn Error>> {
        for m in &["R", "U", "D", "L", "F", "B", "E", "S", "M"] {
            let cube = Cube3x3::from_solved(m)?;
            for k in 1..4 {
                assert!(!cube.pow(k).is_solved(), "{m}^{k} should not be solved");
            }
            assert!(cube.pow(4).is_solved(), "{m}^4 should be solved");
        }
        Ok(())
    }
}
