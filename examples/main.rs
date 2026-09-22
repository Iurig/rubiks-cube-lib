#[allow(clippy::wildcard_imports)]
use rubiks_cube_lib::*;

fn main() {
    let scr = "D2 F2 R2 U L2 D R2 U' B2 L2 B L2 F' L D2 U R' B D2\n";
    let mut scrambled = Cube3x3::from_solved(scr).expect("deu ruim");

    let recon: String = (ROUX)
        .solve(&mut scrambled)
        .recon_with_options(NoOptions::default());

    assert_eq!(
        Cube3x3::from_solved(&(scr.to_string() + recon.as_str())).expect("deu OUTRO ruim"),
        Cube3x3::default()
    );
    println!("{recon}");
}
