use anyhow::Context;
use rubiks_cube::*;

#[test]
fn full_solve_and_checking_bfs() -> anyhow::Result<()> {
    let scr = "U B' D L2 U B2 R2 D2 L2 D' U2 B2 R2 L U2 F' R' B L D'\n";
    let mut scrambled = Cube3x3::from_moves(scr)?;

    let recon: String = Roux::default().solve(&mut scrambled)?.to_string();

    assert_eq!(
        Cube3x3::from_moves(&(scr.to_string() + recon.as_str()))?,
        Cube3x3::default(),
        "{recon}"
    );
    Ok(())
}

#[test]
fn every_roux_option_combination_solves() -> anyhow::Result<()> {
    use rubiks_cube::roux::*;
    let scrambles: Vec<Cube3x3> = (2026..2030).map(Cube3x3::scrambled_with_seed).collect();
    for fb in [
        FirstBlock::OneLook,
        FirstBlock::SquarePair,
        FirstBlock::EdgePairPair,
    ] {
        for sb in [
            SecondBlock::OneLook,
            SecondBlock::SquarePair,
            SecondBlock::EdgePairPair,
        ] {
            for cmll in [Cmll::OneLook, Cmll::TwoLook] {
                let roux = Roux::default().first_block(fb).second_block(sb).cmll(cmll);
                let described = format!("{roux:?}");
                for (i, scrambled) in scrambles.iter().enumerate() {
                    let recon = roux
                        .solve(&mut scrambled.clone())
                        .with_context(|| format!("{described}, scramble {i}"))?
                        .to_string();
                    assert!(
                        scrambled.apply_moves(&recon)?.is_solved(),
                        "{described} did not solve scramble {i}:\n{scrambled}\n{recon}"
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn skips_work() -> anyhow::Result<()> {
    let mut scrambled = Cube3x3::from_moves("U2")?;
    let recon = Roux::default().solve(&mut scrambled)?.to_string();
    assert!(scrambled.is_solved(), "{recon}");
    Ok(())
}

#[test]
fn zz_solves_the_cube() -> anyhow::Result<()> {
    let mut cube = Cube3x3::scrambled_with_seed(2);

    let _ = ZZ::default()
        .solve_steps(&mut cube)
        .collect::<Result<Solution<Cube3x3>, _>>()?;
    assert!(cube.is_solved());
    Ok(())
}

#[test]
fn every_zz_option_combination_solves() -> anyhow::Result<()> {
    use rubiks_cube::zz::{FirstLook, LL, SecondLook};
    let mut last_layers = vec![LL::OneLook];
    for first in [FirstLook::Ocll, FirstLook::Coll] {
        for second in [SecondLook::OneLook, SecondLook::TwoLook] {
            last_layers.push(LL::TwoLook { first, second });
        }
    }
    for (seed, ll) in (0..).zip(last_layers) {
        let zz = ZZ::default().ll(ll);
        let mut cube = Cube3x3::scrambled_with_seed(seed);
        zz.solve(&mut cube)
            .with_context(|| format!("{zz:?}, seed {seed}"))?;
        assert!(
            cube.is_solved(),
            "{zz:?} did not solve seed {seed}:\n{cube}"
        );
    }
    Ok(())
}

/// ZZ's EO Line: the goal names a piece in two edge slots and asks only for orientation in the
/// other ten. When the search key left out the flip of a stray edge in a named slot, seed 2 ended
/// this step with two edges flipped.
#[test]
fn a_goal_mixing_named_and_orientation_only_slots_is_met() -> anyhow::Result<()> {
    let eo_line_moves = AlgSet::from_static_parts("F B U R L D");
    let step = SearchStep::builder("EO Line", eo_line_moves.clone())
        .goal(PieceSet::from_algset(&AlgSet::from_static_parts("U R L")))
        .expect_solved(PieceSet::from_algset(&eo_line_moves))
        .build()?;
    for seed in 0..4 {
        let mut cube = Cube3x3::scrambled_with_seed(seed);
        step.solve(&mut cube)?;
        assert!(step.is_done(&cube), "seed {seed}:\n{cube}");
    }
    Ok(())
}
