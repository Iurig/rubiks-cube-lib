pub mod algs;
pub mod facelets;
pub mod moves;
pub mod pieces;

use std::ops::Neg;

#[allow(
    clippy::wildcard_imports,
    reason = "`allow`, not `expect`: the lint is skipped when the library is compiled with `cfg(test)`"
)]
use self::{moves::*, pieces::*};

#[allow(
    clippy::enum_glob_use,
    reason = "`allow`, not `expect`: the lint is skipped when the library is compiled with `cfg(test)`"
)]
use crate::{
    ops::{Inv, Pow},
    puzzles::{Puzzle, cube3by3::moves::MoveModifier::*},
    zn::Zn,
};

/// A 3x3x3 cube state; `a * b` applies `a` then `b`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Copy, Hash)]
#[expect(
    clippy::struct_field_names,
    reason = "_configuration makes clear what all variables are"
)]
pub struct Cube3x3 {
    /// `CENTER_ORIENTATION_COUNT` is 1, centers are considered without orientation
    center_configuration: CenterConfiguration,
    /// Corner orientation is done with the convention of clockwise rotations from white/yellow sticker being in the faces U and D
    corner_configuration: CornerConfiguration,
    /// 0 is oriented, 1 is misoriented
    edge_configuration: EdgeConfiguration,
}
macro_rules! unify_pieces {
    ($($piece_type:ident: [$($p:ident),+]),+ $(,)?) => {
        [
            $($(Pieces3x3::$piece_type($piece_type::$p)),+),+
        ]
    };
}
impl Puzzle for Cube3x3 {
    type Piece = Pieces3x3;
    type Moves = Move3x3;

    const ALL_PIECES: &'static [Self::Piece] = unify_pieces!(
        Center: [U, F, R, B, L, D],
        Corner: [Ubl, Ubr, Ufr, Ufl, Dfl, Dfr, Dbr, Dbl],
        Edge: [Ub, Ur, Uf, Ul, Fl, Fr, Br, Bl, Df, Dr, Db, Dl],
    )
    .as_slice();

    const ALL_MOVES: &'static [Self::Moves] = {
        #[allow(
            clippy::enum_glob_use,
            reason = "`allow`, not `expect`: the lint is skipped when the library is compiled with `cfg(test)`"
        )]
        use crate::{Center::*, puzzles::cube3by3::moves::MovablePart::*};
        use moves::Move3x3;
        [
            Move3x3::new(Face(F), Clockwise),
            Move3x3::new(Face(F), CounterClockwise),
            Move3x3::new(Face(F), Double),
            Move3x3::new(Face(U), Clockwise),
            Move3x3::new(Face(U), CounterClockwise),
            Move3x3::new(Face(U), Double),
            Move3x3::new(Face(R), Clockwise),
            Move3x3::new(Face(R), CounterClockwise),
            Move3x3::new(Face(R), Double),
            Move3x3::new(Face(L), Clockwise),
            Move3x3::new(Face(L), CounterClockwise),
            Move3x3::new(Face(L), Double),
            Move3x3::new(Face(D), Clockwise),
            Move3x3::new(Face(D), CounterClockwise),
            Move3x3::new(Face(D), Double),
            Move3x3::new(Face(B), Clockwise),
            Move3x3::new(Face(B), CounterClockwise),
            Move3x3::new(Face(B), Double),
            Move3x3::new(Slice(Slices::M), Clockwise),
            Move3x3::new(Slice(Slices::M), CounterClockwise),
            Move3x3::new(Slice(Slices::M), Double),
            Move3x3::new(Slice(Slices::E), Clockwise),
            Move3x3::new(Slice(Slices::E), CounterClockwise),
            Move3x3::new(Slice(Slices::E), Double),
            Move3x3::new(Slice(Slices::S), Clockwise),
            Move3x3::new(Slice(Slices::S), CounterClockwise),
            Move3x3::new(Slice(Slices::S), Double),
            Move3x3::new(Wide(F), Clockwise),
            Move3x3::new(Wide(F), CounterClockwise),
            Move3x3::new(Wide(F), Double),
            Move3x3::new(Wide(U), Clockwise),
            Move3x3::new(Wide(U), CounterClockwise),
            Move3x3::new(Wide(U), Double),
            Move3x3::new(Wide(R), Clockwise),
            Move3x3::new(Wide(R), CounterClockwise),
            Move3x3::new(Wide(R), Double),
            Move3x3::new(Wide(L), Clockwise),
            Move3x3::new(Wide(L), CounterClockwise),
            Move3x3::new(Wide(L), Double),
            Move3x3::new(Wide(D), Clockwise),
            Move3x3::new(Wide(D), CounterClockwise),
            Move3x3::new(Wide(D), Double),
            Move3x3::new(Wide(B), Clockwise),
            Move3x3::new(Wide(B), CounterClockwise),
            Move3x3::new(Wide(B), Double),
        ]
        .as_slice()
    };

    /// Whether this state is a (possibly rotated) solved cube.
    fn is_solved(&self) -> bool {
        self.rotated_until_solved_centers() == Some(Self::default())
    }

    fn piece_location(&self, piece: &Self::Piece) -> Self::Piece {
        Self::ALL_PIECES
            .iter()
            .find(|&slot| self.piece_at(slot) == *piece)
            .copied()
            .expect("All Cubes should have all pieces somewhere")
    }

    fn piece_at(&self, slot: &Self::Piece) -> Self::Piece {
        match slot {
            Self::Piece::Corner(co) => Pieces3x3::Corner(self.corner_configuration.piece_at(co)),
            Self::Piece::Edge(ed) => Pieces3x3::Edge(self.edge_configuration.piece_at(ed)),
            Self::Piece::Center(ce) => Pieces3x3::Center(self.center_configuration.piece_at(ce)),
        }
    }

    fn orientation_at(&self, slot: &Self::Piece) -> usize {
        match slot {
            Self::Piece::Corner(co) => self.corner_configuration.orientation_at(co).value(),
            Self::Piece::Edge(ed) => self.edge_configuration.orientation_at(ed).value(),
            Self::Piece::Center(ce) => self.center_configuration.orientation_at(ce).value(),
        }
    }

    fn random_state_with_seed(seed: u64) -> Self {
        let mut rng = fastrand::Rng::with_seed(seed);
        let mut attempt = Self {
            corner_configuration: CornerConfiguration::random_state(&mut rng),
            edge_configuration: EdgeConfiguration::random_state(&mut rng),
            ..Default::default()
        };
        attempt.corner_configuration.orientation[0] = attempt.corner_configuration.orientation[0]
            + attempt
                .corner_configuration
                .orientation
                .iter()
                .fold(Zn::ZERO, |sum, next| sum + *next)
                .neg();
        attempt.edge_configuration.orientation[0] = attempt.edge_configuration.orientation[0]
            + attempt
                .edge_configuration
                .orientation
                .iter()
                .fold(Zn::ZERO, |sum, next| sum + *next)
                .neg();
        if !(attempt.is_reachable()) {
            attempt.edge_configuration.permutation.swap(0, 1);
        }
        attempt
    }
}

impl std::ops::Mul for Cube3x3 {
    type Output = Self;
    /// Applies the state (permutations and orientations) that is the second argument to the first argument, which is a cube.
    ///
    /// IMPORTANT: associative, but non-commutative
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            center_configuration: self.center_configuration.then(&rhs.center_configuration),
            corner_configuration: self.corner_configuration.then(&rhs.corner_configuration),
            edge_configuration: self.edge_configuration.then(&rhs.edge_configuration),
        }
    }
}

impl std::ops::Mul<Move3x3> for Cube3x3 {
    type Output = Self;
    fn mul(self, m: Move3x3) -> Self::Output {
        self * Self::from(m)
    }
}

impl Pow for Cube3x3 {
    fn identity() -> Self {
        Self::IDENTITY
    }
}

impl Inv for Cube3x3 {
    fn inverse(&self) -> Self {
        Self {
            center_configuration: self.center_configuration.inverse(),
            corner_configuration: self.corner_configuration.inverse(),
            edge_configuration: self.edge_configuration.inverse(),
        }
    }
}

impl Cube3x3 {
    /// The multiplicative identity of the cube group: the solved cube
    pub const IDENTITY: Self = Self {
        center_configuration: CenterConfiguration::IDENTITY,
        corner_configuration: CornerConfiguration::IDENTITY,
        edge_configuration: EdgeConfiguration::IDENTITY,
    };

    /// The corner permutation and twists.
    #[must_use]
    pub const fn corners(&self) -> &CornerConfiguration {
        &self.corner_configuration
    }
    /// The edge permutation and flips.
    #[must_use]
    pub const fn edges(&self) -> &EdgeConfiguration {
        &self.edge_configuration
    }
    /// The center permutation. For consistency, acompanied by a `Zn::ZERO` orientation
    #[must_use]
    pub const fn centers(&self) -> &CenterConfiguration {
        &self.center_configuration
    }

    /// Applies a move sequence to this cube state, in order, from a `&str`
    ///
    /// # Errors
    ///
    /// Errors when a whitespace separated &str outside of comments is not parseable
    /// as a move; the error names that &str and its line and position, both counted
    /// from 1, using the type [`ParseSequenceError`]
    pub fn move_sequence(&self, moves: &str) -> Result<Self, ParseSequenceError> {
        Move3x3::sequence(moves).try_fold(*self, |cube, m| Ok(cube * Self::from(m?)))
    }

    /// Applies a move sequence to the solved cube.
    ///
    /// # Errors
    ///
    /// Same as [`Self::move_sequence`].
    pub fn from_solved(m: &str) -> Result<Self, ParseSequenceError> {
        Self::default().move_sequence(m)
    }

    fn rotated_until_solved_centers(&self) -> Option<Self> {
        let mut rotated_self = *self;
        if ![
            rotated_self.centers().piece_at(&Center::F),
            rotated_self.centers().piece_at(&Center::U),
            rotated_self.centers().piece_at(&Center::B),
            rotated_self.centers().piece_at(&Center::D),
        ]
        .contains(&Faces::F)
        {
            rotated_self = rotated_self
                * Move3x3::new(MovablePart::Rotation(Rotations::y), MoveModifier::Clockwise);
        }
        for _ in 0..4 {
            if rotated_self.centers().piece_at(&Faces::F) != Faces::F {
                rotated_self = rotated_self
                    * Move3x3::new(MovablePart::Rotation(Rotations::x), MoveModifier::Clockwise);
            }
        }
        for _ in 0..4 {
            if rotated_self.centers().piece_at(&Faces::U) != Faces::U {
                rotated_self = rotated_self
                    * Move3x3::new(MovablePart::Rotation(Rotations::z), MoveModifier::Clockwise);
            }
        }

        (rotated_self.center_configuration == CenterConfiguration::default())
            .then_some(rotated_self)
    }

    /// Whether some move sequence produces this state from the solved cube.
    ///
    /// Checks for four invariants: that centers are solved with respect to each other, that edge flips are even,
    /// that corner twists are divisable by 3, and that an even number of 2-swaps reaches the permutation of the
    /// pieces.
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
        self.corners().orientation_sum() == Zn::ZERO
    }

    /// Edge flips sum to zero mod 2: a face turn flips an even number of
    /// edges, so a lone flipped edge is unreachable.
    fn flips_cancel(&self) -> bool {
        self.edges().orientation_sum() == Zn::ZERO
    }

    /// The permutation parities of corners, edges, and centers sum to zero
    /// mod 2. A face turn is odd on corners and edges; a slice turn is odd on
    /// edges and centers. Every move flips exactly two of the three, so the
    /// sum stays zero. A two-way corner/edge check would reject a lone `M`.
    fn parities_cancel(&self) -> bool {
        self.corners().parity() + self.edges().parity() + self.centers().parity() == Zn::ZERO
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

    use std::error::Error;

    #[test]
    fn hundred_random_states_are_reachable() {
        for seed in 0..100 {
            assert!(
                Cube3x3::random_state_with_seed(seed).is_reachable(),
                "the state from seed {seed} is not reachable"
            );
        }
    }

    #[test]
    fn default_is_solved() {
        assert!(Cube3x3::default().is_solved());
    }

    #[test]
    fn y_rotated_solved_is_solved() -> Result<(), Box<dyn Error>> {
        let rotated_def = Cube3x3::from_solved("y")?;
        assert!(rotated_def.is_solved());
        Ok(())
    }

    #[test]
    fn rotated_solved_is_solved() -> Result<(), Box<dyn Error>> {
        let rotated_def = Cube3x3::from_solved("y z y z x2 z2")?;
        assert!(rotated_def.is_solved());
        Ok(())
    }

    #[test]
    fn center_3_cyle_isnt_reachable_even_if_respects_parity() {
        assert!(
            !Cube3x3 {
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
            !Cube3x3 {
                corner_configuration: CornerConfiguration {
                    orientation: Zn::array([1, 0, 0, 0, 0, 0, 0, 0,]),
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
            !Cube3x3 {
                edge_configuration: EdgeConfiguration {
                    orientation: Zn::array([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,]),
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
            !Cube3x3 {
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
        assert!(Cube3x3::from_solved("Q").is_err());
        assert!(Cube3x3::from_solved("R Q U").is_err());
        assert!(Cube3x3::from_solved("R3").is_err());
    }

    #[test]
    fn rotations_match_moves() -> Result<(), Box<dyn Error>> {
        assert_eq!(Cube3x3::from_solved("y")?, Cube3x3::from_solved("U E' D'")?);
        assert_eq!(Cube3x3::from_solved("z")?, Cube3x3::from_solved("F S B'")?);
        assert_eq!(Cube3x3::from_solved("x")?, Cube3x3::from_solved("R M' L'")?);
        Ok(())
    }

    #[test]
    fn all_axes_rotated_solved_is_solved_but_not_after_a_face_turn() -> Result<(), Box<dyn Error>> {
        assert!(Cube3x3::from_solved("x y2 z'")?.is_solved());
        assert!(!Cube3x3::from_solved("x y2 z' R")?.is_solved());
        Ok(())
    }

    #[test]
    fn r_4_times_is_solved() -> Result<(), Box<dyn Error>> {
        let cube = Cube3x3::from_solved("R R R R")?;
        assert!(cube.is_solved());
        Ok(())
    }

    #[test]
    fn r_2_is_equal_to_r_prime_2() -> Result<(), Box<dyn Error>> {
        let r2 = Cube3x3::from_solved("R2")?;
        let r_prime_2 = Cube3x3::from_solved("R' R'")?;
        assert_eq!(r2, r_prime_2);
        Ok(())
    }

    #[test]
    fn r_r_prime_is_solved() -> Result<(), Box<dyn Error>> {
        let mut cube = Cube3x3::from_solved("R")?;
        assert!(!cube.is_solved());
        cube = cube.move_sequence("R'")?;
        assert!(cube.is_solved());
        Ok(())
    }

    #[test]
    fn r_prime_is_inverse_of_r() -> Result<(), Box<dyn Error>> {
        let r = Cube3x3::from_solved("R")?;
        let r_prime = Cube3x3::from_solved("R'")?;
        assert_eq!(r.inverse(), r_prime);
        Ok(())
    }

    #[test]
    fn composing_u_and_r_works() -> Result<(), Box<dyn Error>> {
        let u = Cube3x3::from_solved("U")?;
        let ur = Cube3x3::from_solved("U R")?;
        assert_eq!(u.move_sequence("R")?, ur);
        Ok(())
    }

    #[test]
    fn composition_works() -> Result<(), Box<dyn Error>> {
        let scramble_string = "R' ";
        let solution_string = "D2";
        let scramble = Cube3x3::from_solved(scramble_string)?;
        assert_eq!(
            Cube3x3::from_solved(&format!("{scramble_string} {solution_string}"))?,
            scramble.move_sequence(solution_string)?,
            "composing {scramble_string} and {solution_string} doesn't result in applying {scramble_string} {solution_string}"
        );
        Ok(())
    }

    #[test]
    fn composition_works_on_fmc_wr() -> Result<(), Box<dyn Error>> {
        let scramble_string =
            "R' U' F D2 L2 F R2 U2 R2 B D2 L B2 D' B2 L' R' B D2 B U2 L U2 R' U' F";
        let solution_string = "D2 F' D2 U2 F' L2 D R2 D B2 F L2 R' F' D U'";
        let scramble = Cube3x3::from_solved(scramble_string)?;
        assert_eq!(
            Cube3x3::from_solved(&format!("{scramble_string} {solution_string}"))?,
            scramble.move_sequence(solution_string)?,
            "composing {scramble_string} and {solution_string} doesn't result in applying {scramble_string} {solution_string}"
        );
        Ok(())
    }

    #[test]
    fn mul_carries_orientation_along_with_the_piece() -> Result<(), Box<dyn Error>> {
        // Pre-twist the piece at UFR, then apply R: that piece lands at UBR and
        // its twist is added to the twist R gives the UBR slot.
        let r = Cube3x3::from_solved("R")?;
        let mut twisted = Cube3x3::default();
        twisted.corner_configuration.orientation[Corner::Ufr as usize] = Zn::new(1);
        let after = twisted * r;
        let mut expected = r;
        expected.corner_configuration.orientation[Corner::Ubr as usize] =
            expected.corner_configuration.orientation[Corner::Ubr as usize] + Zn::new(1);
        assert_eq!(after, expected);
        Ok(())
    }
    #[test]
    fn u_perm_repeats_after_3_applications() {
        use Edge::{Uf, Ul, Ur};
        let u_perm = Cube3x3 {
            edge_configuration: EdgeConfiguration::cycle([[Ur, Uf, Ul]]),
            ..Default::default()
        };
        assert_eq!(u_perm.pow(3), Cube3x3::default());
    }

    #[test]
    fn sequences_without_moves_leave_the_cube_unchanged() -> Result<(), Box<dyn Error>> {
        let cube = Cube3x3::from_solved("R U")?;
        assert_eq!(cube.move_sequence("")?, cube);
        assert_eq!(cube.move_sequence("// nothing here")?, cube);
        assert_eq!(
            Cube3x3::from_solved("\n  // only comments\n")?,
            Cube3x3::default()
        );
        Ok(())
    }
}
