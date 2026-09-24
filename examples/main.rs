#[allow(clippy::wildcard_imports)]
use rubiks_cube_lib::*;

#[allow(clippy::panic_in_result_fn)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scr = "D2 F2 R2 U L2 D R2 U' B2 L2 B L2 F' L D2 U R' B D2\n";
    let mut scrambled = Cube3x3::from_solved(scr).expect("deu ruim");

    let recon_beginner_options: String = scr.to_string()
        + Roux::from_options(RouxOptions {
            fb_as_one_step: false,
            sb_square_as_one_step: false,
            one_look_cmll: false,
        })
        .solve(&mut scrambled)?
        .recon_with_options(NoOptions::default())
        .as_str();
    assert!(
        Cube3x3::from_solved(&recon_beginner_options)
            .expect("deu OUTRO ruim")
            .is_solved()
    );

    println!("{recon_beginner_options}");

    scrambled = Cube3x3::from_solved(scr).expect("deu ruim");

    let recon_default_options: String = scr.to_string()
        + Roux::default()
            .solve(&mut scrambled)?
            .recon_with_options(NoOptions::default())
            .as_str();

    assert!(
        Cube3x3::from_solved(&recon_default_options)
            .expect("deu OUTRO ruim")
            .is_solved(),
    );

    println!("{recon_default_options}");

    Ok(())
}
