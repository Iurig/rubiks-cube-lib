use anyhow::Context;
use clap::Parser;
use rubiks_cube::{Algorithm, Cube3x3, Inv, Method, Puzzle, RouxOptions};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    method: String,

    #[arg(long)]
    number: u32,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    for solve in 1..=args.number {
        match args.method {
            ref m if m == "roux" => {
                let mut cube = Cube3x3::scramble();

                let scramble = Method::kociemba()
                    .solve(&mut cube.clone())
                    .unwrap()
                    .iter()
                    .flat_map(|s| s.moves().iter().copied())
                    .collect::<Algorithm<Cube3x3>>()
                    .inverse();

                let roux = Method::roux(RouxOptions::default())
                    .solve(&mut cube)
                    .unwrap();

                println!("{solve}. {scramble}\n{roux}");
                assert!(
                    Cube3x3::from_solved(&format!("{scramble} {roux}"))
                        .context("The previous scramble didn't result in a solved cube.")?
                        .is_solved()
                );
            }
            _ => println!("invalid method"),
        }
    }
    Ok(())
}
