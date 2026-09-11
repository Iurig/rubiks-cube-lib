pub trait Inv: Sized {
    /// Inverts a state multiplicatively, possibly fallibly
    ///
    /// # Examples
    ///
    /// ```
    /// use rubiks_cube_lib::Inv;
    /// #[derive(Debug, Clone, PartialEq)]
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

pub trait Pow: std::ops::Mul<Self, Output = Self> + Clone {
    const IDENTITY: Self;

    /// Performs the power operation based on `std::ops::Mul` assuming an empty multiplication returns `IDENTITY`
    ///
    /// # Examples
    ///
    /// ```
    /// use rubiks_cube_lib::Pow;
    ///
    /// #[derive(Debug, Clone, PartialEq)]
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
    ///     const IDENTITY: Self = TurnCount(0);
    /// }
    ///
    /// assert_eq!(TurnCount(2).pow(3), TurnCount(2));
    /// ```
    #[must_use = "returns the power instead of applying it in place"]
    fn pow(&self, exponent: u32) -> Self {
        // values 0, 1 and 2 are needed for recursion on `_` branch
        match exponent {
            0 => Self::IDENTITY,
            1 => self.clone(),
            2 => self.clone() * self.clone(),
            _ => self.pow(exponent % 2) * self.pow(exponent / 2).pow(2),
        }
    }
}
