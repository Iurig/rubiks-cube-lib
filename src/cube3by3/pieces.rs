use crate::{Piece, piece::PieceConfiguration};

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
        pub enum $type_name {
            $($p),+
        }
        unsafe impl Piece<$amount> for $type_name {
            const ALL: [Self; $amount] = [
                $($type_name::$p),+
            ];
        }
        const _: () = {
            let mut i = 0;
            while i < $amount {
                assert!($type_name::ALL[i] as usize == i);
                i += 1;
            }
        };
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

pub type Centers = PieceConfiguration<SingleCenter, CENTERS_COUNT, CENTER_ORIENTATION_COUNT>;
pub type Corners = PieceConfiguration<SingleCorner, CORNERS_COUNT, CO_COUNT>;
pub type Edges = PieceConfiguration<SingleEdge, EDGES_COUNT, EO_COUNT>;
pub type Faces = SingleCenter;
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Slices {
    M,
    S,
    E,
}
impl Slices {
    pub const ALL: [Self; 3] = [Self::M, Self::S, Self::E];
}
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
#[expect(non_camel_case_types, reason = "rotations are inherently lower case")]
pub enum Rotations {
    x,
    y,
    z,
}
impl Rotations {
    pub const ALL: [Self; 3] = [Self::x, Self::y, Self::z];
}
