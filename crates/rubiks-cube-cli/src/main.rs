use anyhow::ensure;
use clap::Parser;
use rubiks_cube::{Method, Puzzle, RouxOptions};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    method: String,

    #[arg(long)]
    number: u32,
}

fn reconstruct_n<P: Puzzle>(n: u32, technique: &Method<P>) -> anyhow::Result<()> {
    for count in 1..=n {
        let scramble = P::scramble()?;

        let mut puzzle = scramble.iter().fold(P::default(), |p, m| p * *m);

        let solution = technique.solve(&mut puzzle)?;

        ensure!(
            puzzle.is_solved(),
            "scramble:\n {scramble} \n and solve:\n {solution} \n did not solve the cube"
        );

        println!("{count}. {scramble} {solution}");
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.method {
        ref m if m == "roux" => {
            reconstruct_n(args.number, &Method::roux(RouxOptions::default()))?;
        }
        _ => println!("invalid method"),
    }

    Ok(())
}
