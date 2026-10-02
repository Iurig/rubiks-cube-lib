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

fn main() {
    let args = Args::parse();
    for solve in 1..=args.number {
        match args.method {
            ref m if m == "roux" => {
                let mut cube = Cube3x3::scramble();
                // The inverse of a solution is a scramble that leads to the same state.
                let kociemba: Algorithm<Cube3x3> = Method::kociemba()
                    .solve(&mut cube.clone())
                    .unwrap()
                    .iter()
                    .flat_map(|s| s.moves().iter().copied())
                    .collect();
                println!(
                    "{solve}. {}\n{}",
                    kociemba.inverse(),
                    Method::roux(RouxOptions::default())
                        .solve(&mut cube)
                        .unwrap()
                );
            }
            _ => println!("invalid method"),
        }
    }
}
