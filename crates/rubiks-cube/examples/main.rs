//! Solves one scramble using the Roux method with a few different options.
use rubiks_cube::{
    CMLLOptions, Cube3x3, FirstBlockOptions, Method, Puzzle, Roux, SecondBlockOptions,
};

#[expect(
    clippy::panic_in_result_fn,
    reason = "`?` reports errors; `assert!` checks the recon solves the scramble"
)]
#[expect(
    clippy::print_stdout,
    reason = "the example prints the recons it finds"
)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Solver progress is logged; see it with `RUST_LOG=rubiks_cube=trace`.
    env_logger::init();
    let scr = "D2 F2 R2 U L2 D R2 U' B2 L2 B L2 F' L D2 U R' B D2\n";
    let mut scrambled = Cube3x3::from_solved(scr)?;

    let recon_beginner_options: String = scr.to_string()
        + Roux::default()
            .first_block(FirstBlockOptions::SquarePair)
            .second_block(SecondBlockOptions::SquarePair)
            .cmll(CMLLOptions::TwoLook)
            .solve(&mut scrambled)?
            .to_string()
            .as_str();
    assert!(Cube3x3::from_solved(&recon_beginner_options)?.is_solved());

    println!("{recon_beginner_options}");

    scrambled = Cube3x3::from_solved(scr)?;

    let recon_default_options: String =
        scr.to_string() + Roux::default().solve(&mut scrambled)?.to_string().as_str();

    assert!(Cube3x3::from_solved(&recon_default_options)?.is_solved());

    println!("{recon_default_options}");

    Ok(())
}
