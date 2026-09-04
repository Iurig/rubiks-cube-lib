pub mod zn;

use zn::ZnRing;

/// # Safety
/// Implementors must be fieldless `#[repr(u8)]` enums whose discriminants
/// are `0..N`, in the same order as `ALL`.
pub unsafe trait SinglePiece<const N: usize>: Copy + Eq {
    const ALL: [Self; N];
}
#[must_use]
pub fn from_index<P, const N: usize>(index: usize) -> P
where
    P: SinglePiece<N>,
{
    P::ALL[index]
}
#[must_use]
pub const fn index<P, const N: usize>(piece: P) -> usize
where
    P: SinglePiece<N>,
{
    unsafe { (&raw const piece).cast::<u8>().read() as usize }
}
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PieceConfiguration<P, const N: usize, const O: usize> {
    permutation: [P; N],
    orientation: [ZnRing<O>; N],
}
impl<P, const N: usize, const O: usize> Default for PieceConfiguration<P, N, O>
where
    P: SinglePiece<N>,
{
    fn default() -> Self {
        PieceConfiguration {
            permutation: P::ALL,
            orientation: [ZnRing::ZERO; N],
        }
    }
}
impl<P, const N: usize, const O: usize> Inv for PieceConfiguration<P, N, O>
where
    P: SinglePiece<N>,
{
    fn inverse(&self) -> Self {
        let mut inv = PieceConfiguration::default();
        for i in 0..N {
            inv.permutation[index(self.permutation[i])] = P::ALL[i];
            inv.orientation[index(self.permutation[i])] = -self.orientation[i];
        }
        inv
    }
}
impl<P, const N: usize, const O: usize> PieceConfiguration<P, N, O>
where
    P: SinglePiece<N>,
{
    /// Compose permutations done by `self` with `other`
    #[must_use]
    pub fn then(&self, other: &Self) -> Self {
        let mut composed = Self::default();
        for i in 0..N {
            composed.permutation[i] = self.permutation[index(other.permutation[i])];
            composed.orientation[i] =
                self.orientation[index(other.permutation[i])] + other.orientation[i];
        }
        composed
    }

    pub const fn cycle<const CYCLE_SIZE: usize, const CYCLE_AMOUNT: usize>(
        to_cycle: [[P; CYCLE_SIZE]; CYCLE_AMOUNT],
    ) -> Self {
        let mut resp = PieceConfiguration {
            permutation: P::ALL,
            orientation: [ZnRing::ZERO; N],
        };
        let mut i = 0;
        while i < CYCLE_AMOUNT {
            let mut j = 0;
            while j < CYCLE_SIZE {
                resp.permutation[index(to_cycle[i][(j + 1) % CYCLE_SIZE])] = to_cycle[i][j];
                j += 1;
            }
            i += 1;
        }
        resp
    }
}

const CENTERS_COUNT: usize = 6;
const CORNERS_COUNT: usize = 8;
const EDGES_COUNT: usize = 12;
const CENTER_ORIENTATION_COUNT: usize = 1;
const CO_COUNT: usize = 3;
const EO_COUNT: usize = 2;

macro_rules! new_piece {
    ($type_name:ident, $amount:ident, [$($p:ident),+]) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[repr(u8)]
        enum $type_name {
            $($p),+
        }
        unsafe impl SinglePiece<$amount> for $type_name {
            const ALL: [Self; $amount] = [
                $($type_name::$p),+
            ];
        }
    };
}
// Centers are considered in blind standard order, i.e. `[U, F, R, B, L, D]`
new_piece!(SingleCenter, CENTERS_COUNT, [U, F, R, B, L, D]);
// Corners are considered in blind standard order, i.e. `[UBL, UBR, UFR, UFL, DFL, DFR, DBR, DBL]`
new_piece!(
    SingleCorner,
    CORNERS_COUNT,
    [Ubl, Ubr, Ufr, Ufl, Dfl, Dfr, Dbr, Dbl]
);
// Edges are considered clockwise per layer, i.e. `[UB, UR, UF, UL, FL, FR, BR, BL, DF, DR, DB, DL]`
new_piece!(
    SingleEdge,
    EDGES_COUNT,
    [Ub, Ur, Uf, Ul, Fl, Fr, Br, Bl, Df, Dr, Db, Dl]
);

type Centers = PieceConfiguration<SingleCenter, CENTERS_COUNT, CENTER_ORIENTATION_COUNT>;
type Corners = PieceConfiguration<SingleCorner, CORNERS_COUNT, CO_COUNT>;
type Edges = PieceConfiguration<SingleEdge, EDGES_COUNT, EO_COUNT>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub struct Cube3By3 {
    /// `CENTER_ORIENTATION_COUNT` is 1, centers are considered without orientation
    center_configuration: Centers,
    /// Corner orientation is done with the convention of clockwise rotations from white/yellow sticker being in the U/B faces
    corner_configuration: Corners,
    /// 0 is oriented, 1 is misoriented
    edge_configuration: Edges,
}

impl Default for Cube3By3 {
    fn default() -> Self {
        Cube3By3 {
            center_configuration: PieceConfiguration {
                permutation: SingleCenter::ALL,
                orientation: [ZnRing::<CENTER_ORIENTATION_COUNT>::ZERO; CENTERS_COUNT],
            },
            corner_configuration: PieceConfiguration {
                permutation: SingleCorner::ALL,
                orientation: [ZnRing::<CO_COUNT>::ZERO; CORNERS_COUNT],
            },
            edge_configuration: PieceConfiguration {
                permutation: SingleEdge::ALL,
                orientation: [ZnRing::<EO_COUNT>::ZERO; EDGES_COUNT],
            },
        }
    }
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

pub trait Inv
where
    Self: std::marker::Sized,
{
    /// Inverts a state multiplicatively, possibly fallibly
    ///
    /// # Exemples
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

impl Pow for Cube3By3 {
    type Output = Self;
    fn pow(&self, exponent: u64) -> Self::Output {
        if exponent == 0 {
            Self::default()
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
    pub const R: Cube3By3 = Cube3By3 {
        center_configuration: Centers {
            permutation: SingleCenter::ALL,
            orientation: [ZnRing::ZERO; CENTERS_COUNT],
        },
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
        let mut rotated_self = self.clone();
        // TODO: implement rotating to put U on top and F on front on `rotated_self`
        rotated_self.corner_configuration == PieceConfiguration::default()
            && rotated_self.edge_configuration == PieceConfiguration::default()
    }
    #[must_use]
    pub fn respects_orientation_parity(&self) -> bool {
        self.corner_configuration
            .orientation
            .iter()
            .fold(ZnRing::<CO_COUNT>::default(), |co_sum, &corner_co| {
                co_sum + corner_co
            })
            == ZnRing::default()
            && self
                .edge_configuration
                .orientation
                .iter()
                .fold(ZnRing::<EO_COUNT>::default(), |eo_sum, &edge_eo| {
                    eo_sum + edge_eo
                })
                == ZnRing::default()
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
    fn corner_and_edge_all_match_discriminants() {
        for (i, c) in SingleCorner::ALL.iter().enumerate() {
            assert_eq!(*c as usize, i);
            assert_eq!(from_index::<SingleCorner, _>(i), *c);
        }
        for (i, e) in SingleEdge::ALL.iter().enumerate() {
            assert_eq!(*e as usize, i);
            assert_eq!(from_index::<SingleEdge, _>(i), *e);
        }
    }

    #[test]
    fn cycle_is_a_permutation_and_moves_pieces_forward_and_leaves_rest() {
        use SingleCorner::{Dbr, Dfr, Ubr, Ufr};
        let mut perm = Corners::cycle([[Ufr, Ubr, Dbr, Dfr]]).permutation;
        assert_eq!(perm[index(Ubr)], Ufr);
        assert_eq!(perm[index(Dbr)], Ubr);
        assert_eq!(perm[index(Dfr)], Dbr);
        assert_eq!(perm[index(Ufr)], Dfr);
        for c in [
            SingleCorner::Ubl,
            SingleCorner::Ufl,
            SingleCorner::Dfl,
            SingleCorner::Dbl,
        ] {
            assert_eq!(perm[c as usize], c);
        }
        perm.sort();
        assert_eq!(perm, SingleCorner::ALL);
    }

    #[test]
    fn cycle_with_disjoint_cycles_is_a_permutation() {
        use SingleEdge::{Dl, Dr, Ub, Uf};
        let mut perm = Edges::cycle::<2, 2>([[Ub, Uf], [Dl, Dr]]).permutation;
        assert_eq!(perm[Uf as usize], Ub);
        assert_eq!(perm[Ub as usize], Uf);
        perm.sort();
        assert_eq!(perm, SingleEdge::ALL);
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
    fn zn_ring_reduces_wraps_and_negates() {
        assert_eq!(ZnRing::<3>::from(7), ZnRing::new(1));
        assert_eq!(ZnRing::<3>::new(2) + ZnRing::new(2), ZnRing::new(1));
        for x in 0..CO_COUNT {
            let v = ZnRing::<CO_COUNT>::from(x);
            assert_eq!(v + (-v), ZnRing::new(0));
        }
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
