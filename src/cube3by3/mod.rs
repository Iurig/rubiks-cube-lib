pub mod moves;
pub mod pieces;

#[allow(clippy::wildcard_imports)]
use self::{moves::*, pieces::*};
use crate::{
    ops::{Inv, Pow},
    string_processing::RubiksCubeCleaning,
    zn::ZnRing,
};

#[derive(Debug, Clone, PartialEq, Eq, Default, Copy)]
#[expect(
    clippy::struct_field_names,
    reason = "_configuration makes clear what all variables are"
)]
pub struct Cube3By3 {
    /// `CENTER_ORIENTATION_COUNT` is 1, centers are considered without orientation
    center_configuration: Centers,
    /// Corner orientation is done with the convention of clockwise rotations from white/yellow sticker being in the U/B faces
    corner_configuration: Corners,
    /// 0 is oriented, 1 is misoriented
    edge_configuration: Edges,
}

impl std::ops::Mul for Cube3By3 {
    type Output = Self;
    /// Applies the permutation the second cube to the first cube
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
        center_configuration: Centers::IDENTITY,
        corner_configuration: Corners::IDENTITY,
        edge_configuration: Edges::IDENTITY,
    };

    const fn const_inverse(&self) -> Self {
        Self {
            center_configuration: self.center_configuration.const_inverse(),
            corner_configuration: self.corner_configuration.const_inverse(),
            edge_configuration: self.edge_configuration.const_inverse(),
        }
    }

    /// # Errors
    ///
    /// Errors when passed a string that contains whitespace separated sections that cannot be parsed as moves outside of comments
    pub fn move_sequence(&self, moves: &str) -> Result<Self, String> {
        if moves.contains(char::is_whitespace) {
            moves
                .process_movement_input()
                .try_fold(*self, |cube, single_move| cube.move_sequence(&single_move))
        } else {
            Ok(*self * Self::from(Move::try_from(moves)?))
        }
    }

    /// # Errors
    ///
    /// Errors when passed a string that contains whitespace separated sections that cannot be parsed as moves outside of comments
    pub fn from_solved(m: &str) -> Result<Self, String> {
        Self::default().move_sequence(m)
    }

    #[must_use]
    pub fn is_solved(&self) -> bool {
        let mut rotated_self = *self;
        while ![
            rotated_self.center_configuration.permutation[0],
            rotated_self.center_configuration.permutation[1],
            rotated_self.center_configuration.permutation[3],
            rotated_self.center_configuration.permutation[5],
        ]
        .contains(&Faces::F)
        {
            rotated_self = rotated_self
                * Move::new(MovablePart::Rotation(Rotations::y), MoveModifier::Clockwise);
        }
        while rotated_self.center_configuration.permutation[1] != Faces::F {
            rotated_self = rotated_self
                * Move::new(MovablePart::Rotation(Rotations::x), MoveModifier::Clockwise);
        }
        while rotated_self.center_configuration.permutation[0] != Faces::U {
            rotated_self = rotated_self
                * Move::new(MovablePart::Rotation(Rotations::z), MoveModifier::Clockwise);
        }
        rotated_self == Self::IDENTITY
    }
    #[must_use]
    pub fn respects_orientation_parity(&self) -> bool {
        self.corner_configuration
            .orientation
            .iter()
            .fold(ZnRing::ZERO, |co_sum, &corner_co| co_sum + corner_co)
            == ZnRing::ZERO
            && self
                .edge_configuration
                .orientation
                .iter()
                .fold(ZnRing::ZERO, |eo_sum, &edge_eo| eo_sum + edge_eo)
                == ZnRing::ZERO
    }
}
#[cfg(test)]
#[allow(clippy::panic_in_result_fn)]
mod tests {
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
    fn r_4_times_is_solved_and_respects_parity() -> Result<(), String> {
        let cube = Cube3By3::from_solved("R R R R")?;
        assert!(cube.is_solved());
        assert!(cube.respects_orientation_parity());
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
    fn r_move_respects_bounds_and_touches_only_r_layer() -> Result<(), String> {
        let r = Cube3By3::from_solved("R")?;
        for c in [
            SingleCorner::Ubl,
            SingleCorner::Ufl,
            SingleCorner::Dfl,
            SingleCorner::Dbl,
        ] {
            assert_eq!(r.corner_configuration.orientation[c as usize], ZnRing::ZERO);
            assert_eq!(r.corner_configuration.permutation[c as usize], c);
        }
        for e in [
            SingleEdge::Ub,
            SingleEdge::Uf,
            SingleEdge::Ul,
            SingleEdge::Fl,
            SingleEdge::Bl,
            SingleEdge::Df,
            SingleEdge::Db,
            SingleEdge::Dl,
        ] {
            assert_eq!(r.edge_configuration.permutation[e as usize], e);
        }
        assert!(r.respects_orientation_parity());
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
        twisted.corner_configuration.orientation[SingleCorner::Ufr as usize] = ZnRing::new(1);
        let after = twisted * r;
        let mut expected = r;
        expected.corner_configuration.orientation[SingleCorner::Ubr as usize] =
            expected.corner_configuration.orientation[SingleCorner::Ubr as usize] + ZnRing::new(1);
        assert_eq!(after, expected);
        Ok(())
    }
    #[test]
    fn u_perm_repeats_after_3_applications() {
        use SingleEdge::{Uf, Ul, Ur};
        let u_perm = Cube3By3 {
            edge_configuration: Edges::cycle([[Ur, Uf, Ul]]),
            ..Default::default()
        };
        assert_eq!(u_perm.pow(3), Cube3By3::default());
    }
}
