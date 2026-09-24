//! Benchmark: two passes of 1000 seeded random solves with default Roux, timed per step.
//!
//! Run with `cargo run --release --example bench_solves > /dev/null`: the solver's progress goes
//! to stdout, the results to stderr. The first solve of each pass is timed apart.
use std::{
    collections::BTreeMap,
    error::Error,
    time::{Duration, Instant},
};

use rubiks_cube_lib::{
    Cube3x3, Mask, NamedMoveSequences, Puzzle, Roux, Solution, SolveMethod, SolveStep,
};

const SOLVES: u32 = 1000;

/// Moves and times for one step or for whole solves.
#[derive(Default)]
struct Stats {
    lengths: Vec<usize>,
    times: Vec<Duration>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let roux = Roux::default();
    // Pass 2 gets new scrambles but keeps the memos pass 1 grew, so what pass 2 no longer
    // pays for is memo growth.
    for (pass, seed) in [(1, 2026), (2, 2027)] {
        eprintln!("pass {pass} (seed {seed}):");
        bench(&roux, seed)?;
    }
    Ok(())
}

fn bench(roux: &Roux, seed: u64) -> Result<(), Box<dyn Error>> {
    let mut rng = fastrand::Rng::with_seed(seed);
    let mut whole = Stats::default();
    let mut per_step: BTreeMap<String, Stats> = BTreeMap::new();
    let start = Instant::now();

    for i in 0..SOLVES {
        let scrambled = Cube3x3::random_state_with_seed(&mut rng);
        let mut cube = scrambled;
        let t = Instant::now();
        let (solution, step_times) = solve_timed(roux, &mut cube)?;
        let elapsed = t.elapsed();
        if !scrambled.move_sequence(&solution.to_recon())?.is_solved() {
            return Err(format!("solve {i} does not solve its scramble").into());
        }
        if i == 0 {
            eprintln!("  first solve: {elapsed:.2?}");
            for ((name, _), time) in solution.iter().zip(&step_times) {
                eprintln!("    {name}: {time:.2?}");
            }
            continue;
        }
        let mut total = 0;
        for ((name, moves), time) in solution.iter().zip(step_times) {
            let stats = per_step.entry(name.clone()).or_default();
            stats.lengths.push(moves.len());
            stats.times.push(time);
            total += moves.len();
        }
        whole.lengths.push(total);
        whole.times.push(elapsed);
    }

    eprintln!("  wall time: {:.2?}", start.elapsed());
    eprintln!("  solves 2-{SOLVES}:");
    eprintln!("    whole solve: {}", summary(&mut whole)?);
    for (name, stats) in &mut per_step {
        eprintln!("    {name}: {}", summary(stats)?);
    }
    Ok(())
}

/// `SolveMethod::solve`, with a timer around each step.
fn solve_timed(
    roux: &Roux,
    cube: &mut Cube3x3,
) -> Result<(NamedMoveSequences<Cube3x3>, Vec<Duration>), Box<dyn Error>> {
    let mut solution = NamedMoveSequences::<Cube3x3>::new();
    let mut times = Vec::new();
    let mut solved_pieces = Mask::<Cube3x3>::default();

    for _ in 0..100 {
        if cube.is_solved() {
            return Ok((solution, times));
        }
        for step in roux.steps().iter().filter(|s| s.options_allow(roux)) {
            if step.can_apply(cube) && solved_pieces == step.needs_solved() {
                let t = Instant::now();
                let step_solution = step.solve(cube)?;
                times.push(t.elapsed());
                solved_pieces = step.solved_pieces();
                solution.extend(step_solution);
            }
        }
    }
    Err("no solve after 100 passes over the steps".into())
}

/// Mean, median, p99, and max time, then mean and max moves.
fn summary(stats: &mut Stats) -> Result<String, std::num::TryFromIntError> {
    stats.times.sort();
    let count = u32::try_from(stats.times.len())?;
    let percentile = |p: usize| {
        stats
            .times
            .get(stats.times.len().saturating_sub(1) * p / 100)
            .copied()
            .unwrap_or_default()
    };
    let moves = u32::try_from(stats.lengths.iter().sum::<usize>())?;
    Ok(format!(
        "time mean {:.2?}, median {:.2?}, p99 {:.2?}, max {:.2?} | moves mean {:.1}, max {} ({count} runs)",
        stats.times.iter().sum::<Duration>() / count.max(1),
        percentile(50),
        percentile(99),
        percentile(100),
        f64::from(moves) / f64::from(count.max(1)),
        stats.lengths.iter().max().copied().unwrap_or_default(),
    ))
}
