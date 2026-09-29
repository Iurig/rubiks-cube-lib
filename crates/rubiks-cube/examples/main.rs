use rubiks_cube_lib::{Cube3x3, Method, Puzzle, RouxOptions};

#[expect(
    clippy::panic_in_result_fn,
    reason = "`?` reports errors; `assert!` checks the recon solves the scramble"
)]
#[expect(
    clippy::print_stdout,
    reason = "the example prints the recons it finds"
)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Solver progress is logged; see it with `RUST_LOG=rubiks_cube_lib=trace`.
    env_logger::init();
    let scr = "D2 F2 R2 U L2 D R2 U' B2 L2 B L2 F' L D2 U R' B D2\n";
    let mut scrambled = Cube3x3::from_solved(scr)?;

    let recon_beginner_options: String = scr.to_string()
        + Method::roux(RouxOptions {
            fb_as_one_step: false,
            sb_as_one_step: false,
            one_look_cmll: false,
        })
        .solve(&mut scrambled)?
        .to_string()
        .as_str();
    assert!(Cube3x3::from_solved(&recon_beginner_options)?.is_solved());

    println!("{recon_beginner_options}");

    scrambled = Cube3x3::from_solved(scr)?;

    let recon_default_options: String = scr.to_string()
        + Method::roux(RouxOptions::default())
            .solve(&mut scrambled)?
            .to_string()
            .as_str();

    assert!(Cube3x3::from_solved(&recon_default_options)?.is_solved());

    println!("{recon_default_options}");

    Ok(())
}
