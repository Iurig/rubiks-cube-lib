use crate::{SinglePiece, single_piece::PieceConfiguration};

pub(crate) const CENTERS_COUNT: usize = 6;
pub(crate) const CORNERS_COUNT: usize = 8;
pub(crate) const EDGES_COUNT: usize = 12;
pub(crate) const CENTER_ORIENTATION_COUNT: usize = 1;
pub(crate) const CO_COUNT: usize = 3;
pub(crate) const EO_COUNT: usize = 2;

macro_rules! new_piece {
    ($type_name:ident, $amount:ident, [$($p:ident),+]) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[repr(u8)]
        pub(crate) enum $type_name {
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

pub(crate) type Centers = PieceConfiguration<SingleCenter, CENTERS_COUNT, CENTER_ORIENTATION_COUNT>;
pub(crate) type Corners = PieceConfiguration<SingleCorner, CORNERS_COUNT, CO_COUNT>;
pub(crate) type Edges = PieceConfiguration<SingleEdge, EDGES_COUNT, EO_COUNT>;
