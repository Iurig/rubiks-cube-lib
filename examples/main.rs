#[allow(clippy::wildcard_imports)]
use rubiks_cube_lib::*;

fn main() {
    let mut scrambled =
        Cube3x3::from_solved("U B' D L2 U B2 R2 D2 L2 D' U2 B2 R2 L U2 F' R' B L D'")
            .expect("deu ruim");

    let recon: String = (ROUX).solve(&mut scrambled).to_recon(NoOptions::default());

    println!("{recon}");
}
