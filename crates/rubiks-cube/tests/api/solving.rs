use rubiks_cube::*;
use std::error::Error;

#[test]
fn full_solve_and_checking_bfs() -> Result<(), Box<dyn Error>> {
    let scr = "U B' D L2 U B2 R2 D2 L2 D' U2 B2 R2 L U2 F' R' B L D'\n";
    let mut scrambled = Cube3x3::from_moves(scr).expect("deu ruim");

    let recon: String = Roux::default().solve(&mut scrambled)?.to_string();

    assert_eq!(
        Cube3x3::from_moves(&(scr.to_string() + recon.as_str())).expect("deu OUTRO ruim"),
        Cube3x3::default()
    );
    println!("{recon}");
    Ok(())
}

#[test]
fn every_roux_option_combination_solves() -> Result<(), Box<dyn Error>> {
    use rubiks_cube::roux::*;
    let scrambles: Vec<Cube3x3> = (2026..2030)
        .map(Cube3x3::apply_scramble_with_seed)
        .collect();
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
                        .map_err(|e| format!("{described}, scramble {i}: {e}"))?
                        .to_string();
                    assert!(
                        scrambled.move_sequence(&recon)?.is_solved(),
                        "{described} did not solve scramble {i}:\n{scrambled}\n{recon}"
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn skips_work() -> Result<(), Box<dyn Error>> {
    let mut scrambled = Cube3x3::from_moves("U2")?;
    let recon = Roux::default().solve(&mut scrambled)?.to_string();
    println!("{recon}");
    assert!(scrambled.is_solved());
    Ok(())
}
