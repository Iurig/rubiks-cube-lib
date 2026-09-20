use rubiks_cube_lib::*;

const FMC: SteppedMethod<1, StepByPiece> = SteppedMethod([do_all]);

const SCRAMBLED: Cube3By3 = Cube3By3::random_state();

#[test]
fn nxn() {
    let scrambled: NxN<3> = NxN::<3>::random_state();
}

#[test]
fn manual_scramble_creation() {
    let scramble: String = FMC
        .solve(SCRAMBLED.inverse())
        .map(|(step, step_name)| {
            (step
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<String>>()
                .join(" ")
                + "\t\\")
                + &step_name
                + "\n"
        }) // Iter<String>
        .collect();
}

#[test]
fn roux_recon() {
    let recon: String = Roux {
        cmll: one_look,
        ..Default::default()
    }
    .solve(scrambled)
    .to_recon();
}

#[test]
fn cfop_recon() {
    let cfop_recon: String = CFOP::default().solve(scrambled).to_recon();
}
