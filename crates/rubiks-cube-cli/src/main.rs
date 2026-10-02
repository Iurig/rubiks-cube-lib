use clap::Parser;
use rubiks_cube::{Cube3x3, Method, Puzzle, RouxOptions};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    method: String,
}

fn main() {
    let args = Args::parse();
    match args.method {
        m if &m == "roux" => {
            let mut cube = Cube3x3::scramble();
            println!(
                "{:?}\n{}",
                Method::kociemba()
                    .solve(&mut cube.clone())
                    .unwrap()
                    .iter()
                    .flat_map(|s| s.moves().iter())
                    .rev(),
                Method::roux(RouxOptions::default())
                    .solve(&mut cube)
                    .unwrap()
            );
        }
        _ => println!("invalid method"),
    }
}
