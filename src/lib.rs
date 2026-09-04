use std::ops::Neg;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ZnRing<const N: usize>(usize);

impl<const N: usize> From<usize> for ZnRing<N> {
    fn from(integer: usize) -> Self {
        () = Self::CHECK;
        ZnRing(integer % N)
    }
}
impl<const N: usize> std::ops::Add for ZnRing<N> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        ZnRing((self.0 + rhs.0) % N)
    }
}
impl<const N: usize> ZnRing<N> {
    const CHECK: () = assert!(N >= 1);
    const fn array<const L: usize>(values: [usize; L]) -> [Self; L] {
        let mut final_array = [ZnRing(0); L];
        let mut i = 0;
        while i < L {
            final_array[i] = ZnRing(values[i]);
            i += 1;
        }
        final_array
    }
}
impl<const N: usize> Neg for ZnRing<N> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::from(N - self.0)
    }
}

const CORNERS_COUNT: usize = 8;
const EDGES_COUNT: usize = 12;
const CO_COUNT: usize = 3;
const EO_COUNT: usize = 2;
const ROTATIONS_COUNT: usize = 6 * 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
enum Corner {
    Ubl,
    Ubr,
    Ufr,
    Ufl,
    Dfl,
    Dfr,
    Dbr,
    Dbl,
}

impl Corner {
    pub const ALL: [Corner; CORNERS_COUNT] = [
        Corner::Ubl,
        Corner::Ubr,
        Corner::Ufr,
        Corner::Ufl,
        Corner::Dfl,
        Corner::Dfr,
        Corner::Dbr,
        Corner::Dbl,
    ];

    pub const fn from_index(index: usize) -> Corner {
        Self::ALL[index]
    }

    pub const fn cycle<const M: usize, const N: usize>(
        to_cycle: [[Corner; M]; N],
    ) -> [Corner; CORNERS_COUNT] {
        let mut resp = Self::ALL;
        let mut i = 0;
        while i < N {
            let mut j = 0;
            while j < M {
                resp[to_cycle[i][(j + 1) % M] as usize] = to_cycle[i][j];
                j += 1;
            }
            i += 1;
        }
        resp
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
enum Edge {
    Ub,
    Ur,
    Uf,
    Ul,
    Fl,
    Fr,
    Br,
    Bl,
    Df,
    Dr,
    Db,
    Dl,
}

impl Edge {
    pub const ALL: [Edge; EDGES_COUNT] = [
        Edge::Ub,
        Edge::Ur,
        Edge::Uf,
        Edge::Ul,
        Edge::Fl,
        Edge::Fr,
        Edge::Br,
        Edge::Bl,
        Edge::Df,
        Edge::Dr,
        Edge::Db,
        Edge::Dl,
    ];

    pub const fn from_index(index: usize) -> Edge {
        Self::ALL[index]
    }

    pub const fn cycle<const M: usize, const N: usize>(
        to_cycle: [[Edge; M]; N],
    ) -> [Edge; EDGES_COUNT] {
        let mut resp = Self::ALL;
        let mut i = 0;
        while i < N {
            let mut j = 0;
            while j < M {
                resp[to_cycle[i][(j + 1) % M] as usize] = to_cycle[i][j];
                j += 1;
            }
            i += 1;
        }
        resp
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RubiksCube {
    /// TODO: Decide rotation convention, 0 is white top green front
    rotation: ZnRing<ROTATIONS_COUNT>,
    /// Follows clockwise convention: 0 is oriented, 1 is 1 turn clockwise, 2 is 1 turn counterclockwise
    corner_orientation: [ZnRing<CO_COUNT>; CORNERS_COUNT],
    /// 0 is oriented, 1 is misoriented
    edge_orientation: [ZnRing<EO_COUNT>; EDGES_COUNT],
    /// Each field `i` is the corner on position `i`, `i` indexes in blind standard order, i.e.
    /// `[UBL, UBR, UFR, UFL, DFL, DFR, DBR, DBL]`
    corner_permutation: [Corner; CORNERS_COUNT],
    /// Each field `i` is the edge on position `i`, `i` indexes clockwise per layer
    /// `[UB, UR, UF, UL, FL, FR, BR, BL, DF, DR, DB, DL]`
    edge_permutation: [Edge; EDGES_COUNT],
}

impl Default for RubiksCube {
    fn default() -> Self {
        RubiksCube {
            rotation: ZnRing::<ROTATIONS_COUNT>(0),
            corner_orientation: [ZnRing::<CO_COUNT>(0); CORNERS_COUNT],
            edge_orientation: [ZnRing::<EO_COUNT>(0); EDGES_COUNT],
            corner_permutation: Corner::ALL,
            edge_permutation: Edge::ALL,
        }
    }
}

impl std::ops::Mul for RubiksCube {
    type Output = Self;
    /// Applies the permutation the second cube to the first cube
    /// IMPORTANT: associative, but non-commutative
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, to_be_aplied: Self) -> Self::Output {
        let mut resp = self.clone();
        for i in 0..CORNERS_COUNT {
            resp.corner_permutation[i] =
                self.corner_permutation[to_be_aplied.corner_permutation[i] as usize];
            resp.corner_orientation[i] = self.corner_orientation
                [to_be_aplied.corner_permutation[i] as usize]
                + to_be_aplied.corner_orientation[i];
        }
        for j in 0..EDGES_COUNT {
            resp.edge_permutation[j] =
                self.edge_permutation[to_be_aplied.edge_permutation[j] as usize];
            resp.edge_orientation[j] = self.edge_orientation
                [to_be_aplied.edge_permutation[j] as usize]
                + to_be_aplied.edge_orientation[j];
        }
        resp
    }
}

pub trait Inv
where
    Self: std::marker::Sized,
{
    /// Inverts a state, possibly fallibly
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

impl Pow for RubiksCube {
    type Output = Self;
    fn pow(&self, exponent: u64) -> Self::Output {
        if exponent == 0 {
            Self::default()
        } else {
            self.clone() * self.pow(exponent - 1)
        }
    }
}

impl Inv for RubiksCube {
    fn inverse(&self) -> Self {
        let mut inv = RubiksCube::default();
        for i in 0..EDGES_COUNT {
            inv.edge_permutation[self.edge_permutation[i] as usize] = Edge::from_index(i);
            inv.edge_orientation[self.edge_permutation[i] as usize] = -self.edge_orientation[i];
        }
        for i in 0..CORNERS_COUNT {
            inv.corner_permutation[self.corner_permutation[i] as usize] = Corner::from_index(i);
            inv.corner_orientation[self.corner_permutation[i] as usize] =
                -self.corner_orientation[i];
        }
        inv
    }
}

impl RubiksCube {
    pub const R: RubiksCube = RubiksCube {
        rotation: ZnRing(0),
        corner_orientation: ZnRing::array([0, 1, 2, 0, 0, 1, 2, 0]),
        edge_orientation: ZnRing::array([0; EDGES_COUNT]),
        corner_permutation: {
            use Corner::{Dbr, Dfr, Ubr, Ufr};
            Corner::cycle([[Ufr, Ubr, Dbr, Dfr]])
        },
        edge_permutation: {
            use Edge::{Br, Dr, Fr, Ur};
            Edge::cycle([[Fr, Ur, Br, Dr]])
        },
    };
    /// Can only panic if internal logic is wrong
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn is_solved(&self) -> bool {
        self.corner_orientation == [ZnRing::<3>(0); CORNERS_COUNT]
            && self.edge_orientation == [ZnRing::<2>(0); EDGES_COUNT]
            && self.corner_permutation == Corner::ALL
            && self.edge_permutation == Edge::ALL
    }
    #[must_use]
    pub fn respects_orientation_parity(&self) -> bool {
        self.corner_orientation
            .iter()
            .fold(ZnRing::<CO_COUNT>::default(), |co_sum, &corner_co| {
                co_sum + corner_co
            })
            == ZnRing::default()
            && self
                .edge_orientation
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
        assert!(RubiksCube::default().is_solved());
    }

    #[test]
    fn rotated_solved_is_solved() {
        let rotated_def = RubiksCube {
            rotation: ZnRing::<ROTATIONS_COUNT>(fastrand::usize(0..400)),
            ..Default::default()
        };
        assert!(rotated_def.is_solved());
    }

    #[test]
    fn r_4_times_is_solved_and_respects_parity() {
        let mut cube = RubiksCube::default();
        for _ in 0..4 {
            assert!(cube.respects_orientation_parity());
            cube = cube * RubiksCube::R;
        }
        assert!(cube.is_solved());
    }

    #[test]
    fn r_2_is_equal_to_r_prime_2() {
        let r2 = RubiksCube::default() * RubiksCube::R * RubiksCube::R;
        let r_prime_2 = RubiksCube::default() * RubiksCube::R.inverse() * RubiksCube::R.inverse();
        assert_eq!(r2, r_prime_2);
    }

    #[test]
    fn r_r_prime_is_solved() {
        let mut cube = RubiksCube::default();
        cube = cube * RubiksCube::R * RubiksCube::R.inverse();
        assert!(cube.is_solved());
    }

    #[test]
    fn corner_and_edge_all_match_discriminants() {
        for (i, c) in Corner::ALL.iter().enumerate() {
            assert_eq!(*c as usize, i);
            assert_eq!(Corner::from_index(i), *c);
        }
        for (i, e) in Edge::ALL.iter().enumerate() {
            assert_eq!(*e as usize, i);
            assert_eq!(Edge::from_index(i), *e);
        }
    }

    #[test]
    fn cycle_is_a_permutation_and_moves_pieces_forward_and_leaves_rest() {
        use Corner::{Dbr, Dfr, Ubr, Ufr};
        let mut perm = Corner::cycle([[Ufr, Ubr, Dbr, Dfr]]);
        assert_eq!(perm[Ubr as usize], Ufr);
        assert_eq!(perm[Dbr as usize], Ubr);
        assert_eq!(perm[Dfr as usize], Dbr);
        assert_eq!(perm[Ufr as usize], Dfr);
        for c in [Corner::Ubl, Corner::Ufl, Corner::Dfl, Corner::Dbl] {
            assert_eq!(perm[c as usize], c);
        }
        perm.sort();
        assert_eq!(perm, Corner::ALL);
    }

    #[test]
    fn cycle_with_disjoint_cycles_is_a_permutation() {
        use Edge::{Dl, Dr, Ub, Uf};
        let mut perm = Edge::cycle([[Ub, Uf], [Dl, Dr]]);
        assert_eq!(perm[Uf as usize], Ub);
        assert_eq!(perm[Ub as usize], Uf);
        perm.sort();
        assert_eq!(perm, Edge::ALL);
    }

    #[test]
    fn r_constant_respects_bounds_and_touches_only_r_layer() {
        for co in RubiksCube::R.corner_orientation {
            assert!(co.0 < CO_COUNT);
        }
        for eo in RubiksCube::R.edge_orientation {
            assert!(eo.0 < EO_COUNT);
        }
        for c in [Corner::Ubl, Corner::Ufl, Corner::Dfl, Corner::Dbl] {
            assert_eq!(RubiksCube::R.corner_orientation[c as usize], ZnRing(0));
            assert_eq!(RubiksCube::R.corner_permutation[c as usize], c);
        }
        for e in [
            Edge::Ub,
            Edge::Uf,
            Edge::Ul,
            Edge::Fl,
            Edge::Bl,
            Edge::Df,
            Edge::Db,
            Edge::Dl,
        ] {
            assert_eq!(RubiksCube::R.edge_permutation[e as usize], e);
        }
        assert!(RubiksCube::R.respects_orientation_parity());
    }

    #[test]
    fn mul_carries_orientation_along_with_the_piece() {
        // Pre-twist the piece at UFR, then apply R: that piece lands at UBR and
        // its twist is added to the twist R gives the UBR slot.
        let mut twisted = RubiksCube::default();
        twisted.corner_orientation[Corner::Ufr as usize] = ZnRing(1);
        let after = twisted * RubiksCube::R;
        let mut expected = RubiksCube::R;
        expected.corner_orientation[Corner::Ubr as usize] =
            expected.corner_orientation[Corner::Ubr as usize] + ZnRing(1);
        assert_eq!(after, expected);
    }

    #[test]
    fn zn_ring_reduces_wraps_and_negates() {
        assert_eq!(ZnRing::<3>::from(7), ZnRing(1));
        assert_eq!(ZnRing::<3>(2) + ZnRing(2), ZnRing(1));
        for x in 0..CO_COUNT {
            let v = ZnRing::<CO_COUNT>::from(x);
            assert_eq!(v + (-v), ZnRing(0));
        }
    }
    #[test]
    fn u_perm_repeats_after_3_applications() {
        use Edge::{Uf, Ul, Ur};
        let u_perm = RubiksCube {
            edge_permutation: Edge::cycle([[Ur, Uf, Ul]]),
            ..Default::default()
        };
        assert_eq!(u_perm.pow(3), RubiksCube::default());
    }
}
