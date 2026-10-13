#![allow(
    clippy::pedantic,
    clippy::nursery,
    clippy::allow_attributes_without_reason,
    missing_docs
)]
use rubiks_cube::*;

fn main() {
    let fb = PieceSet::<Cube3x3>::from_algset(&parts!("R U r")).split_if();
}
