pub trait Inv
where
    Self: std::marker::Sized,
{
    /// Inverts a state multiplicatively, possibly fallibly
    ///
    /// # Examples
    ///
    /// ```
    /// use rubiks::Inv;
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

pub trait Pow {
    /// The resulting type after applying the `.pow()` operation.
    type Output;

    /// Performs the power operation.
    ///
    /// # Example
    ///
    /// ```
    /// assert_eq!(2_i32.pow(3), 8);
    /// ```
    fn pow(&self, exponent: u64) -> Self::Output;
}
