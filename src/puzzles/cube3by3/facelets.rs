//! The sticker view of a cube state: which colour shows on each of the 54
//! facelets, printed as the usual unfolded net.
//!
//! A facelet is one square on the surface of the cube; a sticker is one face
//! of a piece. A cube state says which piece sits in each slot and how it is
//! turned, so it also says which sticker covers each facelet. A sticker is
//! named by the face it belongs to when solved, so every facelet reads as one
//! of `U F R B L D`.

use std::fmt::{self, Display};

use super::{
    Cube3x3,
    pieces::{Center, Corner, Edge, Pieces3x3},
};
use crate::{Piece, piece::index};

/// The colour on each facelet of a [`Cube3x3`], one `[Center; 9]` per face
/// in [`Center::ALL`] order, each read row by row as in [`Display`].
///
/// ```
/// use rubiks_cube_lib::Cube3x3;
///
/// let cube = Cube3x3::from_solved("R")?;
/// let expected = [
///     "    UUF",
///     "    UUF",
///     "    UUF",
///     "LLL FFD RRR UBB",
///     "LLL FFD RRR UBB",
///     "LLL FFD RRR UBB",
///     "    DDB",
///     "    DDB",
///     "    DDB",
/// ]
/// .join("\n");
/// assert_eq!(cube.to_string(), expected);
/// # Ok::<(), rubiks_cube_lib::ParseSequenceError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Facelets {
    faces: [[Center; 9]; 6],
}

/// Which slot and which of its faces each facelet belongs to, one `[_; 9]`
/// per face in [`Center::ALL`] order, `[U, F, R, B, L, D]`.
///
/// Each face is read as it is seen in the net below: `U` and `D` with `F`
/// toward the reader, the four side faces with `U` on top and unfolded left
/// to right as `L F R B`.
///
/// ```text
///     UUU
///     UUU
///     UUU
/// LLL FFF RRR BBB
/// LLL FFF RRR BBB
/// LLL FFF RRR BBB
///     DDD
///     DDD
///     DDD
/// ```
const NET: [[(Pieces3x3, Center); 9]; 6] = {
    use Center::{B, D, F, L, R, U};
    use Pieces3x3::{Center as Ce, Corner as Co, Edge as Ed};
    [
        [
            (Co(Corner::Ubl), U),
            (Ed(Edge::Ub), U),
            (Co(Corner::Ubr), U),
            (Ed(Edge::Ul), U),
            (Ce(U), U),
            (Ed(Edge::Ur), U),
            (Co(Corner::Ufl), U),
            (Ed(Edge::Uf), U),
            (Co(Corner::Ufr), U),
        ],
        [
            (Co(Corner::Ufl), F),
            (Ed(Edge::Uf), F),
            (Co(Corner::Ufr), F),
            (Ed(Edge::Fl), F),
            (Ce(F), F),
            (Ed(Edge::Fr), F),
            (Co(Corner::Dfl), F),
            (Ed(Edge::Df), F),
            (Co(Corner::Dfr), F),
        ],
        [
            (Co(Corner::Ufr), R),
            (Ed(Edge::Ur), R),
            (Co(Corner::Ubr), R),
            (Ed(Edge::Fr), R),
            (Ce(R), R),
            (Ed(Edge::Br), R),
            (Co(Corner::Dfr), R),
            (Ed(Edge::Dr), R),
            (Co(Corner::Dbr), R),
        ],
        [
            (Co(Corner::Ubr), B),
            (Ed(Edge::Ub), B),
            (Co(Corner::Ubl), B),
            (Ed(Edge::Br), B),
            (Ce(B), B),
            (Ed(Edge::Bl), B),
            (Co(Corner::Dbr), B),
            (Ed(Edge::Db), B),
            (Co(Corner::Dbl), B),
        ],
        [
            (Co(Corner::Ubl), L),
            (Ed(Edge::Ul), L),
            (Co(Corner::Ufl), L),
            (Ed(Edge::Bl), L),
            (Ce(L), L),
            (Ed(Edge::Fl), L),
            (Co(Corner::Dbl), L),
            (Ed(Edge::Dl), L),
            (Co(Corner::Dfl), L),
        ],
        [
            (Co(Corner::Dfl), D),
            (Ed(Edge::Df), D),
            (Co(Corner::Dfr), D),
            (Ed(Edge::Dl), D),
            (Ce(D), D),
            (Ed(Edge::Dr), D),
            (Co(Corner::Dbl), D),
            (Ed(Edge::Db), D),
            (Co(Corner::Dbr), D),
        ],
    ]
};

/// The three faces of a corner slot, clockwise as seen from outside the cube,
/// starting from its U or D face.
///
/// A piece is named by its home slot, so this is also the order of a corner
/// piece's stickers, starting from its U or D sticker. A twist of `t` puts
/// that sticker `t` steps clockwise from the slot's U or D face.
const fn corner_faces(corner: Corner) -> [Center; 3] {
    use Center::{B, D, F, L, R, U};
    match corner {
        Corner::Ubl => [U, L, B],
        Corner::Ubr => [U, B, R],
        Corner::Ufr => [U, R, F],
        Corner::Ufl => [U, F, L],
        Corner::Dfl => [D, L, F],
        Corner::Dfr => [D, F, R],
        Corner::Dbr => [D, R, B],
        Corner::Dbl => [D, B, L],
    }
}

/// The two faces of an edge slot, the one that defines its flip first: U or
/// D when the slot has one, otherwise F or B.
///
/// As with corners this is also the sticker order of the edge piece. An edge
/// is oriented when its first sticker is on the slot's first face.
const fn edge_faces(edge: Edge) -> [Center; 2] {
    use Center::{B, D, F, L, R, U};
    match edge {
        Edge::Ub => [U, B],
        Edge::Ur => [U, R],
        Edge::Uf => [U, F],
        Edge::Ul => [U, L],
        Edge::Fl => [F, L],
        Edge::Fr => [F, R],
        Edge::Br => [B, R],
        Edge::Bl => [B, L],
        Edge::Df => [D, F],
        Edge::Dr => [D, R],
        Edge::Db => [D, B],
        Edge::Dl => [D, L],
    }
}

/// The faces a slot has, in the order its stickers are counted.
fn slot_faces(slot: Pieces3x3) -> Vec<Center> {
    match slot {
        Pieces3x3::Center(c) => vec![c],
        Pieces3x3::Edge(e) => edge_faces(e).to_vec(),
        Pieces3x3::Corner(c) => corner_faces(c).to_vec(),
    }
}

/// The stickers now covering `slot`, in the same order as [`slot_faces`].
fn stickers_at(cube: &Cube3x3, slot: Pieces3x3) -> Vec<Center> {
    match slot {
        Pieces3x3::Center(c) => vec![cube.centers().piece_at(&c)],
        Pieces3x3::Edge(e) => {
            let mut stickers = edge_faces(cube.edges().piece_at(&e));
            stickers.rotate_left(cube.edges().orientation_at(&e).value());
            stickers.to_vec()
        }
        Pieces3x3::Corner(c) => {
            let mut stickers = corner_faces(cube.corners().piece_at(&c));
            stickers.rotate_right(cube.corners().orientation_at(&c).value());
            stickers.to_vec()
        }
    }
}

/// The sticker showing on the `facelet` face of `slot`, or `None` when the
/// slot has no face there.
fn sticker(cube: &Cube3x3, slot: Pieces3x3, facelet: Center) -> Option<Center> {
    let position = slot_faces(slot).iter().position(|&f| f == facelet)?;
    stickers_at(cube, slot).get(position).copied()
}

const fn letter(face: Center) -> char {
    match face {
        Center::U => 'U',
        Center::F => 'F',
        Center::R => 'R',
        Center::B => 'B',
        Center::L => 'L',
        Center::D => 'D',
    }
}

impl Facelets {
    /// The nine facelets of `face`, read row by row as in [`Display`].
    #[must_use]
    pub const fn face(&self, face: Center) -> &[Center; 9] {
        &self.faces[index(face)]
    }
}

impl Cube3x3 {
    /// The colour on every facelet of this cube state.
    #[must_use]
    #[expect(
        clippy::missing_panics_doc,
        reason = "the `const _` block below checks every entry of `NET`, so `sticker` never returns `None`"
    )]
    pub fn facelets(&self) -> Facelets {
        Facelets {
            faces: NET.map(|face| {
                face.map(|(slot, facelet)| {
                    sticker(self, slot, facelet)
                        .expect("the `const _` block below checks every entry of `NET`")
                })
            }),
        }
    }
}

impl Display for Facelets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rows = |face: Center| self.face(face).chunks(3);
        let write_row = |f: &mut fmt::Formatter<'_>, row: &[Center]| {
            row.iter().try_for_each(|&c| write!(f, "{}", letter(c)))
        };
        for row in rows(Center::U) {
            write!(f, "    ")?;
            write_row(f, row)?;
            writeln!(f)?;
        }
        for (l, fr, r, b) in rows(Center::L)
            .zip(rows(Center::F))
            .zip(rows(Center::R))
            .zip(rows(Center::B))
            .map(|(((l, fr), r), b)| (l, fr, r, b))
        {
            write_row(f, l)?;
            write!(f, " ")?;
            write_row(f, fr)?;
            write!(f, " ")?;
            write_row(f, r)?;
            write!(f, " ")?;
            write_row(f, b)?;
            writeln!(f)?;
        }
        for (i, row) in rows(Center::D).enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "    ")?;
            write_row(f, row)?;
        }
        Ok(())
    }
}

impl Display for Cube3x3 {
    /// The facelet net; see [`Facelets`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.facelets().fmt(f)
    }
}

/// Every entry of [`NET`] names a face its slot has, sits on the face its row
/// belongs to, and appears once.
const _: () = {
    const fn same_face(a: Center, b: Center) -> bool {
        a as u8 == b as u8
    }
    const fn same_slot(a: Pieces3x3, b: Pieces3x3) -> bool {
        match (a, b) {
            (Pieces3x3::Center(x), Pieces3x3::Center(y)) => same_face(x, y),
            (Pieces3x3::Edge(x), Pieces3x3::Edge(y)) => x as u8 == y as u8,
            (Pieces3x3::Corner(x), Pieces3x3::Corner(y)) => x as u8 == y as u8,
            _ => false,
        }
    }
    const fn slot_has(slot: Pieces3x3, face: Center) -> bool {
        match slot {
            Pieces3x3::Center(c) => same_face(c, face),
            Pieces3x3::Edge(e) => {
                let faces = edge_faces(e);
                same_face(faces[0], face) || same_face(faces[1], face)
            }
            Pieces3x3::Corner(c) => {
                let faces = corner_faces(c);
                same_face(faces[0], face) || same_face(faces[1], face) || same_face(faces[2], face)
            }
        }
    }
    let mut face = 0;
    while face < NET.len() {
        let mut i = 0;
        while i < NET[face].len() {
            let (slot, facelet) = NET[face][i];
            assert!(slot_has(slot, facelet), "facelet not on its slot");
            assert!(
                same_face(facelet, Center::ALL[face]),
                "facelet on the wrong face of the net"
            );
            let mut other_face = 0;
            while other_face < NET.len() {
                let mut j = 0;
                while j < NET[other_face].len() {
                    let (other_slot, other_facelet) = NET[other_face][j];
                    assert!(
                        (face == other_face && i == j)
                            || !(same_slot(slot, other_slot) && same_face(facelet, other_facelet)),
                        "facelet listed twice"
                    );
                    j += 1;
                }
                other_face += 1;
            }
            i += 1;
        }
        face += 1;
    }
};

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "tests should panic if failed, and return result for `?` convenience"
)]
mod tests {
    use std::error::Error;

    use super::*;
    use crate::{Piece, Puzzle};

    fn net(rows: [&str; 9]) -> String {
        let [u1, u2, u3, m1, m2, m3, d1, d2, d3] = rows;
        format!("    {u1}\n    {u2}\n    {u3}\n{m1}\n{m2}\n{m3}\n    {d1}\n    {d2}\n    {d3}")
    }

    #[test]
    fn solved_cube_shows_each_face_in_one_colour() {
        assert_eq!(
            Cube3x3::default().to_string(),
            net([
                "UUU",
                "UUU",
                "UUU",
                "LLL FFF RRR BBB",
                "LLL FFF RRR BBB",
                "LLL FFF RRR BBB",
                "DDD",
                "DDD",
                "DDD",
            ])
        );
    }

    // Physical facts: `R` carries F stickers to U, U to B, B to D, D to F.
    #[test]
    fn r_moves_the_right_column_of_each_face_along_the_ring() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            Cube3x3::from_solved("R")?.to_string(),
            net([
                "UUF",
                "UUF",
                "UUF",
                "LLL FFD RRR UBB",
                "LLL FFD RRR UBB",
                "LLL FFD RRR UBB",
                "DDB",
                "DDB",
                "DDB",
            ])
        );
        Ok(())
    }

    // `U` carries F stickers to L, L to B, B to R, R to F.
    #[test]
    fn u_moves_the_top_row_of_the_side_faces() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            Cube3x3::from_solved("U")?.to_string(),
            net([
                "UUU",
                "UUU",
                "UUU",
                "FFF RRR BBB LLL",
                "LLL FFF RRR BBB",
                "LLL FFF RRR BBB",
                "DDD",
                "DDD",
                "DDD",
            ])
        );
        Ok(())
    }

    // `F` carries U stickers to R, R to D, D to L, L to U. It twists the four
    // F corners and flips the four F edges, so this pins both conventions.
    #[test]
    fn f_moves_the_ring_around_the_front_face() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            Cube3x3::from_solved("F")?.to_string(),
            net([
                "UUU",
                "UUU",
                "LLL",
                "LLD FFF URR BBB",
                "LLD FFF URR BBB",
                "LLD FFF URR BBB",
                "RRR",
                "DDD",
                "DDD",
            ])
        );
        Ok(())
    }

    // `M` follows `L`: U stickers go to F, F to D, D to B, B to U.
    #[test]
    fn m_moves_the_middle_column_and_the_centers() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            Cube3x3::from_solved("M")?.to_string(),
            net([
                "UBU",
                "UBU",
                "UBU",
                "LLL FUF RRR BDB",
                "LLL FUF RRR BDB",
                "LLL FUF RRR BDB",
                "DFD",
                "DFD",
                "DFD",
            ])
        );
        Ok(())
    }

    // `y` follows `U`: the whole cube turns, so R stickers show at F.
    #[test]
    fn y_rotates_the_side_faces_whole() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            Cube3x3::from_solved("y")?.to_string(),
            net([
                "UUU",
                "UUU",
                "UUU",
                "FFF RRR BBB LLL",
                "FFF RRR BBB LLL",
                "FFF RRR BBB LLL",
                "DDD",
                "DDD",
                "DDD",
            ])
        );
        Ok(())
    }

    #[test]
    fn face_reads_the_same_facelets_display_prints() -> Result<(), Box<dyn Error>> {
        use Center::{B, D, F, R, U};
        let facelets = Cube3x3::from_solved("R")?.facelets();
        assert_eq!(facelets.face(U), &[U, U, F, U, U, F, U, U, F]);
        assert_eq!(facelets.face(B), &[U, B, B, U, B, B, U, B, B]);
        assert_eq!(facelets.face(R), &[R; 9]);
        assert_eq!(facelets.face(D), &[D, D, B, D, D, B, D, D, B]);
        Ok(())
    }

    #[test]
    fn every_state_shows_nine_stickers_of_each_colour() {
        let mut rng = fastrand::Rng::with_seed(7);
        for _ in 0..50 {
            let facelets = Cube3x3::random_state_with_seed(&mut rng).facelets();
            for colour in Center::ALL {
                let count = Center::ALL
                    .iter()
                    .flat_map(|&face| facelets.face(face))
                    .filter(|&&c| c == colour)
                    .count();
                assert_eq!(count, 9, "{colour:?} in\n{facelets}");
            }
        }
    }

    #[test]
    fn a_move_and_its_inverse_restore_the_net() -> Result<(), Box<dyn Error>> {
        let solved = Cube3x3::default().to_string();
        for m in [
            "R", "U", "F", "L", "D", "B", "M", "E", "S", "x", "y", "z", "r",
        ] {
            let there_and_back = Cube3x3::from_solved(&format!("{m} {m}'"))?;
            assert_eq!(there_and_back.to_string(), solved, "{m}");
        }
        Ok(())
    }
}
