pub(crate) mod pieces;
#[allow(clippy::wildcard_imports)]
use self::pieces::*;
use crate::{
    ops::{Inv, Pow},
    zn::ZnRing,
};
#[derive(Debug, Clone, PartialEq, Eq, Default)]
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
            self.clone() * self.pow(exponent - 1)
        }
    }
}

impl Inv for Cube3By3 {
    fn inverse(&self) -> Self {
        Cube3By3 {
            center_configuration: self.center_configuration.inverse(),
            corner_configuration: self.corner_configuration.inverse(),
            edge_configuration: self.edge_configuration.inverse(),
        }
    }
}

impl Cube3By3 {
    /// The multiplicative identity of the cube group: the solved cube
    pub const IDENTITY: Cube3By3 = Cube3By3 {
        center_configuration: Centers::IDENTITY,
        corner_configuration: Corners::IDENTITY,
        edge_configuration: Edges::IDENTITY,
    };
    pub const R: Cube3By3 = Cube3By3 {
        center_configuration: Centers::IDENTITY,
        corner_configuration: {
            let mut corners = Corners::cycle::<4, 1>([[
                SingleCorner::Ufr,
                SingleCorner::Ubr,
                SingleCorner::Dbr,
                SingleCorner::Dfr,
            ]]);
            corners.orientation = ZnRing::array([0, 1, 2, 0, 0, 1, 2, 0]);
            corners
        },
        edge_configuration: Edges::cycle([[
            SingleEdge::Fr,
            SingleEdge::Ur,
            SingleEdge::Br,
            SingleEdge::Dr,
        ]]),
    };
    //pub const Y: RubiksCube = todo!();
    #[must_use]
    pub fn is_solved(&self) -> bool {
        let rotated_self = self.clone();
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
    fn rotated_solved_is_solved() {
        let rotated_def = Cube3By3::default(); //* RubiksCube::Y.pow(fastrand::u64(0..400));
        assert!(rotated_def.is_solved());
    }

    #[test]
    fn r_4_times_is_solved_and_respects_parity() {
        let mut cube = Cube3By3::default();
        for _ in 0..4 {
            assert!(cube.respects_orientation_parity());
            cube = cube * Cube3By3::R;
        }
        assert!(cube.is_solved());
    }

    #[test]
    fn r_2_is_equal_to_r_prime_2() {
        let r2 = Cube3By3::default() * Cube3By3::R * Cube3By3::R;
        let r_prime_2 = Cube3By3::default() * Cube3By3::R.inverse() * Cube3By3::R.inverse();
        assert_eq!(r2, r_prime_2);
    }

    #[test]
    fn r_r_prime_is_solved() {
        let mut cube = Cube3By3::default();
        cube = cube * Cube3By3::R * Cube3By3::R.inverse();
        assert!(cube.is_solved());
    }

    #[test]
    fn r_constant_respects_bounds_and_touches_only_r_layer() {
        for c in [
            SingleCorner::Ubl,
            SingleCorner::Ufl,
            SingleCorner::Dfl,
            SingleCorner::Dbl,
        ] {
            assert_eq!(
                Cube3By3::R.corner_configuration.orientation[c as usize],
                ZnRing::ZERO
            );
            assert_eq!(Cube3By3::R.corner_configuration.permutation[c as usize], c);
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
            assert_eq!(Cube3By3::R.edge_configuration.permutation[e as usize], e);
        }
        assert!(Cube3By3::R.respects_orientation_parity());
    }

    #[test]
    fn mul_carries_orientation_along_with_the_piece() {
        // Pre-twist the piece at UFR, then apply R: that piece lands at UBR and
        // its twist is added to the twist R gives the UBR slot.
        let mut twisted = Cube3By3::default();
        twisted.corner_configuration.orientation[SingleCorner::Ufr as usize] = ZnRing::new(1);
        let after = twisted * Cube3By3::R;
        let mut expected = Cube3By3::R;
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
