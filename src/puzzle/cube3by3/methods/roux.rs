#[allow(clippy::enum_glob_use)]
use crate::{Center::*, Corner::*, Edge::*, StepByPiece, puzzle::cube3by3::pieces::Pieces3By3::*};

pub const FIRST_BLOCK: StepByPiece = StepByPiece(&[
    Center(L),
    Corner(Dfl),
    Corner(Dbl),
    Edge(Dl),
    Edge(Fl),
    Edge(Bl),
]);

pub const SECOND_BLOCK: StepByPiece = StepByPiece(&[
    Center(R),
    Corner(Dfr),
    Corner(Dbr),
    Edge(Fr),
    Edge(Dr),
    Edge(Br),
]);

pub const CMLL: StepByPiece = StepByPiece(&[Corner(Ufl), Corner(Ufr), Corner(Ubl), Corner(Ubr)]);

pub const LSE: StepByPiece = StepByPiece(&[
    Edge(Uf),
    Edge(Ur),
    Edge(Ul),
    Edge(Ub),
    Edge(Df),
    Edge(Db),
    Center(F),
    Center(U),
    Center(B),
    Center(D),
]);

pub enum Cmll {
    OneLook,
    TwoLook,
}

pub struct Roux {
    pub cmll: Cmll,
}
impl Default for Roux {
    fn default() -> Self {
        Self {
            cmll: Cmll::OneLook,
        }
    }
}

impl  {
    
}
