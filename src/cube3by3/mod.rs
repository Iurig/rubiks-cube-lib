pub mod moves;
pub mod pieces;

// `allow` instead of `expect` because the lint is skipped once the
// library is compiled with `cfg(test)`
#[allow(clippy::wildcard_imports)]
use self::{moves::*, pieces::*};
use crate::{
    ops::{Inv, Pow},
    zn::ZnRing,
};

#[derive(Debug, Clone, PartialEq, Eq, Default, Copy)]
#[expect(
    clippy::struct_field_names,
    reason = "_configuration makes clear what all variables are"
)]
pub struct Cube3By3 {
    /// `CENTER_ORIENTATION_COUNT` is 1, centers are considered without orientation
    center_configuration: CenterConfiguration,
    /// Corner orientation is done with the convention of clockwise rotations from white/yellow sticker being in the faces U and D
    corner_configuration: CornerConfiguration,
    /// 0 is oriented, 1 is misoriented
    edge_configuration: EdgeConfiguration,
}

impl std::ops::Mul for Cube3By3 {
    type Output = Self;
    /// Applies the state (permutations and orientations) that is the second argument to the first argument, which is a cube
    /// IMPORTANT: associative, but non-commutative
    fn mul(self, rhs: Self) -> Self::Output {
        self.const_mul(rhs)
    }
}

impl std::ops::Mul<Move> for Cube3By3 {
    type Output = Self;
    fn mul(self, m: Move) -> Self::Output {
        self.const_mul(Self::from(m))
    }
}

impl Cube3By3 {
    const fn const_mul(self, to_be_aplied: Self) -> Self {
        Self {
            center_configuration: self
                .center_configuration
                .then(&to_be_aplied.center_configuration),
            corner_configuration: self
                .corner_configuration
                .then(&to_be_aplied.corner_configuration),
            edge_configuration: self
                .edge_configuration
                .then(&to_be_aplied.edge_configuration),
        }
    }
}

impl Pow for Cube3By3 {
    type Output = Self;
    fn pow(&self, exponent: u64) -> Self::Output {
        if exponent == 0 {
            Self::IDENTITY
        } else {
            *self * self.pow(exponent - 1)
        }
    }
}

impl Inv for Cube3By3 {
    fn inverse(&self) -> Self {
        self.const_inverse()
    }
}

impl Cube3By3 {
    /// The multiplicative identity of the cube group: the solved cube
    pub const IDENTITY: Self = Self {
        center_configuration: CenterConfiguration::IDENTITY,
        corner_configuration: CornerConfiguration::IDENTITY,
        edge_configuration: EdgeConfiguration::IDENTITY,
    };

    const fn const_inverse(&self) -> Self {
        Self {
            center_configuration: self.center_configuration.const_inverse(),
            corner_configuration: self.corner_configuration.const_inverse(),
            edge_configuration: self.edge_configuration.const_inverse(),
        }
    }

    // Getters for the private fields of `Cube3By3`
    #[must_use]
    pub const fn corners(&self) -> &CornerConfiguration {
        &self.corner_configuration
    }
    #[must_use]
    pub const fn edges(&self) -> &EdgeConfiguration {
        &self.edge_configuration
    }
    #[must_use]
    pub const fn centers(&self) -> &CenterConfiguration {
        &self.center_configuration
    }

    /// Applies a move sequence to this cube state, in order.
    ///
    /// # Errors
    ///
    /// Errors when a token outside of comments is not a move; the error names
    /// that token. Moves before it are not applied.
    pub fn move_sequence(&self, moves: &str) -> Result<Self, String> {
        Move::sequence(moves).try_fold(*self, |cube, m| Ok(cube * Self::from(m?)))
    }

    /// Applies a move sequence to the solved cube.
    ///
    /// # Errors
    ///
    /// Same as [`Self::move_sequence`].
    pub fn from_solved(m: &str) -> Result<Self, String> {
        Self::default().move_sequence(m)
    }

    #[must_use]
    pub fn is_solved(&self) -> bool {
        self.rotated_until_solved_centers() == Some(Self::default())
    }

    fn rotated_until_solved_centers(&self) -> Option<Self> {
        let mut rotated_self = *self;
        if ![
            rotated_self.centers().piece_at(Center::F),
            rotated_self.centers().piece_at(Center::U),
            rotated_self.centers().piece_at(Center::B),
            rotated_self.centers().piece_at(Center::D),
        ]
        .contains(&Faces::F)
        {
            rotated_self = rotated_self
                * Move::new(MovablePart::Rotation(Rotations::y), MoveModifier::Clockwise);
        }
        for _ in 0..4 {
            if rotated_self.centers().piece_at(Faces::F) != Faces::F {
                rotated_self = rotated_self
                    * Move::new(MovablePart::Rotation(Rotations::x), MoveModifier::Clockwise);
            }
        }
        for _ in 0..4 {
            if rotated_self.centers().piece_at(Faces::U) != Faces::U {
                rotated_self = rotated_self
                    * Move::new(MovablePart::Rotation(Rotations::z), MoveModifier::Clockwise);
            }
        }

        (rotated_self.center_configuration == CenterConfiguration::default())
            .then_some(rotated_self)
    }

    /// Whether some move sequence produces this state from the solved cube.
    ///
    /// Four invariants, one per helper below. Every move preserves each of
    /// them, so a state that breaks one cannot be reached.
    #[must_use]
    pub fn is_reachable(&self) -> bool {
        self.twists_cancel()
            && self.flips_cancel()
            && self.parities_cancel()
            && self.centers_form_a_rotation()
    }

    /// Corner twists sum to zero mod 3: a face turn twists corners by amounts
    /// that cancel, so a lone twisted corner is unreachable.
    fn twists_cancel(&self) -> bool {
        self.corners().orientation_sum() == ZnRing::ZERO
    }

    /// Edge flips sum to zero mod 2: a face turn flips an even number of
    /// edges, so a lone flipped edge is unreachable.
    fn flips_cancel(&self) -> bool {
        self.edges().orientation_sum() == ZnRing::ZERO
    }

    /// The permutation parities of corners, edges, and centers sum to zero
    /// mod 2. A face turn is odd on corners and edges; a slice turn is odd on
    /// edges and centers. Every move flips exactly two of the three, so the
    /// sum stays zero. A two-way corner/edge check would reject a lone `M`.
    fn parities_cancel(&self) -> bool {
        self.corners().parity() + self.edges().parity() + self.centers().parity() == ZnRing::ZERO
    }

    /// The centers sit as one of the 24 whole-cube rotations. A 3-cycle of
    /// centers is an even permutation, so the parity sum accepts it, yet no
    /// move sequence produces it.
    fn centers_form_a_rotation(&self) -> bool {
        self.rotated_until_solved_centers().is_some()
    }
}
#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "tests should panic if failed, and return result for `?` convenience"
)]
mod tests {

    use crate::{Piece, piece::index};

    use super::*;
    #[test]
    fn default_is_solved() {
        assert!(Cube3By3::default().is_solved());
    }

    #[test]
    fn y_rotated_solved_is_solved() -> Result<(), String> {
        let rotated_def = Cube3By3::from_solved("y")?;
        assert!(rotated_def.is_solved());
        Ok(())
    }

    #[test]
    fn rotated_solved_is_solved() -> Result<(), String> {
        let rotated_def = Cube3By3::from_solved("y z y z x2 z2")?;
        assert!(rotated_def.is_solved());
        Ok(())
    }

    #[test]
    fn center_3_cyle_isnt_reachable_even_if_respects_parity() {
        assert!(
            !Cube3By3 {
                center_configuration: CenterConfiguration::cycle([[
                    Center::F,
                    Center::R,
                    Center::U
                ]]),
                ..Default::default()
            }
            .is_reachable()
        );
    }

    #[test]
    fn corner_twist_isnt_reachable() {
        assert!(
            !Cube3By3 {
                corner_configuration: CornerConfiguration {
                    orientation: ZnRing::array([1, 0, 0, 0, 0, 0, 0, 0,]),
                    ..Default::default()
                },
                ..Default::default()
            }
            .is_reachable()
        );
    }

    #[test]
    fn edge_flip_isnt_reachable() {
        assert!(
            !Cube3By3 {
                edge_configuration: EdgeConfiguration {
                    orientation: ZnRing::array([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,]),
                    ..Default::default()
                },
                ..Default::default()
            }
            .is_reachable()
        );
    }

    #[test]
    fn single_swap_isnt_reachable() {
        assert!(
            !Cube3By3 {
                corner_configuration: CornerConfiguration {
                    permutation: {
                        let mut p = Corner::ALL;
                        p.swap(index(Corner::Ufr), index(Corner::Ubr));
                        p
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
            .is_reachable()
        );
    }

    #[test]
    fn incorrect_strings_return_error() {
        assert!(Cube3By3::from_solved("Q").is_err());
        assert!(Cube3By3::from_solved("R Q U").is_err());
        assert!(Cube3By3::from_solved("R3").is_err());
    }

    #[test]
    fn rotations_match_moves() -> Result<(), String> {
        assert_eq!(
            Cube3By3::from_solved("y")?,
            Cube3By3::from_solved("U E' D'")?
        );
        assert_eq!(
            Cube3By3::from_solved("z")?,
            Cube3By3::from_solved("F S B'")?
        );
        assert_eq!(
            Cube3By3::from_solved("x")?,
            Cube3By3::from_solved("R M' L'")?
        );
        Ok(())
    }

    #[test]
    fn all_axes_rotated_solved_is_solved_but_not_after_a_face_turn() -> Result<(), String> {
        assert!(Cube3By3::from_solved("x y2 z'")?.is_solved());
        assert!(!Cube3By3::from_solved("x y2 z' R")?.is_solved());
        Ok(())
    }

    #[test]
    fn r_4_times_is_solved() -> Result<(), String> {
        let cube = Cube3By3::from_solved("R R R R")?;
        assert!(cube.is_solved());
        Ok(())
    }

    #[test]
    fn r_2_is_equal_to_r_prime_2() -> Result<(), String> {
        let r2 = Cube3By3::from_solved("R2")?;
        let r_prime_2 = Cube3By3::from_solved("R' R'")?;
        assert_eq!(r2, r_prime_2);
        Ok(())
    }

    #[test]
    fn r_r_prime_is_solved() -> Result<(), String> {
        let mut cube = Cube3By3::from_solved("R")?;
        assert!(!cube.is_solved());
        cube = cube.move_sequence("R'")?;
        assert!(cube.is_solved());
        Ok(())
    }

    #[test]
    fn r_prime_is_inverse_of_r() -> Result<(), String> {
        let r = Cube3By3::from_solved("R")?;
        let r_prime = Cube3By3::from_solved("R'")?;
        assert_eq!(r.const_inverse(), r_prime);
        Ok(())
    }

    #[test]
    fn composing_u_and_r_works() -> Result<(), String> {
        let u = Cube3By3::from_solved("U")?;
        let ur = Cube3By3::from_solved("U R")?;
        assert_eq!(u.move_sequence("R")?, ur);
        Ok(())
    }

    #[test]
    fn composition_works() -> Result<(), String> {
        let scramble_string = "R' ";
        let solution_string = "D2";
        let scramble = Cube3By3::from_solved(scramble_string)?;
        assert_eq!(
            Cube3By3::from_solved(&format!("{scramble_string} {solution_string}"))?,
            scramble.move_sequence(solution_string)?,
            "composing {scramble_string} and {solution_string} doesn't result in applying {scramble_string} {solution_string}"
        );
        Ok(())
    }

    #[test]
    fn composition_works_on_fmc_wr() -> Result<(), String> {
        let scramble_string =
            "R' U' F D2 L2 F R2 U2 R2 B D2 L B2 D' B2 L' R' B D2 B U2 L U2 R' U' F";
        let solution_string = "D2 F' D2 U2 F' L2 D R2 D B2 F L2 R' F' D U'";
        let scramble = Cube3By3::from_solved(scramble_string)?;
        assert_eq!(
            Cube3By3::from_solved(&format!("{scramble_string} {solution_string}"))?,
            scramble.move_sequence(solution_string)?,
            "composing {scramble_string} and {solution_string} doesn't result in applying {scramble_string} {solution_string}"
        );
        Ok(())
    }

    #[test]
    fn mul_carries_orientation_along_with_the_piece() -> Result<(), String> {
        // Pre-twist the piece at UFR, then apply R: that piece lands at UBR and
        // its twist is added to the twist R gives the UBR slot.
        let r = Cube3By3::from_solved("R")?;
        let mut twisted = Cube3By3::default();
        twisted.corner_configuration.orientation[Corner::Ufr as usize] = ZnRing::new(1);
        let after = twisted * r;
        let mut expected = r;
        expected.corner_configuration.orientation[Corner::Ubr as usize] =
            expected.corner_configuration.orientation[Corner::Ubr as usize] + ZnRing::new(1);
        assert_eq!(after, expected);
        Ok(())
    }
    #[test]
    fn u_perm_repeats_after_3_applications() {
        use Edge::{Uf, Ul, Ur};
        let u_perm = Cube3By3 {
            edge_configuration: EdgeConfiguration::cycle([[Ur, Uf, Ul]]),
            ..Default::default()
        };
        assert_eq!(u_perm.pow(3), Cube3By3::default());
    }

    #[test]
    fn sequences_without_moves_leave_the_cube_unchanged() -> Result<(), String> {
        let cube = Cube3By3::from_solved("R U")?;
        assert_eq!(cube.move_sequence("")?, cube);
        assert_eq!(cube.move_sequence("// nothing here")?, cube);
        assert_eq!(
            Cube3By3::from_solved("\n  // only comments\n")?,
            Cube3By3::IDENTITY
        );
        Ok(())
    }
}
