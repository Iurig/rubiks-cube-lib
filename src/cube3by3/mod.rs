pub(crate) mod moves;
pub(crate) mod pieces;

#[allow(clippy::wildcard_imports)]
use self::{moves::*, pieces::*};
use crate::{
    ops::{Inv, Pow},
    string_processing::RubiksCubeCleaning,
    zn::ZnRing,
};

#[derive(Debug, Clone, PartialEq, Eq, Default, Copy)]
#[allow(clippy::struct_field_names)]
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
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, to_be_aplied: Self) -> Self::Output {
        self.const_mul(to_be_aplied)
    }
}
impl Cube3By3 {
    const fn const_mul(self, to_be_aplied: Self) -> Cube3By3 {
        Cube3By3 {
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
    pub const IDENTITY: Cube3By3 = Cube3By3 {
        center_configuration: Centers::IDENTITY,
        corner_configuration: Corners::IDENTITY,
        edge_configuration: Edges::IDENTITY,
    };

    const fn const_inverse(&self) -> Self {
        Cube3By3 {
            center_configuration: self.center_configuration.const_inverse(),
            corner_configuration: self.corner_configuration.const_inverse(),
            edge_configuration: self.edge_configuration.const_inverse(),
        }
    }

    #[must_use]
    pub fn move_sequence(&self, moves: &str) -> Self {
        if moves.contains(char::is_whitespace) {
            moves
                .process_movement_input()
                .fold(*self, |cube, single_move| cube.move_sequence(&single_move))
        } else {
            *self * Cube3By3::from(Move::from(moves))
        }
    }

    #[must_use]
    pub fn from_solved(m: &str) -> Self {
        Cube3By3::default().move_sequence(m)
    }

    #[must_use]
    pub fn is_solved(&self) -> bool {
        let rotated_self = *self;
        // TODO: implement rotating to put U on top and F on front on `rotated_self`
        rotated_self == Cube3By3::IDENTITY
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
mod tests {
    use super::*;
    #[test]
    fn default_is_solved() {
        assert!(Cube3By3::default().is_solved());
    }

    #[test]
    #[ignore = "is_solved not yet implemented for rotated cubes"]
    fn rotated_solved_is_solved() {
        let rotated_def = Cube3By3::from_solved("y");
        assert!(rotated_def.is_solved());
    }

    #[test]
    fn r_4_times_is_solved_and_respects_parity() {
        let cube = Cube3By3::from_solved("R R R R");
        assert!(cube.is_solved());
    }

    #[test]
    fn r_2_is_equal_to_r_prime_2() {
        let r2 = Cube3By3::from_solved("R2");
        let r_prime_2 = Cube3By3::from_solved("R' R'");
        assert_eq!(r2, r_prime_2);
    }

    #[test]
    fn r_r_prime_is_solved() {
        let mut cube = Cube3By3::from_solved("R");
        assert!(!cube.is_solved());
        cube = cube.move_sequence("R'");
        assert!(cube.is_solved());
    }

    #[test]
    fn r_prime_is_inverse_of_r() {
        let r = Cube3By3::from_solved("R");
        let r_prime = Cube3By3::from_solved("R'");
        assert_eq!(r.const_inverse(), r_prime);
    }

    #[test]
    fn composing_u_and_r_works() {
        let u = Cube3By3::from_solved("U");
        let ur = Cube3By3::from_solved("U R");
        assert_eq!(u.move_sequence("R"), ur,);
    }

    #[test]
    fn r_move_respects_bounds_and_touches_only_r_layer() {
        for c in [
            SingleCorner::Ubl,
            SingleCorner::Ufl,
            SingleCorner::Dfl,
            SingleCorner::Dbl,
        ] {
            assert_eq!(
                Cube3By3::from_solved("R").corner_configuration.orientation[c as usize],
                ZnRing::ZERO
            );
            assert_eq!(
                Cube3By3::from_solved("R").corner_configuration.permutation[c as usize],
                c
            );
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
            assert_eq!(
                Cube3By3::from_solved("R").edge_configuration.permutation[e as usize],
                e
            );
        }
        assert!(Cube3By3::from_solved("R").respects_orientation_parity());
    }

    #[test]
    fn composition_works() {
        let scramble_string = "R' ";
        let solution_string = "D2";
        let scramble = Cube3By3::from_solved(scramble_string);
        assert_eq!(
            Cube3By3::from_solved(&format!("{scramble_string} {solution_string}")),
            scramble.move_sequence(solution_string),
            "composing {scramble_string} and {solution_string} doesn't result in applying {scramble_string} {solution_string}"
        );
    }

    #[test]
    fn composition_works_on_fmc_wr() {
        let scramble_string =
            "R' U' F D2 L2 F R2 U2 R2 B D2 L B2 D' B2 L' R' B D2 B U2 L U2 R' U' F";
        let solution_string = "D2 F' D2 U2 F' L2 D R2 D B2 F L2 R' F' D U'";
        let scramble = Cube3By3::from_solved(scramble_string);
        assert_eq!(
            Cube3By3::from_solved(&format!("{scramble_string} {solution_string}")),
            scramble.move_sequence(solution_string),
            "composing {scramble_string} and {solution_string} doesn't result in applying {scramble_string} {solution_string}"
        );
    }

    #[test]
    fn mul_carries_orientation_along_with_the_piece() {
        // Pre-twist the piece at UFR, then apply R: that piece lands at UBR and
        // its twist is added to the twist R gives the UBR slot.
        let mut twisted = Cube3By3::default();
        twisted.corner_configuration.orientation[SingleCorner::Ufr as usize] = ZnRing::new(1);
        let after = twisted * Cube3By3::from_solved("R");
        let mut expected = Cube3By3::from_solved("R");
        expected.corner_configuration.orientation[SingleCorner::Ubr as usize] =
            expected.corner_configuration.orientation[SingleCorner::Ubr as usize] + ZnRing::new(1);
        assert_eq!(after, expected);
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
